#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::super::*;
    use crate::capability_spec::{CapabilitySpec, Status};
    use crate::config::{Config, Repository};
    use crate::engineering_spec::EngineeringSpec;

    fn repository(name: &str, short_name: &str, es_specs: Vec<&str>) -> Repository {
        Repository {
            name: name.to_string(),
            short_name: short_name.to_string(),
            path: format!("../{name}"),
            contracts: Vec::new(),
            es_specs: es_specs.into_iter().map(str::to_string).collect(),
            checks_additional: Vec::new(),
            checks_override: Vec::new(),
            checks_preexisting_errors: Vec::new(),
        }
    }

    fn config(repositories: Vec<Repository>) -> Config {
        Config {
            project_name: "pizza-shop".to_string(),
            cli_version: "0.0.0".to_string(),
            repositories,
        }
    }

    fn capability(id: &str) -> CapabilitySpec {
        CapabilitySpec {
            id: id.to_string(),
            title: "Place an order".to_string(),
            description: "  Customers complete a purchase.  ".to_string(),
            acceptance_criteria: vec![
                "[x] MUST create an order".to_string(),
                "[ ] SHOULD show a confirmation".to_string(),
            ],
            blocked_by: Vec::new(),
            status: Status::Active,
            coverage_ignore: Vec::new(),
        }
    }

    fn engineering(id: &str) -> EngineeringSpec {
        let mut examples = BTreeMap::new();
        examples.insert("prefer".to_string(), "let x = Secret::new();\n".to_string());
        EngineeringSpec {
            id: id.to_string(),
            title: "Secret protection".to_string(),
            description: "Never commit secrets.".to_string(),
            acceptance_criteria: vec!["MUST NOT commit secrets".to_string()],
            include_for_all_repos: false,
            include_for_spex: None,
            checks: Vec::new(),
            examples,
        }
    }

    #[test]
    fn capability_section_carries_id_title_description_and_criteria() {
        let out = render_capability(&capability("CS-PLACE-PIZZA-ORDER"));
        assert_eq!(
            out,
            "# CS-PLACE-PIZZA-ORDER — Place an order\n\
             \n\
             Customers complete a purchase.\n\
             \n\
             ## Acceptance criteria\n\
             \n\
             - [x] MUST create an order\n\
             - [ ] SHOULD show a confirmation\n"
        );
    }

    #[test]
    fn engineering_section_carries_examples() {
        let out = render_engineering(&engineering("ES-SECRET-HANDLING"));
        assert_eq!(
            out,
            "# ES-SECRET-HANDLING — Secret protection\n\
             \n\
             Never commit secrets.\n\
             \n\
             ## Acceptance criteria\n\
             \n\
             - MUST NOT commit secrets\n\
             \n\
             ## Examples\n\
             \n\
             ### prefer\n\
             \n\
             ```\n\
             let x = Secret::new();\n\
             ```\n"
        );
    }

    #[test]
    fn header_omits_title_when_absent() {
        let out = header("CS-A", "", "A description.");
        assert_eq!(out, "# CS-A\n\nA description.\n");
    }

    #[test]
    fn expand_repos_replaces_repo_name_with_its_es_specs() {
        let config = config(vec![repository(
            "pizza-shop-backend",
            "backend",
            vec!["ES-SECRET-HANDLING", "ES-LOGGING"],
        )]);

        let ids = expand_repos(
            &config,
            &["backend".to_string(), "CS-PLACE-PIZZA-ORDER".to_string()],
        );

        assert_eq!(
            ids,
            vec![
                "ES-SECRET-HANDLING".to_string(),
                "ES-LOGGING".to_string(),
                "CS-PLACE-PIZZA-ORDER".to_string(),
            ]
        );
    }

    #[test]
    fn expand_repos_matches_full_name_and_dedupes() {
        let config = config(vec![repository(
            "pizza-shop-backend",
            "backend",
            vec!["ES-SECRET-HANDLING"],
        )]);

        let ids = expand_repos(
            &config,
            &[
                "pizza-shop-backend".to_string(),
                "ES-SECRET-HANDLING".to_string(),
            ],
        );

        assert_eq!(ids, vec!["ES-SECRET-HANDLING".to_string()]);
    }
}
