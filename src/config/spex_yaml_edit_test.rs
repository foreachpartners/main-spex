#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::config::PreexistingError;
    use indoc::indoc;
    use std::collections::HashMap;

    #[test]
    fn set_cli_version_replaces_existing_line() {
        let raw = indoc! {"
            project_name: demo
            cli_version: 0.0.1
            packages: []
        "};
        assert_eq!(
            set_cli_version(raw, "0.2.0"),
            indoc! {"
                project_name: demo
                cli_version: 0.2.0
                packages: []
            "}
        );
    }

    #[test]
    fn set_cli_version_inserts_after_project_name() {
        let raw = indoc! {"
            project_name: demo
            packages: []
        "};
        assert_eq!(
            set_cli_version(raw, "0.2.0"),
            indoc! {"
                project_name: demo
                cli_version: 0.2.0
                packages: []
            "}
        );
    }

    #[test]
    fn set_cli_version_preserves_comments() {
        let raw = indoc! {"
            project_name: demo
            cli_version: 0.0.1
            # keep me
            packages: []
        "};
        assert_eq!(
            set_cli_version(raw, "0.2.0"),
            indoc! {"
                project_name: demo
                cli_version: 0.2.0
                # keep me
                packages: []
            "}
        );
    }

    fn entry(rule_id: &str, check: &str, files: &[&str]) -> PreexistingError {
        PreexistingError {
            rule_id: rule_id.to_string(),
            check: check.to_string(),
            files: files.iter().map(|file| file.to_string()).collect(),
        }
    }

    const TWO_REPOS: &str = indoc! {"
        project_name: demo
        packages:
          - package_name: api
            path: ../api
            rules:
              - X
          - package_name: web
            path: ../web
            rules:
              - X
    "};

    #[test]
    fn set_preexisting_errors_appends_block_with_files() {
        let by_package = HashMap::from([(
            "api".to_string(),
            vec![entry("X", "no-cyrillic", &["README.md", "docs/intro.md"])],
        )]);
        let out = set_preexisting_errors(TWO_REPOS, &by_package);
        assert_eq!(
            out,
            indoc! {"
                project_name: demo
                packages:
                  - package_name: api
                    path: ../api
                    rules:
                      - X
                    ignored_errors:
                      - rule_id: X
                        check: no-cyrillic
                        files:
                          - README.md
                          - docs/intro.md
                  - package_name: web
                    path: ../web
                    rules:
                      - X
            "}
        );
    }

    #[test]
    fn set_preexisting_errors_renders_whole_check_without_files() {
        let by_package =
            HashMap::from([("web".to_string(), vec![entry("Y", "commit-format", &[])])]);
        let out = set_preexisting_errors(TWO_REPOS, &by_package);
        assert!(out.ends_with(
            "  - package_name: web
    path: ../web
    rules:
      - X
    ignored_errors:
      - rule_id: Y
        check: commit-format
"
        ));
        assert!(!out.contains("check: commit-format\n  - package_name: web"));
        assert_eq!(out.matches("ignored_errors:").count(), 1);
    }

    #[test]
    fn set_preexisting_errors_replaces_existing_block() {
        let raw = indoc! {"
            project_name: demo
            packages:
              - package_name: api
                path: ../api
                ignored_errors:
                  - rule_id: X
                    check: stale
                    files:
                      - old.md
                rules:
                  - X
        "};
        let by_package = HashMap::from([(
            "api".to_string(),
            vec![entry("X", "no-cyrillic", &["README.md"])],
        )]);
        let out = set_preexisting_errors(raw, &by_package);
        assert!(!out.contains("stale"), "old block survived:\n{out}");
        assert!(
            out.contains("check: no-cyrillic"),
            "new block missing:\n{out}"
        );
        assert_eq!(out.matches("ignored_errors:").count(), 1);
    }

    #[test]
    fn set_preexisting_errors_removes_stale_block_when_empty() {
        let raw = indoc! {"
            project_name: demo
            packages:
              - package_name: api
                path: ../api
                ignored_errors:
                  - rule_id: X
                    check: gone
        "};
        let by_package = HashMap::from([("api".to_string(), Vec::new())]);
        let out = set_preexisting_errors(raw, &by_package);
        assert_eq!(
            out,
            indoc! {"
                project_name: demo
                packages:
                  - package_name: api
                    path: ../api
            "}
        );
    }
}
