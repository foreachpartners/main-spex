#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::support::testing;
    use indoc::indoc;
    use std::collections::HashMap;
    use tempfile::tempdir;

    // The spex repository directory is named `<project-name>-spex`, so discovery
    // must match the `-spex` suffix and the `spex.yaml` it holds, not a fixed name.
    const SPEX_REPO_DIR: &str = "demo-spex";

    fn workspace() -> tempfile::TempDir {
        let workspace = tempdir().expect("workspace");
        let spex = workspace.path().join(SPEX_REPO_DIR);
        std::fs::create_dir_all(workspace.path().join("demo-api/src")).expect("repo");
        std::fs::create_dir_all(&spex).expect("spex dir");
        std::fs::write(spex.join(SPEX_CONFIG_FILE), "project_name: demo\n").expect("config");
        workspace
    }

    #[test]
    fn finds_from_workspace_root() {
        let workspace = workspace();
        let found = SpexConfig::find_file(workspace.path()).expect("found");
        assert_eq!(
            found,
            workspace.path().join(SPEX_REPO_DIR).join(SPEX_CONFIG_FILE)
        );
    }

    #[test]
    fn finds_from_spex_dir() {
        let workspace = workspace();
        let spex = workspace.path().join(SPEX_REPO_DIR);
        let found = SpexConfig::find_file(&spex).expect("found");
        assert_eq!(found, spex.join(SPEX_CONFIG_FILE));
    }

    #[test]
    fn finds_from_nested_repo_subdirectory() {
        let workspace = workspace();
        let found = SpexConfig::find_file(&workspace.path().join("demo-api/src")).expect("found");
        assert_eq!(
            found,
            workspace.path().join(SPEX_REPO_DIR).join(SPEX_CONFIG_FILE)
        );
    }

    #[test]
    fn errors_when_not_in_a_workspace() {
        let dir = tempdir().expect("dir");
        assert!(SpexConfig::find_file(dir.path()).is_err());
    }

    #[test]
    fn ignores_child_directory_not_named_like_a_spex_repository() {
        let dir = tempdir().expect("dir");
        let templates = dir.path().join("templates");
        std::fs::create_dir_all(&templates).expect("templates dir");
        std::fs::write(templates.join(SPEX_CONFIG_FILE), "project_name: unknown\n")
            .expect("config");
        assert!(SpexConfig::find_file(dir.path()).is_err());
    }

    #[test]
    fn finds_child_directory_named_spex() {
        let dir = tempdir().expect("dir");
        let spex = dir.path().join(SPEX_DIR_NAME);
        std::fs::create_dir_all(&spex).expect("spex dir");
        std::fs::write(spex.join(SPEX_CONFIG_FILE), "project_name: demo\n").expect("config");
        assert_eq!(
            SpexConfig::find_file(dir.path()).expect("found"),
            spex.join(SPEX_CONFIG_FILE)
        );
    }

    fn config_with_version(version: &str) -> SpexConfig {
        SpexConfig {
            cli_version: version.to_string(),
            ..testing::spex_config()
        }
    }

    #[test]
    fn finds_spex_dir_directly_when_run_inside_it() {
        let dir = tempdir().expect("dir");
        std::fs::write(dir.path().join(SPEX_CONFIG_FILE), "project_name: demo\n").expect("config");
        assert_eq!(
            SpexConfig::find_file(dir.path()).expect("found"),
            dir.path().join(SPEX_CONFIG_FILE)
        );
    }

    #[test]
    fn version_warning_is_none_when_matching() {
        assert!(config_with_version(CLI_VERSION).version_warning().is_none());
    }

    #[test]
    fn version_warning_reports_mismatch() {
        let warning = config_with_version("0.0.1")
            .version_warning()
            .expect("warning");
        assert!(warning.contains("0.0.1"));
        assert!(warning.contains(CLI_VERSION));
    }

    #[test]
    fn set_cli_version_replaces_existing_line() {
        let raw = indoc! {"
            project_name: demo
            cli_version: 0.0.1
            repositories: []
        "};
        assert_eq!(
            SpexConfig::set_cli_version(raw, "0.2.0"),
            indoc! {"
                project_name: demo
                cli_version: 0.2.0
                repositories: []
            "}
        );
    }

    #[test]
    fn set_cli_version_inserts_after_project_name() {
        let raw = indoc! {"
            project_name: demo
            repositories: []
        "};
        assert_eq!(
            SpexConfig::set_cli_version(raw, "0.2.0"),
            indoc! {"
                project_name: demo
                cli_version: 0.2.0
                repositories: []
            "}
        );
    }

    #[test]
    fn set_cli_version_preserves_comments() {
        let raw = indoc! {"
            project_name: demo
            cli_version: 0.0.1
            # keep me
            repositories: []
        "};
        assert_eq!(
            SpexConfig::set_cli_version(raw, "0.2.0"),
            indoc! {"
                project_name: demo
                cli_version: 0.2.0
                # keep me
                repositories: []
            "}
        );
    }

    fn entry(rule_id: &str, check: &str, files: &[&str]) -> PreexistingEntry {
        PreexistingEntry {
            rule_id: rule_id.to_string(),
            check: check.to_string(),
            files: files.iter().map(|file| file.to_string()).collect(),
        }
    }

    const TWO_REPOS: &str = indoc! {"
        project_name: demo
        repositories:
          - repository_name: api
            path: ../api
            rules:
              - X
          - repository_name: web
            path: ../web
            rules:
              - X
    "};

    #[test]
    fn set_preexisting_errors_appends_block_with_files() {
        let by_repo = HashMap::from([(
            "api".to_string(),
            vec![entry("X", "no-cyrillic", &["README.md", "docs/intro.md"])],
        )]);
        let out = SpexConfig::set_preexisting_errors(TWO_REPOS, &by_repo);
        assert_eq!(
            out,
            indoc! {"
                project_name: demo
                repositories:
                  - repository_name: api
                    path: ../api
                    rules:
                      - X
                    ignored_errors:
                      - rule_id: X
                        check: no-cyrillic
                        files:
                          - README.md
                          - docs/intro.md
                  - repository_name: web
                    path: ../web
                    rules:
                      - X
            "}
        );
    }

    #[test]
    fn set_preexisting_errors_renders_whole_check_without_files() {
        let by_repo = HashMap::from([("web".to_string(), vec![entry("Y", "commit-format", &[])])]);
        let out = SpexConfig::set_preexisting_errors(TWO_REPOS, &by_repo);
        // A mid-document fragment keeps its 2-space indent, which indoc would strip.
        assert!(out.ends_with(
            "  - repository_name: web
    path: ../web
    rules:
      - X
    ignored_errors:
      - rule_id: Y
        check: commit-format
"
        ));
        // The api repository is out of scope, so it keeps no block.
        assert!(!out.contains("check: commit-format\n  - repository_name: web"));
        assert_eq!(out.matches("ignored_errors:").count(), 1);
    }

    #[test]
    fn set_preexisting_errors_replaces_existing_block() {
        let raw = indoc! {"
            project_name: demo
            repositories:
              - repository_name: api
                path: ../api
                ignored_errors:
                  - rule_id: X
                    check: stale
                    files:
                      - old.md
                rules:
                  - X
        "};
        let by_repo = HashMap::from([(
            "api".to_string(),
            vec![entry("X", "no-cyrillic", &["README.md"])],
        )]);
        let out = SpexConfig::set_preexisting_errors(raw, &by_repo);
        assert!(!out.contains("stale"), "old block survived:\n{out}");
        assert!(
            out.contains("check: no-cyrillic"),
            "new block missing:\n{out}"
        );
        // The block is regenerated once, not duplicated.
        assert_eq!(out.matches("ignored_errors:").count(), 1);
    }

    #[test]
    fn set_preexisting_errors_removes_stale_block_when_empty() {
        let raw = indoc! {"
            project_name: demo
            repositories:
              - repository_name: api
                path: ../api
                ignored_errors:
                  - rule_id: X
                    check: gone
        "};
        // api is in scope but has no current errors, so its block is dropped.
        let by_repo = HashMap::from([("api".to_string(), Vec::new())]);
        let out = SpexConfig::set_preexisting_errors(raw, &by_repo);
        assert_eq!(
            out,
            indoc! {"
                project_name: demo
                repositories:
                  - repository_name: api
                    path: ../api
            "}
        );
    }

    #[test]
    fn serialization_keeps_descriptions_and_rules_unused() {
        let config: SpexConfig = serde_yaml_ng::from_str(indoc! {"
            project_name: demo
            cli_version: 0.0.0
            description: A demo project.
            rules_unused:
              - graphql-conventions
            repositories:
              - repository_name: demo-api
                repository_short_name: api
                description: REST API.
                path: ../demo-api
                contracts:
                  - path: contracts/api.yaml
                    description: OpenAPI contract.
        "})
        .expect("parse config");

        let json = serde_json::to_value(&config).expect("serialize");

        assert_eq!(json["description"], "A demo project.");
        assert_eq!(json["rules_unused"][0], "graphql-conventions");
        assert_eq!(json["repositories"][0]["description"], "REST API.");
        assert_eq!(
            json["repositories"][0]["contracts"][0]["description"],
            "OpenAPI contract."
        );
    }

    #[test]
    fn serialization_omits_absent_descriptions_but_keeps_empty_lists() {
        let config: SpexConfig = serde_yaml_ng::from_str(indoc! {"
            project_name: demo
            cli_version: 0.0.0
            repositories:
              - repository_name: demo-api
                repository_short_name: api
                path: ../demo-api
        "})
        .expect("parse config");

        let json = serde_json::to_value(&config).expect("serialize");

        assert!(json.get("description").is_none());
        assert_eq!(json["rules_unused"], serde_json::json!([]));
        assert!(json["repositories"][0].get("description").is_none());
        assert_eq!(json["repositories"][0]["contracts"], serde_json::json!([]));
    }
}
