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

    fn package(spex_dir: &Path, path: &str, rules: Vec<&str>) -> Package {
        Package {
            rules: rules.into_iter().map(testing::rule_id).collect(),
            path: spex_dir.join(path),
            ..testing::package("api", path)
        }
    }

    fn check_name(finding: &Finding) -> &str {
        finding.source.baseline_key().expect("check source").1
    }

    #[test]
    fn run_check_reports_config_error_when_script_missing() {
        let project = tempdir().expect("project");
        let package_dir = project.path().join("api");
        let spex = project.path().join("spex");
        std::fs::create_dir_all(&package_dir).expect("package");
        std::fs::create_dir_all(&spex).expect("spex");

        let check = script_check("sample", "sample.sh", RunScope::PerPackage);
        let ctx = PackageCtx {
            package_path: &package_dir,
            spex_dir: &spex,
            candidates: &[],
        };

        let findings = ctx.run_check(&rule_in(&spex, "commit-message-format"), &check);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Error);
        assert!(findings[0].message.contains("not found"));
    }

    #[test]
    fn check_package_skips_negated_checks_when_files_provided() {
        let project = tempdir().expect("project");
        let spex = project.path().join("spex");
        let package_dir = project.path().join("api");
        std::fs::create_dir_all(&spex).expect("spex");
        std::fs::create_dir_all(&package_dir).expect("package");
        std::fs::create_dir_all(package_dir.join("src")).expect("src");
        std::fs::write(package_dir.join("src/main.rs"), "fn main() {}").expect("file");

        let negated_glob = glob_check("dprint-config-present", "dprint.json", true);
        let non_negated = glob_check("no-rs-files", "**/*.rs", false);

        let rules = [rule("x", vec![negated_glob.clone(), non_negated])];

        let package = package(&spex, "../api", vec!["x"]);

        let findings = check_package(
            CheckPackageArgs {
                package: &package,
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

        let findings = check_package(
            CheckPackageArgs {
                package: &package,
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
    fn project_check_runs_once_across_packages() {
        let project = tempdir().expect("project");
        let spex = project.path().join("spex");
        let api = project.path().join("api");
        let web = project.path().join("web");
        std::fs::create_dir_all(&api).expect("api");
        std::fs::create_dir_all(&web).expect("web");

        let script = spex.join("rules/x/list.sh");
        write_script(
            &script,
            "#!/bin/sh\n[ \"$#\" -eq 0 ] || { echo \"unexpected args: $*\"; exit 1; }\necho run >> runs.log\nprintf 'api/a.txt\\tflagged\\n'\nprintf 'web/b.txt\\tflagged\\n'\nexit 1\n",
        );

        let listing = script_check("list", "list.sh", RunScope::Project);
        let rules = [Rule {
            checks: vec![listing],
            ..rule_in(&spex, "x")
        }];

        let api_package = package(&spex, "../api", vec!["x"]);
        let web_package = package(&spex, "../web", vec!["x"]);
        let known_ids = HashMap::new();

        let mut collected = Vec::new();
        let mut package_findings = check_package(
            CheckPackageArgs {
                package: &api_package,
                spex_dir: &spex,
                rules: &rules,
                known_ids: &known_ids,
                files: &[],
            },
            &mut collected,
        )
        .expect("api");
        package_findings.extend(
            check_package(
                CheckPackageArgs {
                    package: &web_package,
                    spex_dir: &spex,
                    rules: &rules,
                    known_ids: &known_ids,
                    files: &[],
                },
                &mut collected,
            )
            .expect("web"),
        );

        assert!(package_findings.is_empty());
        assert_eq!(collected.len(), 1);

        let findings = run_project_checks(&spex, &collected);

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
