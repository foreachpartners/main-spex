#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::support::testing;
    use indoc::indoc;
    use tempfile::tempdir;

    const SPEX_REPO_DIR: &str = "demo-spex";

    fn project() -> tempfile::TempDir {
        let project = tempdir().expect("project");
        let spex = project.path().join(SPEX_REPO_DIR);
        std::fs::create_dir_all(project.path().join("demo-api/src")).expect("package");
        std::fs::create_dir_all(&spex).expect("spex dir");
        std::fs::write(spex.join(SPEX_CONFIG_FILE), "project_name: demo\n").expect("config");
        project
    }

    #[test]
    fn finds_from_project_root() {
        let project = project();
        let found = SpexConfig::find_file(project.path()).expect("found");
        assert_eq!(
            found,
            project.path().join(SPEX_REPO_DIR).join(SPEX_CONFIG_FILE)
        );
    }

    #[test]
    fn finds_from_spex_dir() {
        let project = project();
        let spex = project.path().join(SPEX_REPO_DIR);
        let found = SpexConfig::find_file(&spex).expect("found");
        assert_eq!(found, spex.join(SPEX_CONFIG_FILE));
    }

    #[test]
    fn finds_from_nested_package_subdirectory() {
        let project = project();
        let found = SpexConfig::find_file(&project.path().join("demo-api/src")).expect("found");
        assert_eq!(
            found,
            project.path().join(SPEX_REPO_DIR).join(SPEX_CONFIG_FILE)
        );
    }

    #[test]
    fn errors_when_not_in_a_project() {
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
            layout: multi-repo
            agent_root: .
            description: A demo project.
            rules_unused:
              - graphql-conventions
            packages:
              - package_name: demo-api
                package_short_name: api
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
        assert_eq!(json["packages"][0]["description"], "REST API.");
        assert_eq!(
            json["packages"][0]["contracts"][0]["description"],
            "OpenAPI contract."
        );
    }

    #[test]
    fn serialization_keeps_the_prd_path_and_omits_an_absent_one() {
        let config = |prd_line: &str| -> serde_json::Value {
            let config: SpexConfig = serde_yaml_ng::from_str(&format!(
                "project_name: demo\ncli_version: 0.0.0\nlayout: multi-repo\nagent_root: .\n{prd_line}"
            ))
            .expect("parse config");
            serde_json::to_value(&config).expect("serialize")
        };

        assert_eq!(config("prd: docs/prd.md\n")["prd"], "docs/prd.md");
        assert!(config("").get("prd").is_none());
    }

    #[test]
    fn serialization_omits_absent_descriptions_but_keeps_empty_lists() {
        let config: SpexConfig = serde_yaml_ng::from_str(indoc! {"
            project_name: demo
            cli_version: 0.0.0
            layout: multi-repo
            agent_root: .
            packages:
              - package_name: demo-api
                package_short_name: api
                path: ../demo-api
        "})
        .expect("parse config");

        let json = serde_json::to_value(&config).expect("serialize");

        assert!(json.get("description").is_none());
        assert_eq!(json["rules_unused"], serde_json::json!([]));
        assert!(json["packages"][0].get("description").is_none());
        assert_eq!(json["packages"][0]["contracts"], serde_json::json!([]));
    }

    #[test]
    fn parses_package_coverage_ignore_entries_naming_a_spec() {
        let config: SpexConfig = serde_yaml_ng::from_str(indoc! {"
            project_name: demo
            cli_version: 0.0.0
            layout: multi-repo
            agent_root: .
            packages:
              - package_name: demo-api
                package_short_name: api
                path: ../demo-api
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
            json["packages"][0]["coverage_ignore"],
            serde_json::json!([
                { "spec_id": "place-order", "layer": "contract" },
                { "spec_id": "place-order", "layer": "impl", "ac": "rejects-empty-cart" },
            ])
        );
    }

    #[test]
    fn parses_a_package_exclude_and_keeps_an_absent_one_as_an_empty_list() {
        let config: SpexConfig = serde_yaml_ng::from_str(indoc! {"
            project_name: demo
            cli_version: 0.0.0
            layout: multi-repo
            agent_root: .
            packages:
              - package_name: demo-api
                package_short_name: api
                path: ../demo-api
                exclude:
                  - vendor
                  - \"**/*.generated.ts\"
              - package_name: demo-web
                package_short_name: web
                path: ../demo-web
        "})
        .expect("parse config");

        let json = serde_json::to_value(&config).expect("serialize");

        assert_eq!(
            json["packages"][0]["exclude"],
            serde_json::json!(["vendor", "**/*.generated.ts"])
        );
        assert_eq!(json["packages"][1]["exclude"], serde_json::json!([]));
    }

    #[test]
    fn rejects_a_config_without_a_layout() {
        let parsed = serde_yaml_ng::from_str::<SpexConfig>(indoc! {"
            project_name: demo
            cli_version: 0.0.0
            agent_root: .
        "});

        assert!(parsed.is_err());
    }

    #[test]
    fn serialization_keeps_an_absent_package_coverage_ignore_as_an_empty_list() {
        let config: SpexConfig = serde_yaml_ng::from_str(indoc! {"
            project_name: demo
            cli_version: 0.0.0
            layout: multi-repo
            agent_root: .
            packages:
              - package_name: demo-api
                package_short_name: api
                path: ../demo-api
        "})
        .expect("parse config");

        let json = serde_json::to_value(&config).expect("serialize");

        assert_eq!(
            json["packages"][0]["coverage_ignore"],
            serde_json::json!([])
        );
    }
}
