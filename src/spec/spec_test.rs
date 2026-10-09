#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::support::testing;

    fn spec(coverage_ignore: Vec<CoverageIgnore>) -> Spec {
        Spec {
            status: Status::Active,
            coverage_ignore,
            ..testing::spec("a")
        }
    }

    fn ignore(layer: CoverageLayer, ac: Option<&str>) -> CoverageIgnore {
        CoverageIgnore {
            layer,
            ac: ac.map(testing::ac_id),
        }
    }

    #[test]
    fn scope_selected_matches_any_requested_scope() {
        let api_and_web = Spec {
            scope: vec!["api".to_string(), "web".to_string()],
            ..testing::spec("a")
        };
        let web = Spec {
            scope: vec!["web".to_string()],
            ..testing::spec("b")
        };
        let unscoped = testing::spec("c");

        assert!(api_and_web.scope_selected(&["api".to_string()]));
        assert!(!web.scope_selected(&["api".to_string()]));
        assert!(unscoped.scope_selected(&[]));
    }

    #[test]
    fn layer_ignored_true_for_whole_layer_entry() {
        let spec = spec(vec![ignore(CoverageLayer::Contract, None)]);
        assert!(spec.layer_ignored(CoverageLayer::Contract));
        assert!(!spec.layer_ignored(CoverageLayer::Impl));
    }

    #[test]
    fn ac_ignored_true_when_layer_is_ignored_for_whole_spec() {
        let spec = spec(vec![ignore(CoverageLayer::Test, None)]);
        assert!(spec.ac_ignored(&testing::ac_id("any-ac"), CoverageLayer::Test));
    }

    #[test]
    fn ac_ignored_true_when_ac_specifically_ignored() {
        let spec = spec(vec![ignore(
            CoverageLayer::Impl,
            Some("no-null-typed-string"),
        )]);
        assert!(spec.ac_ignored(&testing::ac_id("no-null-typed-string"), CoverageLayer::Impl));
        assert!(!spec.ac_ignored(&testing::ac_id("other-ac"), CoverageLayer::Impl));
        assert!(!spec.ac_ignored(&testing::ac_id("no-null-typed-string"), CoverageLayer::Test));
    }

    #[test]
    fn ac_ignored_false_with_no_coverage_ignore() {
        let spec = spec(Vec::new());
        assert!(!spec.ac_ignored(&testing::ac_id("any-ac"), CoverageLayer::Impl));
    }

    #[test]
    fn ac_contract_required_respects_whole_layer_ignore() {
        let mut spec = spec(vec![ignore(CoverageLayer::Contract, None)]);
        spec.contract = ContractLevel::ForEachAc;
        assert!(!spec.ac_contract_required(&testing::ac_id("any-ac")));
    }

    #[test]
    fn ac_contract_required_only_under_for_each_ac() {
        let mut spec = spec(Vec::new());
        spec.contract = ContractLevel::Single;
        assert!(!spec.ac_contract_required(&testing::ac_id("any-ac")));
        spec.contract = ContractLevel::ForEachAc;
        assert!(spec.ac_contract_required(&testing::ac_id("any-ac")));
    }

    #[test]
    fn ac_contract_required_respects_per_ac_ignore() {
        let mut spec = spec(vec![ignore(CoverageLayer::Contract, Some("waived-ac"))]);
        spec.contract = ContractLevel::ForEachAc;
        assert!(!spec.ac_contract_required(&testing::ac_id("waived-ac")));
        assert!(spec.ac_contract_required(&testing::ac_id("other-ac")));
    }

    #[test]
    fn from_config_keeps_the_path_and_numbers_criteria_by_source_line() {
        let raw = "id: a\ncontract: single\nacceptance_criteria:\n  - id: one\n    must: x\n  - id: two\n    must: y\n";
        let config = serde_yaml_ng::from_str(raw).expect("parse spec config");

        let spec = Spec::from_config(config, PathBuf::from("/spex/specs/a.yaml"), raw);

        assert_eq!(spec.path, PathBuf::from("/spex/specs/a.yaml"));
        let lines: Vec<usize> = spec.acceptance_criteria.iter().map(|ac| ac.line).collect();
        assert_eq!(lines, vec![4, 6]);
    }

    #[test]
    fn from_config_treats_absent_is_done_as_done() {
        let raw = "id: a\ncontract: single\nacceptance_criteria:\n  - id: one\n    must: x\n  - id: two\n    must: y\n    is_done: false\n";
        let config = serde_yaml_ng::from_str(raw).expect("parse spec config");

        let spec = Spec::from_config(config, PathBuf::from("a.yaml"), raw);

        let done: Vec<bool> = spec
            .acceptance_criteria
            .iter()
            .map(|ac| ac.is_done)
            .collect();
        assert_eq!(done, vec![true, false]);
    }
}
