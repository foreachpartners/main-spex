#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::config::{Config, PreexistingError, Repository};
    use crate::rule::{Check, CheckKind, Rule, RunScope, Severity};
    use crate::spec::{
        AcceptanceCriterion, Commitment, ContractLevel, CoverageIgnore, CoverageLayer,
    };
    use serial_test::serial;
    use std::collections::HashMap;
    use tempfile::{TempDir, tempdir};

    fn check(kind: CheckKind) -> Check {
        Check {
            name: "sample".to_string(),
            description: None,
            kind,
            include: Vec::new(),
            exclude: Vec::new(),
            severity: Severity::Error,
        }
    }

    fn regexp_check(pattern: &str) -> Check {
        check(CheckKind::Regexp {
            pattern: pattern.to_string(),
            negate: false,
        })
    }

    fn glob_check(pattern: &str) -> Check {
        check(CheckKind::Glob {
            pattern: pattern.to_string(),
            negate: false,
        })
    }

    fn ast_grep_check(pattern: Option<&str>) -> Check {
        check(CheckKind::AstGrep {
            pattern: pattern.map(str::to_string),
            rule: None,
            language: None,
            negate: false,
        })
    }

    fn script_check(reference: &str, run: RunScope) -> Check {
        check(CheckKind::Script {
            script: reference.to_string(),
            run,
        })
    }

    fn set_negate(check: &mut Check, value: bool) {
        match &mut check.kind {
            CheckKind::Regexp { negate, .. }
            | CheckKind::Glob { negate, .. }
            | CheckKind::AstGrep { negate, .. } => *negate = value,
            CheckKind::Script { .. } => panic!("script checks have no `negate`"),
        }
    }

    fn set_rule(check: &mut Check, value: serde_yaml_ng::Value) {
        let CheckKind::AstGrep { rule, .. } = &mut check.kind else {
            panic!("only ast-grep checks have `rule`")
        };
        *rule = Some(value);
    }

    fn set_language(check: &mut Check, value: &str) {
        let CheckKind::AstGrep { language, .. } = &mut check.kind else {
            panic!("only ast-grep checks have `language`")
        };
        *language = Some(value.to_string());
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
        let resolved = ScopedFile::provided(dir.path(), &files);
        let names: Vec<String> = resolved.iter().map(ScopedFile::rel_str).collect();

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
        let check = regexp_check("api_key\\s*=\\s*\"[^\"]{8,}\"");

        let mut findings = Vec::new();
        run_regexp("secret-handling", &check, &[&file], &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Error);
        assert_eq!(findings[0].location.as_deref(), Some("src/main.rs:1"));
    }

    #[test]
    fn regexp_negate_reports_when_absent() {
        let dir = tempdir().expect("temp dir");
        let file = scoped(&dir, "src/main.rs", "fn main() {}\n");
        let mut check = regexp_check("Copyright");
        set_negate(&mut check, true);

        let mut findings = Vec::new();
        run_regexp("LICENSE", &check, &[&file], &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].location.as_deref(), Some("src/main.rs"));
    }

    #[test]
    fn glob_forbidden_reports_matching_paths() {
        let dir = tempdir().expect("temp dir");
        let bak = scoped(&dir, "notes.bak", "");
        let src = scoped(&dir, "src/main.rs", "");
        let check = glob_check("**/*.bak");

        let mut findings = Vec::new();
        run_glob("secret-handling", &check, &[&bak, &src], &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].location.as_deref(), Some("notes.bak"));
    }

    #[test]
    fn glob_negate_reports_when_required_absent() {
        let dir = tempdir().expect("temp dir");
        let src = scoped(&dir, "src/main.rs", "");
        let mut check = glob_check("**/LICENSE");
        set_negate(&mut check, true);

        let mut findings = Vec::new();
        run_glob("LICENSE", &check, &[&src], &mut findings);

        assert_eq!(findings.len(), 1);
        assert!(findings[0].location.is_none());
    }

    #[test]
    fn ast_grep_pattern_matches_structurally_not_textually() {
        let dir = tempdir().expect("temp dir");
        let file = scoped(
            &dir,
            "src/main.ts",
            "// console.log(\"not real\")\nconsole.log(\"hi\");\nconsole.error(\"bye\");\n",
        );
        let check = ast_grep_check(Some("console.log($$$ARGS)"));

        let mut findings = Vec::new();
        run_ast_grep("secret-handling", &check, &[&file], &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].location.as_deref(), Some("src/main.ts:2"));
    }

    #[test]
    fn ast_grep_infers_language_from_extension() {
        let dir = tempdir().expect("temp dir");
        let ts = scoped(&dir, "src/main.ts", "console.log(1);\n");
        let py = scoped(&dir, "src/main.py", "print(1)\n");
        let check = ast_grep_check(Some("console.log($$$ARGS)"));

        let mut findings = Vec::new();
        run_ast_grep("secret-handling", &check, &[&ts, &py], &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].location.as_deref(), Some("src/main.ts:1"));
    }

    #[test]
    fn ast_grep_negate_reports_when_absent() {
        let dir = tempdir().expect("temp dir");
        let file = scoped(&dir, "src/main.ts", "console.error(1);\n");
        let mut check = ast_grep_check(Some("console.log($$$ARGS)"));
        set_negate(&mut check, true);

        let mut findings = Vec::new();
        run_ast_grep("secret-handling", &check, &[&file], &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].location.as_deref(), Some("src/main.ts"));
    }

    #[test]
    fn ast_grep_rule_supports_constraints_on_metavariables() {
        let dir = tempdir().expect("temp dir");
        let file = scoped(
            &dir,
            "src/main.ts",
            "console.log(userId);\nconsole.log(apiToken);\n",
        );
        let mut check = ast_grep_check(None);
        set_rule(
            &mut check,
            serde_yaml_ng::from_str(
                r#"
rule:
  pattern: console.log($ARG)
constraints:
  ARG:
    regex: "(?i)token|secret|password"
"#,
            )
            .expect("valid rule yaml"),
        );

        let mut findings = Vec::new();
        run_ast_grep("secret-handling", &check, &[&file], &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].location.as_deref(), Some("src/main.ts:2"));
    }

    #[test]
    fn ast_grep_reports_config_error_for_invalid_language() {
        let dir = tempdir().expect("temp dir");
        let file = scoped(&dir, "src/main.ts", "console.log(1);\n");
        let mut check = ast_grep_check(Some("console.log($$$ARGS)"));
        set_language(&mut check, "not-a-real-language");

        let mut findings = Vec::new();
        run_ast_grep("secret-handling", &check, &[&file], &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].check, "sample");
        assert!(findings[0].location.is_none());
    }

    #[test]
    fn ast_grep_rule_constraint_on_multi_metavariable() {
        let dir = tempdir().expect("temp dir");
        let file = scoped(
            &dir,
            "src/main.ts",
            "console.log(userId);\nconsole.log(\"token=\", apiToken);\n",
        );
        let mut check = ast_grep_check(None);
        set_rule(
            &mut check,
            serde_yaml_ng::from_str(
                r#"
rule:
  all:
    - pattern: console.$METHOD($$$ARGS)
    - has:
        stopBy: end
        regex: "(?i)(token|secret|password|credential|api[_-]?key)"
"#,
            )
            .expect("valid rule yaml"),
        );

        let mut findings = Vec::new();
        run_ast_grep("secret-handling", &check, &[&file], &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].location.as_deref(), Some("src/main.ts:2"));
    }

    #[test]
    fn ast_grep_rule_supports_any_composite_with_constraints() {
        let dir = tempdir().expect("temp dir");
        let file = scoped(&dir, "src/main.rs", "info!(user_id);\nwarn!(api_token);\n");
        let mut check = ast_grep_check(None);
        set_language(&mut check, "rust");
        set_rule(
            &mut check,
            serde_yaml_ng::from_str(
                r#"
rule:
  all:
    - any:
        - pattern: info!($$$ARGS)
        - pattern: warn!($$$ARGS)
        - pattern: error!($$$ARGS)
    - has:
        stopBy: end
        regex: "(?i)(token|secret|password|credential|api[_-]?key)"
"#,
            )
            .expect("valid rule yaml"),
        );

        let mut findings = Vec::new();
        run_ast_grep("secret-handling", &check, &[&file], &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].location.as_deref(), Some("src/main.rs:2"));
    }

    #[test]
    fn ast_grep_rule_constraint_only_applies_when_var_is_bound() {
        let dir = tempdir().expect("temp dir");
        let file = scoped(
            &dir,
            "src/main.test.ts",
            "fit('runs', () => {});\ndescribe.only('suite', () => {});\narr.only('nope', () => {});\n",
        );
        let mut check = ast_grep_check(None);
        set_rule(
            &mut check,
            serde_yaml_ng::from_str(
                r#"
rule:
  any:
    - pattern: fit($$$ARGS)
    - pattern: fdescribe($$$ARGS)
    - pattern: $OBJ.only($$$ARGS)
constraints:
  OBJ:
    regex: "^(describe|it|test|context)$"
"#,
            )
            .expect("valid rule yaml"),
        );

        let mut findings = Vec::new();
        run_ast_grep("testing", &check, &[&file], &mut findings);

        let mut locations: Vec<_> = findings
            .iter()
            .map(|finding| finding.location.clone().unwrap())
            .collect();
        locations.sort();
        assert_eq!(
            locations,
            vec![
                "src/main.test.ts:1".to_string(),
                "src/main.test.ts:2".to_string(),
            ]
        );
    }

    #[test]
    fn resolve_script_joins_scripts_and_spec_dir() {
        let workspace = tempdir().expect("workspace");
        let spex = workspace.path().join("spex");
        std::fs::create_dir_all(spex.join("scripts/x")).expect("spex scripts");
        std::fs::write(spex.join("scripts/x/lint.sh"), "").expect("spex script");

        let resolved = resolve_script("lint.sh", &spex, "x").expect("resolved");
        assert_eq!(resolved, spex.join("scripts/x/lint.sh"));
    }

    #[test]
    fn resolve_script_returns_none_when_missing() {
        let workspace = tempdir().expect("workspace");
        let spex = workspace.path().join("spex");
        std::fs::create_dir_all(&spex).expect("spex");

        assert!(resolve_script("lint.sh", &spex, "X").is_none());
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
        std::fs::create_dir_all(spex.join("scripts/commit-message-format")).expect("spex scripts");
        std::fs::create_dir_all(&repo).expect("repo");
        let script = spex.join("scripts/commit-message-format/sample.sh");
        std::fs::write(&script, "#!/bin/sh\necho boom\nexit 1\n").expect("write script");
        make_executable(&script);

        let mut check = script_check("sample.sh", RunScope::PerRepo);
        check.severity = Severity::Warn;

        let mut findings = Vec::new();
        run_per_repo_script("commit-message-format", &check, &repo, &spex, &mut findings);

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
        std::fs::create_dir_all(spex.join("scripts/writing-style")).expect("spex scripts");
        std::fs::create_dir_all(&repo).expect("repo");
        let script = spex.join("scripts/writing-style/echo-args.sh");
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
            "writing-style",
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
        std::fs::create_dir_all(spex.join("scripts/writing-style")).expect("spex scripts");
        std::fs::create_dir_all(&repo).expect("repo");
        let script = spex.join("scripts/writing-style/locations.sh");
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
        run_per_repo_script("writing-style", &check, &repo, &spex, &mut findings);

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
        run_per_repo_script("commit-message-format", &check, &repo, &spex, &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Error);
        assert!(findings[0].message.contains("not found"));
    }

    #[test]
    fn scope_files_applies_include_and_exclude() {
        let dir = tempdir().expect("temp dir");
        let rs = scoped(&dir, "src/main.rs", "");
        let ts = scoped(&dir, "src/app.ts", "");
        let test = scoped(&dir, "src/main_test.rs", "");
        let files = vec![rs, ts, test];

        let mut check = regexp_check("x");
        check.include = vec!["**/*.rs".to_string()];
        check.exclude = vec!["**/*_test.rs".to_string()];

        let scoped = scope_files(&check, &files).expect("scoped");
        let names: Vec<String> = scoped.iter().map(|file| file.rel_str()).collect();
        assert_eq!(names, vec!["src/main.rs".to_string()]);
    }

    fn rule(id: &str, checks: Vec<Check>) -> Rule {
        Rule {
            id: id.to_string(),
            path: format!("{id}.yaml"),
            acceptance_criteria: Vec::new(),
            checks,
        }
    }

    fn repository(path: &str, rules: Vec<&str>) -> Repository {
        Repository {
            repository_name: "api".to_string(),
            repository_short_name: "api".to_string(),
            dir: path.to_string(),
            contracts: Vec::new(),
            rules: rules.into_iter().map(str::to_string).collect(),
            ignored_errors: Vec::new(),
        }
    }

    fn named_glob_check(name: &str, pattern: &str) -> Check {
        let mut check = glob_check(pattern);
        check.name = name.to_string();
        check
    }

    #[test]
    fn check_repo_skips_negated_checks_when_files_provided() {
        let workspace = tempdir().expect("workspace");
        let spex = workspace.path().join("spex");
        let repo = workspace.path().join("api");
        std::fs::create_dir_all(&spex).expect("spex");
        std::fs::create_dir_all(&repo).expect("repo");
        std::fs::create_dir_all(repo.join("src")).expect("src");
        std::fs::write(repo.join("src/main.rs"), "fn main() {}").expect("file");

        // A negated glob check: "dprint.json must exist"
        let mut negated_glob = named_glob_check("dprint-config-present", "dprint.json");
        set_negate(&mut negated_glob, true);
        // A non-negated glob check: "*.rs files are forbidden"
        let non_negated = named_glob_check("no-rs-files", "**/*.rs");

        let rules = HashMap::from([(
            "X".to_string(),
            rule("X", vec![negated_glob.clone(), non_negated]),
        )]);

        let repository = repository("../api", vec!["X"]);

        // Without --files: negated check fires because dprint.json is absent.
        let mut findings = Vec::new();
        check_repo(
            CheckRepoArgs {
                repo: &repository,
                spex_dir: &spex,
                rules: &rules,
                known_ids: &HashMap::new(),
                files: &[],
            },
            &mut findings,
            &mut Vec::new(),
        )
        .expect("check");
        assert_eq!(findings.len(), 2);
        assert!(findings.iter().any(|f| f.check == "dprint-config-present"));
        assert!(findings.iter().any(|f| f.check == "no-rs-files"));

        // With --files: negated check is skipped, non-negated still fires.
        let mut findings = Vec::new();
        check_repo(
            CheckRepoArgs {
                repo: &repository,
                spex_dir: &spex,
                rules: &rules,
                known_ids: &HashMap::new(),
                files: &["src/main.rs".to_string()],
            },
            &mut findings,
            &mut Vec::new(),
        )
        .expect("check");
        assert_eq!(findings.len(), 1);
        assert!(findings.iter().all(|f| f.check != "dprint-config-present"));
        assert!(findings.iter().any(|f| f.check == "no-rs-files"));
    }

    #[test]
    #[serial(script_exec)]
    fn workspace_check_runs_once_across_repos() {
        let workspace = tempdir().expect("workspace");
        let spex = workspace.path().join("spex");
        let api = workspace.path().join("api");
        let web = workspace.path().join("web");
        std::fs::create_dir_all(spex.join("scripts/x")).expect("spex scripts");
        std::fs::create_dir_all(&api).expect("api");
        std::fs::create_dir_all(&web).expect("web");

        let script = spex.join("scripts/x/list.sh");
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
        let rules = HashMap::from([("x".to_string(), rule("x", vec![listing]))]);

        let mut collected = Vec::new();
        let mut findings = Vec::new();
        check_repo(
            CheckRepoArgs {
                repo: &repository("../api", vec!["x"]),
                spex_dir: &spex,
                rules: &rules,
                known_ids: &HashMap::new(),
                files: &[],
            },
            &mut findings,
            &mut collected,
        )
        .expect("api");
        check_repo(
            CheckRepoArgs {
                repo: &repository("../web", vec!["x"]),
                spex_dir: &spex,
                rules: &rules,
                known_ids: &HashMap::new(),
                files: &[],
            },
            &mut findings,
            &mut collected,
        )
        .expect("web");

        // The workspace check is deferred, so neither repository reports it, and
        // the two repositories collapse into a single pending check.
        assert!(findings.is_empty());
        assert_eq!(collected.len(), 1);

        run_workspace_checks(&spex, &collected, &mut findings);

        // It ran exactly once, despite both repositories inheriting it.
        let runs = std::fs::read_to_string(spex.join("runs.log")).expect("runs.log");
        assert_eq!(runs.lines().count(), 1);

        // The script's own contract lines become findings.
        assert_eq!(findings.len(), 2);
        assert!(
            findings
                .iter()
                .all(|finding| finding.spec_id == "x" && finding.check == "list")
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
        api.repository_name = "api".to_string();
        let mut web = repository("web", vec![]);
        web.repository_name = "web".to_string();
        let mut cli = repository("cli", vec![]);
        cli.repository_name = "cli".to_string();
        let config = Config {
            project_name: String::new(),
            cli_version: String::new(),
            repositories: vec![api, web, cli],
        };

        let files = vec!["a.txt".to_string(), "b.txt".to_string()];

        // Files in api and web select exactly their owning repositories.
        let selected = select_repos(&config, spex, &[], &files).expect("select");
        let names: Vec<&str> = selected
            .iter()
            .map(|repo| repo.repository_name.as_str())
            .collect();
        assert_eq!(names, vec!["api", "web"]);

        // --repo cli unions in even though no provided file lives there.
        let selected = select_repos(&config, spex, &["cli".to_string()], &files).expect("select");
        let names: Vec<&str> = selected
            .iter()
            .map(|repo| repo.repository_name.as_str())
            .collect();
        assert_eq!(names, vec!["api", "web", "cli"]);

        // An unknown --repo name still errors.
        assert!(select_repos(&config, spex, &["ghost".to_string()], &files).is_err());

        // A --files entry that doesn't exist in any repository errors.
        let missing = vec!["missing.txt".to_string()];
        assert!(select_repos(&config, spex, &[], &missing).is_err());
    }

    fn finding(severity: Severity, check: &str, location: Option<&str>) -> Finding {
        Finding {
            severity,
            spec_id: "X".to_string(),
            check: check.to_string(),
            location: location.map(str::to_string),
            message: "boom".to_string(),
        }
    }

    fn preexisting(check: &str, files: Vec<&str>) -> PreexistingError {
        PreexistingError {
            rule_id: "X".to_string(),
            check: check.to_string(),
            files: files.into_iter().map(str::to_string).collect(),
        }
    }

    #[test]
    fn normalize_location_drops_trailing_line_number() {
        assert_eq!(Finding::normalize_location("src/main.rs:42"), "src/main.rs");
        assert_eq!(Finding::normalize_location("src/main.rs"), "src/main.rs");
        // A non-numeric suffix is part of the path, not a line number.
        assert_eq!(Finding::normalize_location("a:b/c.rs"), "a:b/c.rs");
    }

    #[test]
    fn preexisting_empty_files_suppresses_whole_check() {
        let baseline = vec![preexisting("no-cyrillic", vec![])];
        assert!(
            finding(Severity::Error, "no-cyrillic", Some("README.md:1")).is_preexisting(&baseline)
        );
        assert!(finding(Severity::Error, "no-cyrillic", None).is_preexisting(&baseline));
    }

    #[test]
    fn preexisting_matches_listed_file_only() {
        let baseline = vec![preexisting("no-cyrillic", vec!["docs/old.md"])];
        // Listed file, normalized from `file:line`, is suppressed.
        assert!(
            finding(Severity::Error, "no-cyrillic", Some("docs/old.md:3"))
                .is_preexisting(&baseline)
        );
        // A new error in another file still surfaces.
        assert!(
            !finding(Severity::Error, "no-cyrillic", Some("docs/new.md:3"))
                .is_preexisting(&baseline)
        );
    }

    #[test]
    fn preexisting_never_suppresses_warnings_or_other_checks() {
        let baseline = vec![preexisting("no-cyrillic", vec![])];
        // Warnings are never a baseline concern.
        assert!(
            !finding(Severity::Warn, "no-cyrillic", Some("README.md:1")).is_preexisting(&baseline)
        );
        // A different check name does not match.
        assert!(
            !finding(Severity::Error, "writing", Some("README.md:1")).is_preexisting(&baseline)
        );
    }

    fn ac(ids: &[&str]) -> Vec<AcceptanceCriterion> {
        ac_with_done(ids, &[])
    }

    // Builds acceptance criteria for the given ids, marking `not_done` ones
    // with `is_done: false` and leaving the rest at the (done) default.
    fn ac_with_done(ids: &[&str], not_done: &[&str]) -> Vec<AcceptanceCriterion> {
        ids.iter()
            .map(|id| AcceptanceCriterion {
                id: id.to_string(),
                commitment: Commitment::Must(id.to_string()),
                is_done: not_done.contains(id).then_some(false),
                line: 0,
            })
            .collect()
    }

    fn spec(id: &str, blocked_by: Vec<&str>) -> Spec {
        spec_with_criteria(id, Vec::new(), blocked_by)
    }

    fn spec_with_criteria(
        id: &str,
        acceptance_criteria: Vec<AcceptanceCriterion>,
        blocked_by: Vec<&str>,
    ) -> Spec {
        Spec {
            id: id.to_string(),
            path: format!("{id}.yaml"),
            description: String::new(),
            acceptance_criteria,
            blocked_by: blocked_by.into_iter().map(str::to_string).collect(),
            scope: Vec::new(),
            status: Status::Planned,
            contract: ContractLevel::Single,
            coverage_ignore: Vec::new(),
        }
    }

    #[test]
    fn status_finding_flags_fully_done_unimplemented_spec() {
        let criteria = ac(&["a", "b"]);
        let spec = spec_with_criteria("a", criteria, Vec::new());
        let result = status_finding(&spec);
        let finding = result.expect("expected a status finding");
        assert_eq!(finding.severity, Severity::Warn);
        assert_eq!(finding.check, "status");
        assert_eq!(finding.location.as_deref(), Some("specs/a.yaml"));
    }

    #[test]
    fn status_finding_is_silent_when_consistent_or_incomplete() {
        let full = ac(&["a", "b"]);
        // Fully done and already implemented: nothing to flag.
        let mut active = spec_with_criteria("A", full.clone(), Vec::new());
        active.status = Status::Active;
        assert!(status_finding(&active).is_none());
        let mut deprecated = spec_with_criteria("A", full, Vec::new());
        deprecated.status = Status::Deprecated;
        assert!(status_finding(&deprecated).is_none());
        // A not-done criterion means the spec isn't done, so any status is allowed.
        let partial = ac_with_done(&["a", "b"], &["b"]);
        assert!(status_finding(&spec_with_criteria("A", partial, Vec::new())).is_none());
        // No criteria at all: nothing to infer.
        assert!(status_finding(&spec_with_criteria("A", Vec::new(), Vec::new())).is_none());
    }

    #[test]
    fn active_incomplete_finding_flags_active_with_not_done_criteria() {
        let partial = ac_with_done(&["a", "b"], &["b"]);
        let mut spec = spec_with_criteria("a", partial, Vec::new());
        spec.status = Status::Active;
        let finding = active_incomplete_finding(&spec).expect("expected a finding");
        assert_eq!(finding.severity, Severity::Error);
        assert_eq!(finding.check, "status");
        assert_eq!(finding.location.as_deref(), Some("specs/a.yaml"));
        assert!(finding.message.contains("1/2"));
    }

    #[test]
    fn active_incomplete_finding_is_silent_when_complete_or_not_active() {
        let full = ac(&["a", "b"]);
        let mut active = spec_with_criteria("a", full, Vec::new());
        active.status = Status::Active;
        assert!(active_incomplete_finding(&active).is_none());

        let partial = ac_with_done(&["a", "b"], &["b"]);
        let mut in_progress = spec_with_criteria("a", partial, Vec::new());
        in_progress.status = Status::InProgress;
        assert!(active_incomplete_finding(&in_progress).is_none());
    }

    #[test]
    fn validate_blocked_by_flags_missing_and_active_references() {
        let status_by_id = HashMap::from([
            ("DONE", Status::Active),
            ("PENDING", Status::Planned),
            ("STILL-PENDING", Status::Blocked),
        ]);
        let spec = spec("A", vec!["MISSING", "DONE", "PENDING", "STILL-PENDING"]);

        let mut findings = Vec::new();
        validate_blocked_by(&spec, &status_by_id, &mut findings);

        assert_eq!(findings.len(), 2);

        let missing = &findings[0];
        assert_eq!(missing.severity, Severity::Error);
        assert_eq!(missing.check, "blocked-by");
        assert!(missing.message.contains("MISSING"));
        assert!(missing.message.contains("non-existing spec"));

        let active = &findings[1];
        assert_eq!(active.severity, Severity::Warn);
        assert!(active.message.contains("DONE"));
        assert!(active.message.contains("already active"));
    }

    #[test]
    fn validate_scope_flags_unknown_repository_short_name() {
        let mut spec = spec("A", Vec::new());
        spec.scope = vec!["api".to_string(), "unknown".to_string()];
        let repositories = vec![repository("../api", Vec::new())];

        let mut findings = Vec::new();
        validate_scope(&spec, &repositories, &mut findings);

        assert_eq!(findings.len(), 1);
        let finding = &findings[0];
        assert_eq!(finding.severity, Severity::Error);
        assert_eq!(finding.check, "scope");
        assert!(finding.message.contains("unknown"));
        assert!(finding.message.contains("repository_short_name"));
    }

    fn contract_ignore(ac: Option<&str>) -> CoverageIgnore {
        CoverageIgnore {
            layer: CoverageLayer::Contract,
            ac: ac.map(str::to_string),
        }
    }

    fn coverage_ignore_findings(spec: &Spec) -> Vec<Finding> {
        let mut findings = Vec::new();
        validate_coverage_ignore(spec, &mut findings);
        findings
    }

    #[test]
    fn coverage_ignore_whole_layer_contract_is_silent_with_contract_single() {
        let mut spec = spec("A", Vec::new());
        spec.contract = ContractLevel::Single;
        spec.coverage_ignore = vec![contract_ignore(None)];
        assert!(coverage_ignore_findings(&spec).is_empty());
    }

    #[test]
    fn coverage_ignore_whole_layer_contract_is_silent_with_for_each_ac() {
        let mut spec = spec("A", Vec::new());
        spec.contract = ContractLevel::ForEachAc;
        spec.coverage_ignore = vec![contract_ignore(None)];
        assert!(coverage_ignore_findings(&spec).is_empty());
    }

    #[test]
    fn coverage_ignore_per_ac_contract_is_silent_with_for_each_ac() {
        let mut spec = spec("A", Vec::new());
        spec.contract = ContractLevel::ForEachAc;
        spec.coverage_ignore = vec![contract_ignore(Some("ac-1"))];
        assert!(coverage_ignore_findings(&spec).is_empty());
    }

    #[test]
    fn coverage_ignore_contract_errors_with_contract_none() {
        let mut spec = spec("A", Vec::new());
        spec.contract = ContractLevel::None;
        spec.coverage_ignore = vec![contract_ignore(None), contract_ignore(Some("ac-1"))];

        let findings = coverage_ignore_findings(&spec);
        assert_eq!(findings.len(), 2);
        for finding in &findings {
            assert_eq!(finding.severity, Severity::Error);
            assert_eq!(finding.check, "coverage-ignore");
            assert_eq!(finding.location.as_deref(), Some("specs/A.yaml"));
            assert!(finding.message.contains("contract: none"));
        }
    }

    #[test]
    fn coverage_ignore_per_ac_contract_errors_with_contract_single() {
        let mut spec = spec("A", Vec::new());
        spec.contract = ContractLevel::Single;
        spec.coverage_ignore = vec![contract_ignore(Some("ac-1"))];

        let findings = coverage_ignore_findings(&spec);
        assert_eq!(findings.len(), 1);
        let finding = &findings[0];
        assert_eq!(finding.severity, Severity::Error);
        assert_eq!(finding.check, "coverage-ignore");
        assert!(finding.message.contains("contract: single"));
        assert!(finding.message.contains("contract: for-each-ac"));
    }

    #[test]
    fn coverage_ignore_other_layers_are_always_silent() {
        let mut spec = spec("A", Vec::new());
        spec.contract = ContractLevel::None;
        spec.coverage_ignore = vec![
            CoverageIgnore {
                layer: CoverageLayer::Impl,
                ac: Some("ac-1".to_string()),
            },
            CoverageIgnore {
                layer: CoverageLayer::Test,
                ac: None,
            },
        ];
        assert!(coverage_ignore_findings(&spec).is_empty());
    }
}
