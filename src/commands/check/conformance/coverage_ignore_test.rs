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
            assert!(
                matches!(&finding.source, Source::Spec { check, .. } if check == "coverage-ignore")
            );
            assert_eq!(finding.location, Some(Location::in_file(spec.path.clone())));
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
        assert!(
            matches!(&finding.source, Source::Spec { check, .. } if check == "coverage-ignore")
        );
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

    fn package_waiver_spec(contract: ContractLevel, packages: &[&str]) -> Spec {
        Spec {
            packages: packages.iter().map(|name| name.to_string()).collect(),
            package_coverage_ignore: [(
                "api".to_string(),
                vec![CoverageIgnore {
                    layer: CoverageLayer::Impl,
                    ac: None,
                }],
            )]
            .into(),
            ..spec(contract, Vec::new())
        }
    }

    #[test]
    fn a_package_waiver_is_silent_when_the_spec_lists_the_package() {
        let spec = package_waiver_spec(ContractLevel::Single, &["api", "frontend"]);
        assert!(validate_coverage_ignore(&spec).is_empty());
    }

    #[test]
    fn a_package_waiver_errors_when_the_spec_does_not_list_the_package() {
        for packages in [&["frontend"][..], &[][..]] {
            let spec = package_waiver_spec(ContractLevel::Single, packages);

            let findings = validate_coverage_ignore(&spec);

            assert_eq!(findings.len(), 1);
            let finding = &findings[0];
            assert_eq!(finding.severity, Severity::Error);
            assert!(
                matches!(&finding.source, Source::Spec { check, .. } if check == "coverage-ignore")
            );
            assert_eq!(finding.location, Some(Location::in_file(spec.path.clone())));
            assert!(finding.message.contains("package `api`"));
            assert!(finding.message.contains("`packages` does not list `api`"));
        }
    }

    #[test]
    fn a_package_contract_waiver_is_matched_to_the_spec_contract_setting() {
        let mut spec = package_waiver_spec(ContractLevel::None, &["api"]);
        spec.package_coverage_ignore
            .insert("api".to_string(), vec![contract_ignore(None)]);

        let findings = validate_coverage_ignore(&spec);

        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("contract: none"));
    }
}
