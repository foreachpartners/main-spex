#[cfg(test)]
mod tests {
    use super::super::*;
    use indoc::{formatdoc, indoc};
    use tempfile::tempdir;

    fn spec(id: &str, extra: &str) -> serde_json::Value {
        serde_json::from_str(&format!(r#"{{"id": "{id}"{extra}}}"#)).expect("valid json")
    }

    #[test]
    fn package_selected_defaults_to_true_when_empty() {
        assert!(package_selected(
            &spec("a", r#", "packages": ["api"]"#),
            &[]
        ));
    }

    #[test]
    fn package_selected_matches_short_name_in_spec_packages() {
        let api = ["api"];
        assert!(package_selected(
            &spec("a", r#", "packages": ["api"]"#),
            &api
        ));
        assert!(!package_selected(
            &spec("a", r#", "packages": ["frontend"]"#),
            &api
        ));
        assert!(!package_selected(&spec("a", ""), &api));
    }

    #[test]
    fn id_selected_defaults_to_true_when_empty() {
        assert!(id_selected(&spec("a", ""), &[]));
    }

    #[test]
    fn id_selected_filters_to_given_ids() {
        let ids = vec!["a".to_string(), "b".to_string()];
        assert!(id_selected(&spec("a", ""), &ids));
        assert!(!id_selected(&spec("c", ""), &ids));
    }

    fn write_project(dir: &std::path::Path) {
        std::fs::write(
            dir.join("spex.yaml"),
            indoc! {"
                project_name: demo
                cli_version: 0.0.0
                layout: multi-repo
                agent_root: .
                prd: docs/prd.md
                packages:
                  - package_name: demo-api
                    package_short_name: api
                    path: ../demo-api
                  - package_name: demo-frontend
                    package_short_name: frontend
                    path: ../demo-frontend
            "},
        )
        .expect("write spex.yaml");
        std::fs::create_dir_all(dir.join("specs")).expect("specs dir");
        std::fs::write(
            dir.join("specs/place-pizza-order.yaml"),
            indoc! {r#"
                id: place-pizza-order
                title: Place a pizza order
                description: Lets a customer place an order.
                acceptance_criteria:
                  - "[ ] MUST accept a cart"
                packages: [api]
            "#},
        )
        .expect("write spec");
        std::fs::write(
            dir.join("specs/list-pizza-menu.yaml"),
            indoc! {r#"
                id: list-pizza-menu
                title: List the pizza menu
                description: Lets a customer browse the menu.
                acceptance_criteria:
                  - "[ ] MUST list every active pizza"
                packages: [frontend]
            "#},
        )
        .expect("write spec");
    }

    fn project(dir: &std::path::Path) -> Spex {
        let config = crate::config::SpexConfig::load(&dir.join(crate::config::SPEX_CONFIG_FILE))
            .expect("load config");
        Spex::from_config(dir.to_path_buf(), config)
    }

    fn args(ai: bool, id: &[&str], package: &[&str]) -> SpecsArgs {
        SpecsArgs {
            ai,
            id: id.iter().map(|s| s.to_string()).collect(),
            package: package.iter().map(|s| s.to_string()).collect(),
            output: None,
        }
    }

    #[test]
    fn render_with_no_flags_prints_every_spec() {
        let dir = tempdir().expect("temp dir");
        write_project(dir.path());
        let project = project(dir.path());

        let yaml = render(&project, &args(false, &[], &[])).expect("render");

        assert_eq!(
            yaml,
            indoc! {"
                - id: list-pizza-menu
                  title: List the pizza menu
                  description: Lets a customer browse the menu.
                  acceptance_criteria:
                  - '[ ] MUST list every active pizza'
                  packages:
                  - frontend

                - id: place-pizza-order
                  title: Place a pizza order
                  description: Lets a customer place an order.
                  acceptance_criteria:
                  - '[ ] MUST accept a cart'
                  packages:
                  - api
            "}
        );
    }

    #[test]
    fn render_with_ai_flag_adds_path() {
        let dir = tempdir().expect("temp dir");
        write_project(dir.path());
        let project = project(dir.path());

        let yaml = render(&project, &args(true, &["place-pizza-order"], &[])).expect("render");

        assert_eq!(
            yaml,
            formatdoc! {"
                - id: place-pizza-order
                  path: {dir}/specs/place-pizza-order.yaml
                  title: Place a pizza order
                  description: Lets a customer place an order.
                  acceptance_criteria:
                  - '[ ] MUST accept a cart'
                  packages:
                  - api
            ", dir = dir.path().display()}
        );
    }

    #[test]
    fn render_with_single_package_returns_only_matching_specs() {
        let dir = tempdir().expect("temp dir");
        write_project(dir.path());
        let project = project(dir.path());

        let yaml = render(&project, &args(false, &[], &["frontend"])).expect("render");

        assert_eq!(
            yaml,
            indoc! {"
                - id: list-pizza-menu
                  title: List the pizza menu
                  description: Lets a customer browse the menu.
                  acceptance_criteria:
                  - '[ ] MUST list every active pizza'
                  packages:
                  - frontend
            "}
        );
    }

    #[test]
    fn render_with_full_package_name_returns_the_specs_of_that_package() {
        let dir = tempdir().expect("temp dir");
        write_project(dir.path());
        let project = project(dir.path());

        let by_full_name = render(&project, &args(false, &[], &["demo-frontend"])).expect("render");
        let by_short_name = render(&project, &args(false, &[], &["frontend"])).expect("render");

        assert_eq!(by_full_name, by_short_name);
        assert!(by_full_name.contains("list-pizza-menu"));
    }

    #[test]
    fn render_errors_on_unknown_package() {
        let dir = tempdir().expect("temp dir");
        write_project(dir.path());
        let project = project(dir.path());

        let result = render(&project, &args(false, &[], &["missing"]));

        assert!(result.is_err());
    }

    #[test]
    fn render_errors_on_unknown_id() {
        let dir = tempdir().expect("temp dir");
        write_project(dir.path());
        let project = project(dir.path());

        let result = render(&project, &args(false, &["missing"], &[]));

        assert!(result.is_err());
    }
}
