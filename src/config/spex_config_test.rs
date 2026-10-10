#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::support::testing;
    use indoc::indoc;
    use tempfile::tempdir;

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

    #[test]
    fn parses_coverage_ignore_entries_naming_a_spec() {
        let config: SpexConfig = serde_yaml_ng::from_str(indoc! {"
            project_name: demo
            cli_version: 0.0.0
            coverage_ignore:
              - spec_id: place-order
                layer: contract
              - spec_id: place-order
                layer: impl
                ac: rejects-empty-cart
        "})
        .expect("parse config");

        let json = serde_json::to_value(&config).expect("serialize");

        assert_eq!(
            json["coverage_ignore"],
            serde_json::json!([
                { "spec_id": "place-order", "layer": "contract" },
                { "spec_id": "place-order", "layer": "impl", "ac": "rejects-empty-cart" },
            ])
        );
    }

    #[test]
    fn serialization_keeps_an_absent_coverage_ignore_as_an_empty_list() {
        let config: SpexConfig = serde_yaml_ng::from_str(indoc! {"
            project_name: demo
            cli_version: 0.0.0
        "})
        .expect("parse config");

        let json = serde_json::to_value(&config).expect("serialize");

        assert_eq!(json["coverage_ignore"], serde_json::json!([]));
    }
}
