#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::spec::CoverageLayer;
    use crate::support::testing;

    fn parse(yaml: &str) -> SpecConfig {
        serde_yaml_ng::from_str(yaml).expect("parse spec config")
    }

    #[test]
    fn omitting_status_defaults_to_planned() {
        let config = parse("id: a\ncontract: single\n");
        assert_eq!(config.status, Status::Planned);
        assert!(config.coverage_ignore.is_empty());
    }

    #[test]
    fn coverage_ignore_parses_whole_layer_and_per_ac_entries() {
        let config = parse(
            "id: a\ncontract: single\ncoverage_ignore:\n  - layer: contract\n    reason: traced by the shared spec\n  - layer: impl\n    ac: no-null-typed-string\n    reason: enforced by the type checker\n",
        );
        assert_eq!(config.coverage_ignore.len(), 2);
        assert_eq!(config.coverage_ignore[0].layer, CoverageLayer::Contract);
        assert!(config.coverage_ignore[0].ac.is_none());
        assert_eq!(
            config.coverage_ignore[0].reason,
            "traced by the shared spec"
        );
        assert_eq!(config.coverage_ignore[1].layer, CoverageLayer::Impl);
        assert_eq!(
            config.coverage_ignore[1].ac,
            Some(testing::ac_id("no-null-typed-string"))
        );
    }

    #[test]
    fn coverage_ignore_without_a_reason_is_rejected() {
        let result: Result<SpecConfig, _> =
            serde_yaml_ng::from_str("id: a\ncontract: single\ncoverage_ignore:\n  - layer: test\n");
        let message = result.expect_err("missing reason").to_string();
        assert!(message.contains("reason"), "{message}");
    }

    #[test]
    fn contract_parses_all_levels() {
        for (yaml, level) in [
            ("none", ContractLevel::None),
            ("single", ContractLevel::Single),
            ("for-each-ac", ContractLevel::ForEachAc),
        ] {
            let config = parse(&format!("id: a\ncontract: {yaml}\n"));
            assert_eq!(config.contract, level);
        }
    }
}
