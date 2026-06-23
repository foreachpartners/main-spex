#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::super::*;
    use crate::biz_spec::{BizSpec, Status};
    use crate::tech_spec::TechSpec;

    fn biz(id: &str) -> BizSpec {
        BizSpec {
            id: id.to_string(),
            title: "Place an order".to_string(),
            description: "  Customers complete a purchase.  ".to_string(),
            acceptance_criteria: vec![
                "[x] MUST create an order".to_string(),
                "[ ] SHOULD show a confirmation".to_string(),
            ],
            status: Status::Active,
            coverage_ignore: Vec::new(),
        }
    }

    fn tech(id: &str) -> TechSpec {
        let mut examples = BTreeMap::new();
        examples.insert("prefer".to_string(), "let x = Secret::new();\n".to_string());
        TechSpec {
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
    fn biz_section_carries_id_title_description_and_criteria() {
        let out = render_biz(&biz("BIZ-PLACE-PIZZA-ORDER"));
        assert_eq!(
            out,
            "# BIZ-PLACE-PIZZA-ORDER — Place an order\n\
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
    fn tech_section_carries_examples() {
        let out = render_tech(&tech("TECH-SECRET-HANDLING"));
        assert_eq!(
            out,
            "# TECH-SECRET-HANDLING — Secret protection\n\
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
        let out = header("BIZ-A", "", "A description.");
        assert_eq!(out, "# BIZ-A\n\nA description.\n");
    }
}
