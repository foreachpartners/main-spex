#[cfg(test)]
mod tests {
    use super::super::*;
    use indoc::indoc;
    use tempfile::tempdir;

    fn spec(id: &str, extra: &str) -> serde_json::Value {
        serde_json::from_str(&format!(r#"{{"id": "{id}"{extra}}}"#)).expect("valid json")
    }

    #[test]
    fn scope_selected_defaults_to_true_when_empty() {
        assert!(scope_selected(&spec("A", r#", "scope": ["api"]"#), &[]));
    }

    #[test]
    fn scope_selected_matches_short_name_in_spec_scope() {
        let api = ["api".to_string()];
        assert!(scope_selected(&spec("A", r#", "scope": ["api"]"#), &api));
        assert!(!scope_selected(
            &spec("A", r#", "scope": ["frontend"]"#),
            &api
        ));
        assert!(!scope_selected(&spec("A", ""), &api));
    }

    #[test]
    fn id_selected_defaults_to_true_when_empty() {
        assert!(id_selected(&spec("A", ""), &[]));
    }

    #[test]
    fn id_selected_filters_to_given_ids() {
        let ids = vec!["A".to_string(), "B".to_string()];
        assert!(id_selected(&spec("A", ""), &ids));
        assert!(!id_selected(&spec("C", ""), &ids));
    }

    fn write_workspace(dir: &std::path::Path) {
        std::fs::write(
            dir.join("spex.yaml"),
            indoc! {"
                project_name: demo
                cli_version: 0.0.0
                repositories:
                  - repository_name: demo-api
                    repository_short_name: api
                    path: ../demo-api
                  - repository_name: demo-frontend
                    repository_short_name: frontend
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
                scope: [api]
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
                scope: [frontend]
            "#},
        )
        .expect("write spec");
    }

    fn workspace(dir: &std::path::Path) -> Workspace {
        Workspace {
            dir: dir.to_path_buf(),
            config: crate::config::load(dir).expect("load config"),
        }
    }

    fn args(ai: bool, id: &[&str], scope: &[&str]) -> SpecsArgs {
        SpecsArgs {
            ai,
            id: id.iter().map(|s| s.to_string()).collect(),
            scope: scope.iter().map(|s| s.to_string()).collect(),
            output: None,
        }
    }

    #[test]
    fn render_with_no_flags_prints_every_spec() {
        let dir = tempdir().expect("temp dir");
        write_workspace(dir.path());
        let workspace = workspace(dir.path());

        let yaml = render(&workspace, &args(false, &[], &[])).expect("render");

        assert_eq!(
            yaml,
            indoc! {"
                - id: list-pizza-menu
                  title: List the pizza menu
                  description: Lets a customer browse the menu.
                  acceptance_criteria:
                  - '[ ] MUST list every active pizza'
                  scope:
                  - frontend

                - id: place-pizza-order
                  title: Place a pizza order
                  description: Lets a customer place an order.
                  acceptance_criteria:
                  - '[ ] MUST accept a cart'
                  scope:
                  - api
            "}
        );
    }

    #[test]
    fn render_with_ai_flag_adds_path() {
        let dir = tempdir().expect("temp dir");
        write_workspace(dir.path());
        let workspace = workspace(dir.path());

        let yaml = render(&workspace, &args(true, &["place-pizza-order"], &[])).expect("render");

        assert_eq!(
            yaml,
            indoc! {"
                - id: place-pizza-order
                  path: specs/place-pizza-order.yaml
                  title: Place a pizza order
                  description: Lets a customer place an order.
                  acceptance_criteria:
                  - '[ ] MUST accept a cart'
                  scope:
                  - api
            "}
        );
    }

    #[test]
    fn render_with_single_scope_returns_only_matching_specs() {
        let dir = tempdir().expect("temp dir");
        write_workspace(dir.path());
        let workspace = workspace(dir.path());

        let yaml = render(&workspace, &args(false, &[], &["frontend"])).expect("render");

        assert_eq!(
            yaml,
            indoc! {"
                - id: list-pizza-menu
                  title: List the pizza menu
                  description: Lets a customer browse the menu.
                  acceptance_criteria:
                  - '[ ] MUST list every active pizza'
                  scope:
                  - frontend
            "}
        );
    }

    #[test]
    fn render_with_full_repo_name_does_not_match_scope() {
        let dir = tempdir().expect("temp dir");
        write_workspace(dir.path());
        let workspace = workspace(dir.path());

        let result = render(&workspace, &args(false, &[], &["demo-api"]));

        assert!(result.is_err());
    }

    #[test]
    fn render_errors_on_unknown_scope() {
        let dir = tempdir().expect("temp dir");
        write_workspace(dir.path());
        let workspace = workspace(dir.path());

        let result = render(&workspace, &args(false, &[], &["missing"]));

        assert!(result.is_err());
    }
}
