#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::super::*;
    use crate::capability_spec::{CapabilitySpec, Status};
    use crate::engineering_spec::EngineeringSpec;

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
}
