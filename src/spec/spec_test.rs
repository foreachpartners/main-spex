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
    fn coverage_ignore_parses_whole_layer_and_per_ac_entries() {
        let spec: Spec = serde_yaml_ng::from_str(
            "id: a\ncontract: single\ncoverage_ignore:\n  - layer: contract\n  - layer: impl\n    ac: no-null-typed-string\n",
        )
        .expect("parse spec");
        assert!(spec.layer_ignored(CoverageLayer::Contract));
        assert!(spec.ac_ignored(&testing::ac_id("no-null-typed-string"), CoverageLayer::Impl));
        assert!(!spec.ac_ignored(&testing::ac_id("other-ac"), CoverageLayer::Impl));
    }

    #[test]
    fn contract_parses_all_levels() {
        for (yaml, level) in [
            ("none", ContractLevel::None),
            ("single", ContractLevel::Single),
            ("for-each-ac", ContractLevel::ForEachAc),
        ] {
            let spec: Spec =
                serde_yaml_ng::from_str(&format!("id: a\ncontract: {yaml}\n")).expect("parse spec");
            assert_eq!(spec.contract, level);
        }
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
    fn is_done_defaults_true_when_absent() {
        let ac = testing::criterion("a");
        assert!(ac.is_done());
    }

    #[test]
    fn is_done_reads_explicit_false() {
        let ac = AcceptanceCriterion {
            is_done: Some(false),
            ..testing::criterion("a")
        };
        assert!(!ac.is_done());
    }
}
