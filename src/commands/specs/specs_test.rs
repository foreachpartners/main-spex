#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::error::describe;
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
                project_short_name: abc
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

    fn args(id: &[&str], package: &[&str]) -> SpecsArgs {
        SpecsArgs {
            id: id.iter().map(|s| s.to_string()).collect(),
            package: package.iter().map(|s| s.to_string()).collect(),
            search: Vec::new(),
            fields: Vec::new(),
            output: None,
        }
    }

    fn names(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn render_with_no_flags_prints_every_spec_with_its_path_after_the_id() {
        let dir = tempdir().expect("temp dir");
        write_project(dir.path());
        let project = project(dir.path());

        let yaml = render(&project, &args(&[], &[])).expect("render");

        assert_eq!(
            yaml,
            formatdoc! {"
                - id: list-pizza-menu
                  path: {dir}/specs/list-pizza-menu.yaml
                  title: List the pizza menu
                  description: Lets a customer browse the menu.
                  acceptance_criteria:
                  - '[ ] MUST list every active pizza'
                  packages:
                  - frontend

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

        let yaml = render(&project, &args(&[], &["frontend"])).expect("render");

        assert_eq!(
            yaml,
            formatdoc! {"
                - id: list-pizza-menu
                  path: {dir}/specs/list-pizza-menu.yaml
                  title: List the pizza menu
                  description: Lets a customer browse the menu.
                  acceptance_criteria:
                  - '[ ] MUST list every active pizza'
                  packages:
                  - frontend
            ", dir = dir.path().display()}
        );
    }

    #[test]
    fn render_with_full_package_name_returns_the_specs_of_that_package() {
        let dir = tempdir().expect("temp dir");
        write_project(dir.path());
        let project = project(dir.path());

        let by_full_name = render(&project, &args(&[], &["demo-frontend"])).expect("render");
        let by_short_name = render(&project, &args(&[], &["frontend"])).expect("render");

        assert_eq!(by_full_name, by_short_name);
        assert!(by_full_name.contains("list-pizza-menu"));
    }

    #[test]
    fn render_with_fields_prints_only_those_fields_in_spec_order() {
        let dir = tempdir().expect("temp dir");
        write_project(dir.path());
        let project = project(dir.path());
        let args = SpecsArgs {
            fields: names(&["description", "id"]),
            ..args(&[], &[])
        };

        let yaml = render(&project, &args).expect("render");

        assert_eq!(
            yaml,
            indoc! {"
                - id: list-pizza-menu
                  description: Lets a customer browse the menu.

                - id: place-pizza-order
                  description: Lets a customer place an order.
            "}
        );
    }

    #[test]
    fn render_with_the_path_field_prints_the_spec_path() {
        let dir = tempdir().expect("temp dir");
        write_project(dir.path());
        let project = project(dir.path());
        let args = SpecsArgs {
            fields: names(&["id", "path"]),
            ..args(&["place-pizza-order"], &[])
        };

        let yaml = render(&project, &args).expect("render");

        assert_eq!(
            yaml,
            formatdoc! {"
                - id: place-pizza-order
                  path: {dir}/specs/place-pizza-order.yaml
            ", dir = dir.path().display()}
        );
    }

    #[test]
    fn render_with_a_spec_field_no_spec_sets_prints_the_specs_without_it() {
        let dir = tempdir().expect("temp dir");
        write_project(dir.path());
        let project = project(dir.path());
        let args = SpecsArgs {
            fields: names(&["id", "blocked_by"]),
            ..args(&["place-pizza-order"], &[])
        };

        let yaml = render(&project, &args).expect("render");

        assert_eq!(yaml, "- id: place-pizza-order\n");
    }

    #[test]
    fn render_errors_on_a_field_no_spec_has() {
        let dir = tempdir().expect("temp dir");
        write_project(dir.path());
        let project = project(dir.path());
        let args = SpecsArgs {
            fields: names(&["id", "titel"]),
            ..args(&[], &[])
        };

        let error = render(&project, &args).expect_err("unknown field");

        assert!(describe(&error).starts_with("unknown field `titel` — a spec has: id, "));
    }

    #[test]
    fn render_with_search_prints_specs_containing_any_keyword_ignoring_case() {
        let dir = tempdir().expect("temp dir");
        write_project(dir.path());
        let project = project(dir.path());
        let args = SpecsArgs {
            search: names(&["BROWSE", "nothing-like-this"]),
            fields: names(&["id"]),
            ..args(&[], &[])
        };

        let yaml = render(&project, &args).expect("render");

        assert_eq!(yaml, "- id: list-pizza-menu\n");
    }

    #[test]
    fn render_with_search_does_not_match_the_spec_path_or_field_names() {
        let dir = tempdir().expect("temp dir");
        write_project(dir.path());
        let project = project(dir.path());

        for keyword in ["specs", "acceptance_criteria"] {
            let args = SpecsArgs {
                search: names(&[keyword]),
                ..args(&[], &[])
            };

            let yaml = render(&project, &args).expect("render");

            assert_eq!(yaml, "[]\n", "keyword `{keyword}` matched");
        }
    }

    #[test]
    fn render_with_search_and_package_prints_only_specs_in_both() {
        let dir = tempdir().expect("temp dir");
        write_project(dir.path());
        let project = project(dir.path());
        let args = SpecsArgs {
            search: names(&["customer"]),
            fields: names(&["id"]),
            ..args(&[], &["api"])
        };

        let yaml = render(&project, &args).expect("render");

        assert_eq!(yaml, "- id: place-pizza-order\n");
    }

    #[test]
    fn render_errors_on_unknown_package() {
        let dir = tempdir().expect("temp dir");
        write_project(dir.path());
        let project = project(dir.path());

        let result = render(&project, &args(&[], &["missing"]));

        assert!(result.is_err());
    }

    #[test]
    fn render_errors_on_unknown_id() {
        let dir = tempdir().expect("temp dir");
        write_project(dir.path());
        let project = project(dir.path());

        let result = render(&project, &args(&["missing"], &[]));

        assert!(result.is_err());
    }
}
