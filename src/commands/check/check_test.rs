#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::commands::check::finding::Location;
    use crate::commands::check::fixtures::{check, write_script};
    use crate::rule::{RULES_DIR, Severity};
    use crate::support::testing;
    use serial_test::serial;
    use tempfile::tempdir;

    fn glob_check(name: &str, pattern: &str, negate: bool) -> Check {
        Check {
            name: name.to_string(),
            ..check(CheckKind::Glob {
                pattern: pattern.to_string(),
                negate,
            })
        }
    }

    fn script_check(name: &str, script: &str, run: RunScope) -> Check {
        Check {
            name: name.to_string(),
            ..check(CheckKind::Script {
                script: script.to_string(),
                run,
            })
        }
    }

    fn rule_in(spex_dir: &Path, id: &str) -> Rule {
        Rule {
            path: spex_dir.join(RULES_DIR).join(format!("{id}.yaml")),
            ..testing::rule(id)
        }
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

    #[test]
    fn run_check_reports_config_error_when_script_missing() {
        let workspace = tempdir().expect("workspace");
        let repo = workspace.path().join("api");
        let spex = workspace.path().join("spex");
        std::fs::create_dir_all(&repo).expect("repo");
        std::fs::create_dir_all(&spex).expect("spex");

        let check = script_check("sample", "sample.sh", RunScope::PerRepo);
        let ctx = RepoCtx {
            repo_path: &repo,
            spex_dir: &spex,
            candidates: &[],
        };

        let findings = ctx.run_check(&rule_in(&spex, "commit-message-format"), &check);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Error);
        assert!(findings[0].message.contains("not found"));
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

        let negated_glob = glob_check("dprint-config-present", "dprint.json", true);
        let non_negated = glob_check("no-rs-files", "**/*.rs", false);

        let rules = [rule("x", vec![negated_glob.clone(), non_negated])];

        let repository = repository(&spex, "../api", vec!["x"]);

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
        std::fs::create_dir_all(&api).expect("api");
        std::fs::create_dir_all(&web).expect("web");

        let script = spex.join("rules/x/list.sh");
        write_script(
            &script,
            "#!/bin/sh\n[ \"$#\" -eq 0 ] || { echo \"unexpected args: $*\"; exit 1; }\necho run >> runs.log\nprintf 'api/a.txt\\tflagged\\n'\nprintf 'web/b.txt\\tflagged\\n'\nexit 1\n",
        );

        let listing = script_check("list", "list.sh", RunScope::Workspace);
        let rules = [Rule {
            checks: vec![listing],
            ..rule_in(&spex, "x")
        }];

        let api_repo = repository(&spex, "../api", vec!["x"]);
        let web_repo = repository(&spex, "../web", vec!["x"]);
        let known_ids = HashMap::new();

        let mut collected = Vec::new();
        let mut repo_findings = check_repo(
            CheckRepoArgs {
                repo: &api_repo,
                spex_dir: &spex,
                rules: &rules,
                known_ids: &known_ids,
                files: &[],
            },
            &mut collected,
        )
        .expect("api");
        repo_findings.extend(
            check_repo(
                CheckRepoArgs {
                    repo: &web_repo,
                    spex_dir: &spex,
                    rules: &rules,
                    known_ids: &known_ids,
                    files: &[],
                },
                &mut collected,
            )
            .expect("web"),
        );

        assert!(repo_findings.is_empty());
        assert_eq!(collected.len(), 1);

        let findings = run_workspace_checks(&spex, &collected);

        let runs = std::fs::read_to_string(spex.join("runs.log")).expect("runs.log");
        assert_eq!(runs.lines().count(), 1);

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
