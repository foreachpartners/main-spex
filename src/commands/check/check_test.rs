#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::config::Repository;
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
        std::fs::create_dir_all(repo.join("scripts/TECH-COMMIT-MESSAGE-FORMAT"))
            .expect("repo scripts");
        std::fs::create_dir_all(&spex).expect("spex");
        let script = repo.join("scripts/TECH-COMMIT-MESSAGE-FORMAT/sample.sh");
        std::fs::write(&script, "#!/bin/sh\necho boom\nexit 1\n").expect("write script");
        make_executable(&script);

        let mut check = script_check("sample.sh", RunScope::PerRepo);
        check.severity = Severity::Warn;

        let mut findings = Vec::new();
        run_script(
            "TECH-COMMIT-MESSAGE-FORMAT",
            &check,
            &repo,
            &repo,
            &[],
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
        std::fs::create_dir_all(repo.join("scripts/TECH-WRITING-STYLE")).expect("repo scripts");
        std::fs::create_dir_all(&spex).expect("spex");
        let script = repo.join("scripts/TECH-WRITING-STYLE/echo-args.sh");
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
        run_script(
            "TECH-WRITING-STYLE",
            &check,
            &repo,
            &repo,
            &[&file],
            &mut findings,
        );

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].message, "README.md");
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
        run_script(
            "TECH-COMMIT-MESSAGE-FORMAT",
            &check,
            &repo,
            &repo,
            &[],
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
        run_script(
            "TECH-WRITING-STYLE",
            &check,
            &repo,
            &repo,
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
            include_for_all_repos: false,
            include_for_spex: None,
            checks,
        }
    }

    fn repository(path: &str, tech_specs: Vec<&str>) -> Repository {
        Repository {
            name: "api".to_string(),
            path: path.to_string(),
            tech_specs: tech_specs.into_iter().map(str::to_string).collect(),
            checks_additional: Vec::new(),
            checks_override: Vec::new(),
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
    fn override_sets_repo_script() {
        let workspace = tempdir().expect("workspace");
        let spex = workspace.path().join("spex");
        let repo = workspace.path().join("api");
        std::fs::create_dir_all(spex.join("scripts/TECH-X")).expect("spex scripts");
        std::fs::create_dir_all(repo.join("scripts/TECH-X")).expect("repo scripts");
        let spex_script = spex.join("scripts/TECH-X/foo.sh");
        std::fs::write(&spex_script, "#!/bin/sh\necho spex\nexit 1\n").expect("spex script");
        make_executable(&spex_script);
        let repo_script = repo.join("scripts/TECH-X/foo.sh");
        std::fs::write(&repo_script, "#!/bin/sh\necho repo\nexit 1\n").expect("repo script");
        make_executable(&repo_script);

        let mut foo = script_check("foo.sh", RunScope::PerRepo);
        foo.name = "foo".to_string();
        let specs = HashMap::from([("TECH-X".to_string(), tech_spec("TECH-X", vec![foo]))]);

        let mut repository = repository("../api", vec!["TECH-X"]);
        let mut over = check_override("TECH-X", "foo");
        over.script = Some("foo.sh".to_string());
        repository.checks_override = vec![over];

        let mut findings = Vec::new();
        check_repo(
            &repository,
            &spex,
            &specs,
            &[],
            &mut findings,
            &mut Vec::new(),
        )
        .expect("check");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].message, "repo");
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
            &mut findings,
            &mut Vec::new(),
        )
        .expect("check");

        assert_eq!(findings.len(), 2);
        assert!(findings.iter().all(|finding| finding.check == "no-todo"));
    }

    #[test]
    #[serial(script_exec)]
    fn additional_script_resolves_from_repo_base() {
        let workspace = tempdir().expect("workspace");
        let spex = workspace.path().join("spex");
        let repo = workspace.path().join("api");
        std::fs::create_dir_all(repo.join("scripts/TECH-X")).expect("repo scripts");
        std::fs::create_dir_all(&spex).expect("spex");
        let script = repo.join("scripts/TECH-X/repo-only.sh");
        std::fs::write(&script, "#!/bin/sh\necho repo-only\nexit 1\n").expect("script");
        make_executable(&script);

        let specs = HashMap::from([("TECH-X".to_string(), tech_spec("TECH-X", vec![]))]);

        let mut added = script_check("repo-only.sh", RunScope::PerRepo);
        added.name = "repo-only".to_string();
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
            &mut findings,
            &mut Vec::new(),
        )
        .expect("check");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].message, "repo-only");
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
        std::fs::write(api.join("a.txt"), "").expect("a");
        std::fs::write(web.join("b.txt"), "").expect("b");

        let script = spex.join("scripts/TECH-X/list.sh");
        std::fs::write(
            &script,
            "#!/bin/sh\nfor f in \"$@\"; do echo \"$f\"; done\nexit 1\n",
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
            &mut findings,
            &mut collected,
        )
        .expect("api");
        check_repo(
            &repository("../web", vec!["TECH-X"]),
            &spex,
            &specs,
            &[],
            &mut findings,
            &mut collected,
        )
        .expect("web");

        // The workspace check is deferred, so neither repository reports it.
        assert!(findings.is_empty());
        // Both repositories merge into a single pending check.
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].files.len(), 2);

        run_workspace_checks(&spex, &collected, &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].spec_id, "TECH-X");
        assert_eq!(findings[0].check, "list");
        let message = &findings[0].message;
        assert!(message.contains("../api/a.txt"), "got: {message}");
        assert!(message.contains("../web/b.txt"), "got: {message}");
    }

    #[test]
    fn workspace_check_respects_exclude() {
        let workspace = tempdir().expect("workspace");
        let spex = workspace.path().join("spex");
        let api = workspace.path().join("api");
        std::fs::create_dir_all(&spex).expect("spex");
        std::fs::create_dir_all(&api).expect("api");
        std::fs::write(api.join("keep.txt"), "").expect("keep");
        std::fs::write(api.join("skip.md"), "").expect("skip");

        let mut listing = script_check("list.sh", RunScope::Workspace);
        listing.name = "list".to_string();
        listing.exclude = vec!["**/*.md".to_string()];
        let specs = HashMap::from([("TECH-X".to_string(), tech_spec("TECH-X", vec![listing]))]);

        let mut collected = Vec::new();
        let mut findings = Vec::new();
        check_repo(
            &repository("../api", vec!["TECH-X"]),
            &spex,
            &specs,
            &[],
            &mut findings,
            &mut collected,
        )
        .expect("api");

        assert_eq!(collected.len(), 1);
        let files: Vec<_> = collected[0]
            .files
            .iter()
            .map(|file| file.to_string_lossy().into_owned())
            .collect();
        assert_eq!(files, vec!["../api/keep.txt".to_string()]);
    }
}
