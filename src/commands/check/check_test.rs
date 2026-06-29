#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::config::{Config, PreexistingError, Repository};
    use crate::tech_spec::{
        Check, CheckKind, CheckOverride, RepoCheck, RunScope, Severity, TechSpec,
    };
    use serial_test::serial;
    use std::collections::HashMap;
    use tempfile::{TempDir, tempdir};

    fn check(kind: CheckKind, pattern: Option<&str>) -> Check {
        Check {
            name: "sample".to_string(),
            description: None,
            kind,
            pattern: pattern.map(str::to_string),
            negate: false,
            script: None,
            run: RunScope::PerFile,
            include: Vec::new(),
            exclude: Vec::new(),
            severity: Severity::Error,
        }
    }

    fn script_check(reference: &str, run: RunScope) -> Check {
        let mut check = check(CheckKind::Script, None);
        check.script = Some(reference.to_string());
        check.run = run;
        check
    }

    fn make_executable(path: &Path) {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(path).expect("metadata").permissions();
        permissions.set_mode(permissions.mode() | 0o111);
        std::fs::set_permissions(path, permissions).expect("set permissions");
    }

    fn scoped(dir: &TempDir, rel: &str, contents: &str) -> ScopedFile {
        let abs = dir.path().join(rel);
        if let Some(parent) = abs.parent() {
            std::fs::create_dir_all(parent).expect("create parent");
        }
        std::fs::write(&abs, contents).expect("write file");
        ScopedFile {
            abs,
            rel: PathBuf::from(rel),
        }
    }

    #[test]
    fn provided_files_resolves_existing_relative_paths() {
        let dir = tempdir().expect("temp dir");
        scoped(&dir, "src/api/main.rs", "");
        scoped(&dir, "README.md", "");

        let files = vec![
            "src/api/main.rs".to_string(),
            "./README.md".to_string(),
            "missing.rs".to_string(),
        ];
        let resolved = provided_files(dir.path(), &files);
        let names: Vec<String> = resolved.iter().map(rel_str).collect();

        assert_eq!(
            names,
            vec!["src/api/main.rs".to_string(), "README.md".to_string()]
        );
    }

    #[test]
    fn build_globset_returns_none_for_empty() {
        assert!(build_globset(&[]).expect("empty globset").is_none());
    }

    #[test]
    fn build_globset_matches_patterns() {
        let set = build_globset(&["**/*.rs".to_string()])
            .expect("valid globset")
            .expect("some globset");
        assert!(set.is_match(Path::new("src/main.rs")));
        assert!(!set.is_match(Path::new("src/main.ts")));
    }

    #[test]
    fn regexp_forbidden_reports_each_match() {
        let dir = tempdir().expect("temp dir");
        let file = scoped(
            &dir,
            "src/main.rs",
            "let api_key = \"superSecretValue\";\nok();\n",
        );
        let check = check(CheckKind::Regexp, Some("api_key\\s*=\\s*\"[^\"]{8,}\""));

        let mut findings = Vec::new();
        run_regexp("TECH-SECRET-HANDLING", &check, &[&file], &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Error);
        assert_eq!(findings[0].location.as_deref(), Some("src/main.rs:1"));
    }

    #[test]
    fn regexp_negate_reports_when_absent() {
        let dir = tempdir().expect("temp dir");
        let file = scoped(&dir, "src/main.rs", "fn main() {}\n");
        let mut check = check(CheckKind::Regexp, Some("Copyright"));
        check.negate = true;

        let mut findings = Vec::new();
        run_regexp("TECH-LICENSE", &check, &[&file], &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].location.as_deref(), Some("src/main.rs"));
    }

    #[test]
    fn glob_forbidden_reports_matching_paths() {
        let dir = tempdir().expect("temp dir");
        let bak = scoped(&dir, "notes.bak", "");
        let src = scoped(&dir, "src/main.rs", "");
        let check = check(CheckKind::Glob, Some("**/*.bak"));

        let mut findings = Vec::new();
        run_glob("TECH-SECRET-HANDLING", &check, &[&bak, &src], &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].location.as_deref(), Some("notes.bak"));
    }

    #[test]
    fn glob_negate_reports_when_required_absent() {
        let dir = tempdir().expect("temp dir");
        let src = scoped(&dir, "src/main.rs", "");
        let mut check = check(CheckKind::Glob, Some("**/LICENSE"));
        check.negate = true;

        let mut findings = Vec::new();
        run_glob("TECH-LICENSE", &check, &[&src], &mut findings);

        assert_eq!(findings.len(), 1);
        assert!(findings[0].location.is_none());
    }

    #[test]
    fn resolve_script_joins_scripts_and_spec_dir() {
        let workspace = tempdir().expect("workspace");
        let spex = workspace.path().join("spex");
        std::fs::create_dir_all(spex.join("scripts/TECH-X")).expect("spex scripts");
        std::fs::write(spex.join("scripts/TECH-X/lint.sh"), "").expect("spex script");

        let resolved = resolve_script("lint.sh", &spex, "TECH-X").expect("resolved");
        assert_eq!(resolved, spex.join("scripts/TECH-X/lint.sh"));
    }

    #[test]
    fn resolve_script_returns_none_when_missing() {
        let workspace = tempdir().expect("workspace");
        let spex = workspace.path().join("spex");
        std::fs::create_dir_all(&spex).expect("spex");

        assert!(resolve_script("lint.sh", &spex, "TECH-X").is_none());
    }

    // Tests that create a script and then execute it run serially: `Command`
    // forks, and a sibling thread's fork inherits the whole process fd table, so
    // it can briefly hold another test's still-open write handle to its script
    // open for writing, making that script's exec fail with ETXTBSY.
    #[test]
    #[serial(script_exec)]
    fn script_per_repo_reports_violation_on_failure() {
        let workspace = tempdir().expect("workspace");
        let repo = workspace.path().join("api");
        let spex = workspace.path().join("spex");
        std::fs::create_dir_all(spex.join("scripts/TECH-COMMIT-MESSAGE-FORMAT"))
            .expect("spex scripts");
        std::fs::create_dir_all(&repo).expect("repo");
        let script = spex.join("scripts/TECH-COMMIT-MESSAGE-FORMAT/sample.sh");
        std::fs::write(&script, "#!/bin/sh\necho boom\nexit 1\n").expect("write script");
        make_executable(&script);

        let mut check = script_check("sample.sh", RunScope::PerRepo);
        check.severity = Severity::Warn;

        let mut findings = Vec::new();
        run_per_repo_script(
            "TECH-COMMIT-MESSAGE-FORMAT",
            &check,
            &repo,
            &spex,
            &mut findings,
        );

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Warn);
        assert_eq!(findings[0].message, "boom");
    }

    #[test]
    #[serial(script_exec)]
    fn script_per_file_passes_files_as_arguments() {
        let workspace = tempdir().expect("workspace");
        let repo = workspace.path().join("api");
        let spex = workspace.path().join("spex");
        std::fs::create_dir_all(spex.join("scripts/TECH-WRITING-STYLE")).expect("spex scripts");
        std::fs::create_dir_all(&repo).expect("repo");
        let script = spex.join("scripts/TECH-WRITING-STYLE/echo-args.sh");
        std::fs::write(
            &script,
            "#!/bin/sh\n[ \"$#\" -gt 0 ] || exit 0\necho \"$1\"\nexit 1\n",
        )
        .expect("write script");
        make_executable(&script);

        let file = ScopedFile {
            abs: repo.join("README.md"),
            rel: PathBuf::from("README.md"),
        };
        let check = script_check("echo-args.sh", RunScope::PerFile);

        let mut findings = Vec::new();
        run_per_file_script(
            "TECH-WRITING-STYLE",
            &check,
            &repo,
            &spex,
            &[&file],
            &mut findings,
        );

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].message, "README.md");
    }

    #[test]
    #[serial(script_exec)]
    fn script_emits_one_finding_per_location_line() {
        let workspace = tempdir().expect("workspace");
        let repo = workspace.path().join("api");
        let spex = workspace.path().join("spex");
        std::fs::create_dir_all(spex.join("scripts/TECH-WRITING-STYLE")).expect("spex scripts");
        std::fs::create_dir_all(&repo).expect("repo");
        let script = spex.join("scripts/TECH-WRITING-STYLE/locations.sh");
        // Emits two contract lines (one with a location, one file-less) plus a
        // blank line that must be ignored.
        std::fs::write(
            &script,
            "#!/bin/sh\nprintf 'docs/intro.md:40\\tmatches forbidden writing pattern\\n'\nprintf '\\n'\nprintf 'a stray diagnostic\\n'\nexit 1\n",
        )
        .expect("write script");
        make_executable(&script);

        let check = script_check("locations.sh", RunScope::PerRepo);

        let mut findings = Vec::new();
        run_per_repo_script("TECH-WRITING-STYLE", &check, &repo, &spex, &mut findings);

        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].location.as_deref(), Some("docs/intro.md:40"));
        assert_eq!(findings[0].message, "matches forbidden writing pattern");
        assert!(findings[1].location.is_none());
        assert_eq!(findings[1].message, "a stray diagnostic");
    }

    #[test]
    fn script_reports_config_error_when_file_missing() {
        let workspace = tempdir().expect("workspace");
        let repo = workspace.path().join("api");
        let spex = workspace.path().join("spex");
        std::fs::create_dir_all(&repo).expect("repo");
        std::fs::create_dir_all(&spex).expect("spex");

        let check = script_check("sample.sh", RunScope::PerRepo);

        let mut findings = Vec::new();
        run_per_repo_script(
            "TECH-COMMIT-MESSAGE-FORMAT",
            &check,
            &repo,
            &spex,
            &mut findings,
        );

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Error);
        assert!(findings[0].message.contains("not found"));
    }

    #[test]
    fn script_reports_config_error_when_unset() {
        let workspace = tempdir().expect("workspace");
        let repo = workspace.path().join("api");
        let spex = workspace.path().join("spex");
        std::fs::create_dir_all(&repo).expect("repo");
        std::fs::create_dir_all(&spex).expect("spex");

        let check = check(CheckKind::Script, None);

        let mut findings = Vec::new();
        run_per_file_script(
            "TECH-WRITING-STYLE",
            &check,
            &repo,
            &spex,
            &[],
            &mut findings,
        );

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Error);
        assert!(findings[0].message.contains("requires `script`"));
    }

    #[test]
    fn scope_files_applies_include_and_exclude() {
        let dir = tempdir().expect("temp dir");
        let rs = scoped(&dir, "src/main.rs", "");
        let ts = scoped(&dir, "src/app.ts", "");
        let test = scoped(&dir, "src/main_test.rs", "");
        let files = vec![rs, ts, test];

        let mut check = check(CheckKind::Regexp, Some("x"));
        check.include = vec!["**/*.rs".to_string()];
        check.exclude = vec!["**/*_test.rs".to_string()];

        let scoped = scope_files(&check, &files).expect("scoped");
        let names: Vec<String> = scoped.iter().map(|file| rel_str(file)).collect();
        assert_eq!(names, vec!["src/main.rs".to_string()]);
    }

    fn tech_spec(id: &str, checks: Vec<Check>) -> TechSpec {
        TechSpec {
            id: id.to_string(),
            title: String::new(),
            description: String::new(),
            acceptance_criteria: Vec::new(),
            status: crate::spec::Status::default(),
            include_for_all_repos: false,
            include_for_spex: None,
            checks,
            examples: std::collections::BTreeMap::new(),
        }
    }

    fn repository(path: &str, tech_specs: Vec<&str>) -> Repository {
        Repository {
            name: "api".to_string(),
            short_name: "api".to_string(),
            path: path.to_string(),
            contracts: Vec::new(),
            tech_specs: tech_specs.into_iter().map(str::to_string).collect(),
            checks_additional: Vec::new(),
            checks_override: Vec::new(),
            checks_preexisting_errors: Vec::new(),
        }
    }

    fn check_override(spec: &str, name: &str) -> CheckOverride {
        CheckOverride {
            spec: spec.to_string(),
            name: name.to_string(),
            disable: false,
            description: None,
            kind: None,
            pattern: None,
            negate: None,
            script: None,
            run: None,
            include: None,
            exclude: None,
            severity: None,
        }
    }

    fn named_check(name: &str, kind: CheckKind, pattern: Option<&str>) -> Check {
        let mut check = check(kind, pattern);
        check.name = name.to_string();
        check
    }

    #[test]
    #[serial(script_exec)]
    fn override_patches_severity_and_reuses_spex_script() {
        let workspace = tempdir().expect("workspace");
        let spex = workspace.path().join("spex");
        let repo = workspace.path().join("api");
        std::fs::create_dir_all(&repo).expect("repo");
        std::fs::create_dir_all(spex.join("scripts/TECH-X")).expect("spex scripts");
        let script = spex.join("scripts/TECH-X/foo.sh");
        std::fs::write(&script, "#!/bin/sh\necho spex\nexit 1\n").expect("script");
        make_executable(&script);

        let mut foo = script_check("foo.sh", RunScope::PerRepo);
        foo.name = "foo".to_string();
        let specs = HashMap::from([("TECH-X".to_string(), tech_spec("TECH-X", vec![foo]))]);

        let mut repository = repository("../api", vec!["TECH-X"]);
        let mut over = check_override("TECH-X", "foo");
        over.severity = Some(Severity::Warn);
        repository.checks_override = vec![over];

        let mut findings = Vec::new();
        check_repo(
            &repository,
            &spex,
            &specs,
            &[],
            &Log::new(false),
            &mut findings,
            &mut Vec::new(),
        )
        .expect("check");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Warn);
        assert_eq!(findings[0].message, "spex");
        assert_eq!(findings[0].spec_id, "TECH-X");
    }

    #[test]
    #[serial(script_exec)]
    fn override_sets_script_resolves_from_spex() {
        let workspace = tempdir().expect("workspace");
        let spex = workspace.path().join("spex");
        let repo = workspace.path().join("api");
        std::fs::create_dir_all(&repo).expect("repo");
        std::fs::create_dir_all(spex.join("scripts/TECH-X")).expect("spex scripts");
        let inherited = spex.join("scripts/TECH-X/foo.sh");
        std::fs::write(&inherited, "#!/bin/sh\necho inherited\nexit 1\n").expect("inherited");
        make_executable(&inherited);
        let custom = spex.join("scripts/TECH-X/custom.sh");
        std::fs::write(&custom, "#!/bin/sh\necho custom\nexit 1\n").expect("custom");
        make_executable(&custom);

        let mut foo = script_check("foo.sh", RunScope::PerRepo);
        foo.name = "foo".to_string();
        let specs = HashMap::from([("TECH-X".to_string(), tech_spec("TECH-X", vec![foo]))]);

        let mut repository = repository("../api", vec!["TECH-X"]);
        let mut over = check_override("TECH-X", "foo");
        over.script = Some("custom.sh".to_string());
        repository.checks_override = vec![over];

        let mut findings = Vec::new();
        check_repo(
            &repository,
            &spex,
            &specs,
            &[],
            &Log::new(false),
            &mut findings,
            &mut Vec::new(),
        )
        .expect("check");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].message, "custom");
    }

    #[test]
    fn orphan_override_reports_error() {
        let workspace = tempdir().expect("workspace");
        let spex = workspace.path().join("spex");
        let repo = workspace.path().join("api");
        std::fs::create_dir_all(&repo).expect("repo");
        std::fs::create_dir_all(&spex).expect("spex");

        let specs = HashMap::from([("TECH-X".to_string(), tech_spec("TECH-X", vec![]))]);

        let mut repository = repository("../api", vec!["TECH-X"]);
        repository.checks_override = vec![check_override("TECH-X", "missing")];

        let mut findings = Vec::new();
        check_repo(
            &repository,
            &spex,
            &specs,
            &[],
            &Log::new(false),
            &mut findings,
            &mut Vec::new(),
        )
        .expect("check");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Error);
        assert!(findings[0].message.contains("not an inherited check"));
    }

    #[test]
    fn disable_override_skips_check() {
        let workspace = tempdir().expect("workspace");
        let spex = workspace.path().join("spex");
        let repo = workspace.path().join("api");
        std::fs::create_dir_all(&repo).expect("repo");
        std::fs::create_dir_all(spex.join("scripts/TECH-X")).expect("spex scripts");
        let script = spex.join("scripts/TECH-X/foo.sh");
        std::fs::write(&script, "#!/bin/sh\necho spex\nexit 1\n").expect("script");
        make_executable(&script);

        let mut foo = script_check("foo.sh", RunScope::PerRepo);
        foo.name = "foo".to_string();
        let specs = HashMap::from([("TECH-X".to_string(), tech_spec("TECH-X", vec![foo]))]);

        let mut repository = repository("../api", vec!["TECH-X"]);
        let mut over = check_override("TECH-X", "foo");
        over.disable = true;
        repository.checks_override = vec![over];

        let mut findings = Vec::new();
        check_repo(
            &repository,
            &spex,
            &specs,
            &[],
            &Log::new(false),
            &mut findings,
            &mut Vec::new(),
        )
        .expect("check");

        assert!(findings.is_empty());
    }

    #[test]
    fn disable_override_orphan_reports_error() {
        let workspace = tempdir().expect("workspace");
        let spex = workspace.path().join("spex");
        let repo = workspace.path().join("api");
        std::fs::create_dir_all(&repo).expect("repo");
        std::fs::create_dir_all(&spex).expect("spex");

        let specs = HashMap::from([("TECH-X".to_string(), tech_spec("TECH-X", vec![]))]);

        let mut repository = repository("../api", vec!["TECH-X"]);
        let mut over = check_override("TECH-X", "missing");
        over.disable = true;
        repository.checks_override = vec![over];

        let mut findings = Vec::new();
        check_repo(
            &repository,
            &spex,
            &specs,
            &[],
            &Log::new(false),
            &mut findings,
            &mut Vec::new(),
        )
        .expect("check");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Error);
        assert!(findings[0].message.contains("not an inherited check"));
    }

    #[test]
    fn additional_spec_not_in_tech_specs_reports_error() {
        let workspace = tempdir().expect("workspace");
        let spex = workspace.path().join("spex");
        let repo = workspace.path().join("api");
        std::fs::create_dir_all(&repo).expect("repo");
        std::fs::create_dir_all(&spex).expect("spex");

        let specs = HashMap::from([("TECH-X".to_string(), tech_spec("TECH-X", vec![]))]);

        let mut repository = repository("../api", vec!["TECH-X"]);
        repository.checks_additional = vec![RepoCheck {
            spec: "TECH-Y".to_string(),
            check: named_check("no-todo", CheckKind::Regexp, Some("TODO")),
        }];

        let mut findings = Vec::new();
        check_repo(
            &repository,
            &spex,
            &specs,
            &[],
            &Log::new(false),
            &mut findings,
            &mut Vec::new(),
        )
        .expect("check");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Error);
        assert!(
            findings[0]
                .message
                .contains("not in this repository's tech_specs")
        );
    }

    #[test]
    fn additional_collision_runs_both() {
        let workspace = tempdir().expect("workspace");
        let spex = workspace.path().join("spex");
        let repo = workspace.path().join("api");
        std::fs::create_dir_all(&repo).expect("repo");
        std::fs::create_dir_all(&spex).expect("spex");
        std::fs::write(repo.join("notes.md"), "TODO: fix\n").expect("file");

        let specs = HashMap::from([(
            "TECH-X".to_string(),
            tech_spec(
                "TECH-X",
                vec![named_check("no-todo", CheckKind::Regexp, Some("TODO"))],
            ),
        )]);

        let mut repository = repository("../api", vec!["TECH-X"]);
        repository.checks_additional = vec![RepoCheck {
            spec: "TECH-X".to_string(),
            check: named_check("no-todo", CheckKind::Regexp, Some("TODO")),
        }];

        let mut findings = Vec::new();
        check_repo(
            &repository,
            &spex,
            &specs,
            &[],
            &Log::new(false),
            &mut findings,
            &mut Vec::new(),
        )
        .expect("check");

        assert_eq!(findings.len(), 2);
        assert!(findings.iter().all(|finding| finding.check == "no-todo"));
    }

    #[test]
    #[serial(script_exec)]
    fn additional_script_resolves_from_spex_base() {
        let workspace = tempdir().expect("workspace");
        let spex = workspace.path().join("spex");
        let repo = workspace.path().join("api");
        std::fs::create_dir_all(spex.join("scripts/TECH-X")).expect("spex scripts");
        std::fs::create_dir_all(&repo).expect("repo");
        let script = spex.join("scripts/TECH-X/added.sh");
        std::fs::write(&script, "#!/bin/sh\necho added\nexit 1\n").expect("script");
        make_executable(&script);

        let specs = HashMap::from([("TECH-X".to_string(), tech_spec("TECH-X", vec![]))]);

        let mut added = script_check("added.sh", RunScope::PerRepo);
        added.name = "added".to_string();
        let mut repository = repository("../api", vec!["TECH-X"]);
        repository.checks_additional = vec![RepoCheck {
            spec: "TECH-X".to_string(),
            check: added,
        }];

        let mut findings = Vec::new();
        check_repo(
            &repository,
            &spex,
            &specs,
            &[],
            &Log::new(false),
            &mut findings,
            &mut Vec::new(),
        )
        .expect("check");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].message, "added");
    }

    #[test]
    #[serial(script_exec)]
    fn workspace_check_runs_once_across_repos() {
        let workspace = tempdir().expect("workspace");
        let spex = workspace.path().join("spex");
        let api = workspace.path().join("api");
        let web = workspace.path().join("web");
        std::fs::create_dir_all(spex.join("scripts/TECH-X")).expect("spex scripts");
        std::fs::create_dir_all(&api).expect("api");
        std::fs::create_dir_all(&web).expect("web");

        let script = spex.join("scripts/TECH-X/list.sh");
        // Fails if handed any file arguments, records each invocation, and emits
        // its own contract findings: a workspace script gets no files and must
        // discover its own paths.
        std::fs::write(
            &script,
            "#!/bin/sh\n[ \"$#\" -eq 0 ] || { echo \"unexpected args: $*\"; exit 1; }\necho run >> runs.log\nprintf 'api/a.txt\\tflagged\\n'\nprintf 'web/b.txt\\tflagged\\n'\nexit 1\n",
        )
        .expect("script");
        make_executable(&script);

        let mut listing = script_check("list.sh", RunScope::Workspace);
        listing.name = "list".to_string();
        let specs = HashMap::from([("TECH-X".to_string(), tech_spec("TECH-X", vec![listing]))]);

        let mut collected = Vec::new();
        let mut findings = Vec::new();
        check_repo(
            &repository("../api", vec!["TECH-X"]),
            &spex,
            &specs,
            &[],
            &Log::new(false),
            &mut findings,
            &mut collected,
        )
        .expect("api");
        check_repo(
            &repository("../web", vec!["TECH-X"]),
            &spex,
            &specs,
            &[],
            &Log::new(false),
            &mut findings,
            &mut collected,
        )
        .expect("web");

        // The workspace check is deferred, so neither repository reports it, and
        // the two repositories collapse into a single pending check.
        assert!(findings.is_empty());
        assert_eq!(collected.len(), 1);

        run_workspace_checks(&spex, &collected, &Log::new(false), &mut findings);

        // It ran exactly once, despite both repositories inheriting it.
        let runs = std::fs::read_to_string(spex.join("runs.log")).expect("runs.log");
        assert_eq!(runs.lines().count(), 1);

        // The script's own contract lines become findings.
        assert_eq!(findings.len(), 2);
        assert!(
            findings
                .iter()
                .all(|finding| finding.spec_id == "TECH-X" && finding.check == "list")
        );
        let locations: Vec<&str> = findings
            .iter()
            .filter_map(|finding| finding.location.as_deref())
            .collect();
        assert!(locations.contains(&"api/a.txt"), "got: {locations:?}");
        assert!(locations.contains(&"web/b.txt"), "got: {locations:?}");
    }

    #[test]
    fn select_repos_with_files_picks_owning_and_named_repos() {
        let dir = tempdir().expect("dir");
        let spex = dir.path();
        for (repo, file) in [("api", "a.txt"), ("web", "b.txt"), ("cli", "c.txt")] {
            std::fs::create_dir_all(spex.join(repo)).expect("repo dir");
            std::fs::write(spex.join(repo).join(file), "").expect("file");
        }

        let mut api = repository("api", vec![]);
        api.name = "api".to_string();
        let mut web = repository("web", vec![]);
        web.name = "web".to_string();
        let mut cli = repository("cli", vec![]);
        cli.name = "cli".to_string();
        let config = Config {
            project_name: String::new(),
            cli_version: String::new(),
            repositories: vec![api, web, cli],
        };

        let files = vec!["a.txt".to_string(), "b.txt".to_string()];

        // Files in api and web select exactly their owning repositories.
        let selected = select_repos(&config, spex, &[], &files).expect("select");
        let names: Vec<&str> = selected.iter().map(|repo| repo.name.as_str()).collect();
        assert_eq!(names, vec!["api", "web"]);

        // --repo cli unions in even though no provided file lives there.
        let selected = select_repos(&config, spex, &["cli".to_string()], &files).expect("select");
        let names: Vec<&str> = selected.iter().map(|repo| repo.name.as_str()).collect();
        assert_eq!(names, vec!["api", "web", "cli"]);

        // An unknown --repo name still errors.
        assert!(select_repos(&config, spex, &["ghost".to_string()], &files).is_err());
    }

    fn finding(severity: Severity, check: &str, location: Option<&str>) -> Finding {
        Finding {
            severity,
            spec_id: "TECH-X".to_string(),
            check: check.to_string(),
            location: location.map(str::to_string),
            message: "boom".to_string(),
        }
    }

    fn preexisting(check: &str, files: Vec<&str>) -> PreexistingError {
        PreexistingError {
            spec: "TECH-X".to_string(),
            check: check.to_string(),
            files: files.into_iter().map(str::to_string).collect(),
        }
    }

    #[test]
    fn normalize_location_drops_trailing_line_number() {
        assert_eq!(normalize_location("src/main.rs:42"), "src/main.rs");
        assert_eq!(normalize_location("src/main.rs"), "src/main.rs");
        // A non-numeric suffix is part of the path, not a line number.
        assert_eq!(normalize_location("a:b/c.rs"), "a:b/c.rs");
    }

    #[test]
    fn preexisting_empty_files_suppresses_whole_check() {
        let baseline = vec![preexisting("no-cyrillic", vec![])];
        assert!(is_preexisting(
            &finding(Severity::Error, "no-cyrillic", Some("README.md:1")),
            &baseline,
        ));
        assert!(is_preexisting(
            &finding(Severity::Error, "no-cyrillic", None),
            &baseline,
        ));
    }

    #[test]
    fn preexisting_matches_listed_file_only() {
        let baseline = vec![preexisting("no-cyrillic", vec!["docs/old.md"])];
        // Listed file, normalized from `file:line`, is suppressed.
        assert!(is_preexisting(
            &finding(Severity::Error, "no-cyrillic", Some("docs/old.md:3")),
            &baseline,
        ));
        // A new error in another file still surfaces.
        assert!(!is_preexisting(
            &finding(Severity::Error, "no-cyrillic", Some("docs/new.md:3")),
            &baseline,
        ));
    }

    #[test]
    fn preexisting_never_suppresses_warnings_or_other_checks() {
        let baseline = vec![preexisting("no-cyrillic", vec![])];
        // Warnings are never a baseline concern.
        assert!(!is_preexisting(
            &finding(Severity::Warn, "no-cyrillic", Some("README.md:1")),
            &baseline,
        ));
        // A different check name does not match.
        assert!(!is_preexisting(
            &finding(Severity::Error, "writing", Some("README.md:1")),
            &baseline,
        ));
    }

    fn ac(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| item.to_string()).collect()
    }

    fn biz_spec(id: &str, blocked_by: Vec<&str>) -> BizSpec {
        BizSpec {
            id: id.to_string(),
            title: String::new(),
            description: String::new(),
            acceptance_criteria: Vec::new(),
            blocked_by: blocked_by.into_iter().map(str::to_string).collect(),
            status: Status::Planned,
            coverage_ignore: Vec::new(),
        }
    }

    #[test]
    fn status_finding_flags_fully_checked_unimplemented_spec() {
        let criteria = ac(&["[x] MUST a", "[x] SHOULD b"]);
        let result = status_finding("BIZ-A", "biz-specs", &criteria, Status::Planned);
        let finding = result.expect("expected a status finding");
        assert_eq!(finding.severity, Severity::Warn);
        assert_eq!(finding.check, "status");
        assert_eq!(finding.location.as_deref(), Some("biz-specs/BIZ-A.yaml"));
    }

    #[test]
    fn status_finding_is_silent_when_consistent_or_incomplete() {
        let full = ac(&["[x] MUST a", "[x] MUST b"]);
        // Fully checked and already implemented: nothing to flag.
        assert!(status_finding("BIZ-A", "biz-specs", &full, Status::Active).is_none());
        assert!(status_finding("BIZ-A", "biz-specs", &full, Status::Deprecated).is_none());
        // A pending box means the spec isn't done, so any status is allowed.
        let partial = ac(&["[x] MUST a", "[ ] MUST b"]);
        assert!(status_finding("BIZ-A", "biz-specs", &partial, Status::Planned).is_none());
        // No criteria at all: nothing to infer.
        assert!(status_finding("BIZ-A", "biz-specs", &[], Status::Planned).is_none());
    }

    #[test]
    fn validate_blocked_by_flags_missing_and_active_references() {
        let status_by_id = HashMap::from([
            ("BIZ-DONE", Status::Active),
            ("BIZ-PENDING", Status::Planned),
            ("TECH-PENDING", Status::Blocked),
        ]);
        let spec = biz_spec(
            "BIZ-A",
            vec!["BIZ-MISSING", "BIZ-DONE", "BIZ-PENDING", "TECH-PENDING"],
        );

        let mut findings = Vec::new();
        validate_blocked_by(&spec, &status_by_id, &mut findings);

        assert_eq!(findings.len(), 2);

        let missing = &findings[0];
        assert_eq!(missing.severity, Severity::Error);
        assert_eq!(missing.check, "blocked-by");
        assert!(missing.message.contains("BIZ-MISSING"));
        assert!(missing.message.contains("non-existing spec"));

        let active = &findings[1];
        assert_eq!(active.severity, Severity::Warn);
        assert!(active.message.contains("BIZ-DONE"));
        assert!(active.message.contains("already active"));
    }
}
