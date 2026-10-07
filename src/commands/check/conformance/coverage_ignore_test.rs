#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::rule::Severity;
    use crate::spec::CoverageIgnore;
    use crate::support::testing;

    fn contract_ignore(ac: Option<&str>) -> CoverageIgnore {
        CoverageIgnore {
            layer: CoverageLayer::Contract,
            ac: ac.map(testing::ac_id),
        }
    }

    fn spec(contract: ContractLevel, coverage_ignore: Vec<CoverageIgnore>) -> Spec {
        Spec {
            contract,
            coverage_ignore,
            ..testing::spec("a")
        }
    }

    #[test]
    fn coverage_ignore_whole_layer_contract_is_silent_with_contract_single() {
        let spec = spec(ContractLevel::Single, vec![contract_ignore(None)]);
        assert!(validate_coverage_ignore(&spec).is_empty());
    }

    #[test]
    fn coverage_ignore_whole_layer_contract_is_silent_with_for_each_ac() {
        let spec = spec(ContractLevel::ForEachAc, vec![contract_ignore(None)]);
        assert!(validate_coverage_ignore(&spec).is_empty());
    }

    #[test]
    fn coverage_ignore_per_ac_contract_is_silent_with_for_each_ac() {
        let spec = spec(
            ContractLevel::ForEachAc,
            vec![contract_ignore(Some("ac-a"))],
        );
        assert!(validate_coverage_ignore(&spec).is_empty());
    }

    #[test]
    fn coverage_ignore_contract_errors_with_contract_none() {
        let spec = spec(
            ContractLevel::None,
            vec![contract_ignore(None), contract_ignore(Some("ac-a"))],
        );

        let findings = validate_coverage_ignore(&spec);
        assert_eq!(findings.len(), 2);
        for finding in &findings {
            assert_eq!(finding.severity, Severity::Error);
            assert_eq!(finding.check, "coverage-ignore");
            assert_eq!(finding.location.as_deref(), Some("specs/a.yaml"));
            assert!(finding.message.contains("contract: none"));
        }
    }

    #[test]
    fn coverage_ignore_per_ac_contract_errors_with_contract_single() {
        let spec = spec(ContractLevel::Single, vec![contract_ignore(Some("ac-a"))]);

        let findings = validate_coverage_ignore(&spec);
        assert_eq!(findings.len(), 1);
        let finding = &findings[0];
        assert_eq!(finding.severity, Severity::Error);
        assert_eq!(finding.check, "coverage-ignore");
        assert!(finding.message.contains("contract: single"));
        assert!(finding.message.contains("contract: for-each-ac"));
    }

    #[test]
    fn coverage_ignore_other_layers_are_always_silent() {
        let spec = spec(
            ContractLevel::None,
            vec![
                CoverageIgnore {
                    layer: CoverageLayer::Impl,
                    ac: Some(testing::ac_id("ac-one")),
                },
                CoverageIgnore {
                    layer: CoverageLayer::Test,
                    ac: None,
                },
            ],
        );
        assert!(validate_coverage_ignore(&spec).is_empty());
    }
}
