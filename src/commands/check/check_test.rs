#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::commands::check::finding::Location;
    use crate::commands::check::runners::{build_globset, resolve_script};
    use crate::rule::{RULES_DIR, Severity};
    use crate::support::testing;
    use serial_test::serial;
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

    fn rule_in(spex_dir: &Path, id: &str) -> Rule {
        Rule {
            path: spex_dir.join(RULES_DIR).join(format!("{id}.yaml")),
            ..testing::rule(id)
        }
    }

    fn run_in(
        repo_path: &Path,
        spex_dir: &Path,
        rule_id: &str,
        check: &Check,
        files: &[ScopedFile],
    ) -> Vec<Finding> {
        let ctx = RepoCtx {
            repo_path,
            spex_dir,
            candidates: files,
        };
        ctx.run_check(&rule_in(spex_dir, rule_id), check)
    }

    fn run_scoped(
        dir: &TempDir,
        rule_id: &str,
        check: &Check,
        files: Vec<ScopedFile>,
    ) -> Vec<Finding> {
        run_in(dir.path(), dir.path(), rule_id, check, &files)
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
        let names: Vec<&Path> = resolved.iter().map(|file| file.rel.as_path()).collect();

        assert_eq!(
            names,
            vec![Path::new("src/api/main.rs"), Path::new("README.md")]
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

        let findings = run_scoped(&dir, "secret-handling", &check, vec![file]);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Error);
        assert_eq!(
            findings[0].location,
            Some(Location::at_line("src/main.rs", 1))
        );
    }

    #[test]
    fn regexp_negate_reports_when_absent() {
        let dir = tempdir().expect("temp dir");
        let file = scoped(&dir, "src/main.rs", "fn main() {}\n");
        let mut check = regexp_check("Copyright");
        set_negate(&mut check, true);

        let findings = run_scoped(&dir, "license", &check, vec![file]);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].location, Some(Location::in_file("src/main.rs")));
    }

    #[test]
    fn glob_forbidden_reports_matching_paths() {
        let dir = tempdir().expect("temp dir");
        let bak = scoped(&dir, "notes.bak", "");
        let src = scoped(&dir, "src/main.rs", "");
        let check = glob_check("**/*.bak");

        let findings = run_scoped(&dir, "secret-handling", &check, vec![bak, src]);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].location, Some(Location::in_file("notes.bak")));
    }

    #[test]
    fn glob_negate_reports_when_required_absent() {
        let dir = tempdir().expect("temp dir");
        let src = scoped(&dir, "src/main.rs", "");
        let mut check = glob_check("**/LICENSE");
        set_negate(&mut check, true);

        let findings = run_scoped(&dir, "license", &check, vec![src]);

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

        let findings = run_scoped(&dir, "secret-handling", &check, vec![file]);

        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].location,
            Some(Location::at_line("src/main.ts", 2))
        );
    }

    #[test]
    fn ast_grep_infers_language_from_extension() {
        let dir = tempdir().expect("temp dir");
        let ts = scoped(&dir, "src/main.ts", "console.log(1);\n");
        let py = scoped(&dir, "src/main.py", "print(1)\n");
        let check = ast_grep_check(Some("console.log($$$ARGS)"));

        let findings = run_scoped(&dir, "secret-handling", &check, vec![ts, py]);

        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].location,
            Some(Location::at_line("src/main.ts", 1))
        );
    }

    #[test]
    fn ast_grep_reports_hits_in_file_order_across_languages() {
        let dir = tempdir().expect("temp dir");
        let files = vec![
            scoped(&dir, "src/a.ts", "trace(1);\n"),
            scoped(&dir, "src/b.js", "trace(1);\n"),
            scoped(&dir, "src/c.tsx", "trace(1);\n"),
            scoped(&dir, "src/d.rs", "fn run() { trace(1); }\n"),
            scoped(&dir, "src/e.py", "trace(1)\n"),
        ];
        let check = ast_grep_check(Some("trace($$$ARGS)"));

        let findings = run_scoped(&dir, "tracing", &check, files);

        let locations: Vec<_> = findings
            .iter()
            .map(|finding| finding.location.clone())
            .collect();
        assert_eq!(
            locations,
            [
                Some(Location::at_line("src/a.ts", 1)),
                Some(Location::at_line("src/b.js", 1)),
                Some(Location::at_line("src/c.tsx", 1)),
                Some(Location::at_line("src/d.rs", 1)),
                Some(Location::at_line("src/e.py", 1)),
            ]
        );
    }

    #[test]
    fn ast_grep_negate_reports_when_absent() {
        let dir = tempdir().expect("temp dir");
        let file = scoped(&dir, "src/main.ts", "console.error(1);\n");
        let mut check = ast_grep_check(Some("console.log($$$ARGS)"));
        set_negate(&mut check, true);

        let findings = run_scoped(&dir, "secret-handling", &check, vec![file]);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].location, Some(Location::in_file("src/main.ts")));
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

        let findings = run_scoped(&dir, "secret-handling", &check, vec![file]);

        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].location,
            Some(Location::at_line("src/main.ts", 2))
        );
    }

    #[test]
    fn ast_grep_reports_config_error_for_invalid_language() {
        let dir = tempdir().expect("temp dir");
        let file = scoped(&dir, "src/main.ts", "console.log(1);\n");
        let mut check = ast_grep_check(Some("console.log($$$ARGS)"));
        set_language(&mut check, "not-a-real-language");

        let findings = run_scoped(&dir, "secret-handling", &check, vec![file]);

        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].source.baseline_key(),
            Some(("secret-handling", "sample"))
        );
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

        let findings = run_scoped(&dir, "secret-handling", &check, vec![file]);

        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].location,
            Some(Location::at_line("src/main.ts", 2))
        );
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

        let findings = run_scoped(&dir, "secret-handling", &check, vec![file]);

        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].location,
            Some(Location::at_line("src/main.rs", 2))
        );
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

        let findings = run_scoped(&dir, "testing", &check, vec![file]);

        let mut lines: Vec<_> = findings
            .iter()
            .map(|finding| finding.location.clone().expect("location").line)
            .collect();
        lines.sort();
        assert_eq!(lines, vec![Some(1), Some(2)]);
    }

    #[test]
    fn resolve_script_joins_script_dir_and_reference() {
        let workspace = tempdir().expect("workspace");
        let script_dir = workspace.path().join("spex/rules/x");
        std::fs::create_dir_all(&script_dir).expect("rule script dir");
        std::fs::write(script_dir.join("lint.sh"), "").expect("rule script");

        let resolved = resolve_script(&script_dir, "lint.sh").expect("resolved");
        assert_eq!(resolved, script_dir.join("lint.sh"));
    }

    #[test]
    fn resolve_script_returns_none_when_missing() {
        let workspace = tempdir().expect("workspace");
        let script_dir = workspace.path().join("spex/rules/x");
        std::fs::create_dir_all(&script_dir).expect("rule script dir");

        assert!(resolve_script(&script_dir, "lint.sh").is_none());
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
        std::fs::create_dir_all(spex.join("rules/commit-message-format")).expect("rule scripts");
        std::fs::create_dir_all(&repo).expect("repo");
        let script = spex.join("rules/commit-message-format/sample.sh");
        std::fs::write(&script, "#!/bin/sh\necho boom\nexit 1\n").expect("write script");
        make_executable(&script);

        let mut check = script_check("sample.sh", RunScope::PerRepo);
        check.severity = Severity::Warn;

        let findings = run_in(&repo, &spex, "commit-message-format", &check, &[]);

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
        std::fs::create_dir_all(spex.join("rules/writing-style")).expect("rule scripts");
        std::fs::create_dir_all(&repo).expect("repo");
        let script = spex.join("rules/writing-style/echo-args.sh");
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

        let findings = run_in(&repo, &spex, "writing-style", &check, &[file]);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].message, "README.md");
    }

    #[test]
    #[serial(script_exec)]
    fn script_emits_one_finding_per_location_line() {
        let workspace = tempdir().expect("workspace");
        let repo = workspace.path().join("api");
        let spex = workspace.path().join("spex");
        std::fs::create_dir_all(spex.join("rules/writing-style")).expect("rule scripts");
        std::fs::create_dir_all(&repo).expect("repo");
        let script = spex.join("rules/writing-style/locations.sh");
        // Emits two contract lines (one with a location, one file-less) plus a
        // blank line that must be ignored.
        std::fs::write(
            &script,
            "#!/bin/sh\nprintf 'docs/intro.md:40\\tmatches forbidden writing pattern\\n'\nprintf '\\n'\nprintf 'a stray diagnostic\\n'\nexit 1\n",
        )
        .expect("write script");
        make_executable(&script);

        let check = script_check("locations.sh", RunScope::PerRepo);

        let findings = run_in(&repo, &spex, "writing-style", &check, &[]);

        assert_eq!(findings.len(), 2);
        assert_eq!(
            findings[0].location,
            Some(Location::at_line("docs/intro.md", 40))
        );
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

        let findings = run_in(&repo, &spex, "commit-message-format", &check, &[]);

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
        let names: Vec<&Path> = scoped.iter().map(|file| file.rel.as_path()).collect();
        assert_eq!(names, vec![Path::new("src/main.rs")]);
    }

    fn rule(id: &str, checks: Vec<Check>) -> Rule {
        Rule {
            checks,
            ..testing::rule(id)
        }
    }

    fn repository(spex_dir: &Path, path: &str, rules: Vec<&str>) -> Repository {
        Repository {
            rules: rules.into_iter().map(testing::rule_id).collect(),
            path: spex_dir.join(path),
            ..testing::repository("api", path)
        }
    }

    fn check_name(finding: &Finding) -> &str {
        finding.source.baseline_key().expect("check source").1
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

        let rules = [rule("x", vec![negated_glob.clone(), non_negated])];

        let repository = repository(&spex, "../api", vec!["x"]);

        // Without --files: negated check fires because dprint.json is absent.
        let findings = check_repo(
            CheckRepoArgs {
                repo: &repository,
                spex_dir: &spex,
                rules: &rules,
                known_ids: &HashMap::new(),
                files: &[],
            },
            &mut Vec::new(),
        )
        .expect("check");
        assert_eq!(findings.len(), 2);
        assert!(
            findings
                .iter()
                .any(|f| check_name(f) == "dprint-config-present")
        );
        assert!(findings.iter().any(|f| check_name(f) == "no-rs-files"));

        // With --files: negated check is skipped, non-negated still fires.
        let findings = check_repo(
            CheckRepoArgs {
                repo: &repository,
                spex_dir: &spex,
                rules: &rules,
                known_ids: &HashMap::new(),
                files: &["src/main.rs".to_string()],
            },
            &mut Vec::new(),
        )
        .expect("check");
        assert_eq!(findings.len(), 1);
        assert!(
            findings
                .iter()
                .all(|f| check_name(f) != "dprint-config-present")
        );
        assert!(findings.iter().any(|f| check_name(f) == "no-rs-files"));
    }

    #[test]
    #[serial(script_exec)]
    fn workspace_check_runs_once_across_repos() {
        let workspace = tempdir().expect("workspace");
        let spex = workspace.path().join("spex");
        let api = workspace.path().join("api");
        let web = workspace.path().join("web");
        std::fs::create_dir_all(spex.join("rules/x")).expect("rule scripts");
        std::fs::create_dir_all(&api).expect("api");
        std::fs::create_dir_all(&web).expect("web");

        let script = spex.join("rules/x/list.sh");
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
        let rules = [Rule {
            checks: vec![listing],
            ..rule_in(&spex, "x")
        }];

        let mut collected = Vec::new();
        let mut repo_findings = check_repo(
            CheckRepoArgs {
                repo: &repository(&spex, "../api", vec!["x"]),
                spex_dir: &spex,
                rules: &rules,
                known_ids: &HashMap::new(),
                files: &[],
            },
            &mut collected,
        )
        .expect("api");
        repo_findings.extend(
            check_repo(
                CheckRepoArgs {
                    repo: &repository(&spex, "../web", vec!["x"]),
                    spex_dir: &spex,
                    rules: &rules,
                    known_ids: &HashMap::new(),
                    files: &[],
                },
                &mut collected,
            )
            .expect("web"),
        );

        // The workspace check is deferred, so neither repository reports it, and
        // the two repositories collapse into a single pending check.
        assert!(repo_findings.is_empty());
        assert_eq!(collected.len(), 1);

        let findings = run_workspace_checks(&spex, &collected);

        // It ran exactly once, despite both repositories inheriting it.
        let runs = std::fs::read_to_string(spex.join("runs.log")).expect("runs.log");
        assert_eq!(runs.lines().count(), 1);

        // The script's own contract lines become findings.
        assert_eq!(findings.len(), 2);
        assert!(
            findings
                .iter()
                .all(|finding| finding.source.baseline_key() == Some(("x", "list")))
        );
        let locations: Vec<Location> = findings
            .iter()
            .filter_map(|finding| finding.location.clone())
            .collect();
        assert!(
            locations.contains(&Location::in_file("api/a.txt")),
            "got: {locations:?}"
        );
        assert!(
            locations.contains(&Location::in_file("web/b.txt")),
            "got: {locations:?}"
        );
    }
}
