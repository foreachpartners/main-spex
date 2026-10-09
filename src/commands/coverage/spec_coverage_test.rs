#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::spec::{CoverageIgnore, Status};
    use crate::support::testing;

    const FULL_COVERAGE_OF_AC_A: &str =
        "// @spec contract a\n// @spec impl a#ac-a\n// @spec test a#ac-a\n";

    fn spec(ac_ids: &[&str]) -> Spec {
        Spec {
            acceptance_criteria: ac_ids.iter().map(|id| testing::criterion(id)).collect(),
            status: Status::Active,
            ..testing::spec("a")
        }
    }

    fn not_done(id: &str) -> AcceptanceCriterion {
        AcceptanceCriterion {
            is_done: false,
            ..testing::criterion(id)
        }
    }

    fn ignoring(layer: CoverageLayer, ac: Option<&str>) -> CoverageIgnore {
        CoverageIgnore {
            layer,
            ac: ac.map(testing::ac_id),
        }
    }

    fn coverage<'a>(spec: &'a Spec, annotations: &str) -> SpecCoverage<'a> {
        SpecCoverage::new(spec, &AnnotationIndex::scan(annotations))
    }

    #[test]
    fn only_active_and_in_progress_specs_are_tracked() {
        for status in [Status::Planned, Status::Blocked, Status::Deprecated] {
            let spec = Spec {
                status,
                ..spec(&["ac-a"])
            };
            let coverage = coverage(&spec, "");
            assert!(!coverage.is_tracked());
            assert!(!coverage.has_gap());
            assert!(!coverage.has_pending());
            assert!(coverage.criteria.is_empty());
        }
        for status in [Status::Active, Status::InProgress] {
            let spec = Spec {
                status,
                ..spec(&["ac-a"])
            };
            assert!(coverage(&spec, "").is_tracked());
        }
    }

    #[test]
    fn contract_is_missing_until_a_spec_level_annotation_is_found() {
        let spec = spec(&[]);
        assert!(coverage(&spec, "").contract_missing);
        assert!(!coverage(&spec, "// @spec contract a\n").contract_missing);
    }

    #[test]
    fn ac_qualified_contract_annotation_satisfies_the_spec_level_contract() {
        let spec = spec(&[]);
        assert!(!coverage(&spec, "// @spec contract a#ac-a\n").contract_missing);
    }

    #[test]
    fn ignored_contract_is_not_missing_and_leaves_no_gap() {
        let spec = Spec {
            coverage_ignore: vec![ignoring(CoverageLayer::Contract, None)],
            ..spec(&["ac-a"])
        };
        let coverage = coverage(&spec, "// @spec impl a#ac-a\n// @spec test a#ac-a\n");
        assert!(!coverage.contract_missing);
        assert!(!coverage.has_gap());
    }

    #[test]
    fn contract_is_never_missing_under_none() {
        let spec = Spec {
            contract: ContractLevel::None,
            ..spec(&[])
        };
        assert!(!coverage(&spec, "").contract_missing);
    }

    #[test]
    fn for_each_ac_requires_a_contract_per_criterion_not_per_spec() {
        let spec = Spec {
            contract: ContractLevel::ForEachAc,
            ..spec(&["ac-one", "ac-two"])
        };
        assert!(!coverage(&spec, "").contract_missing);

        let coverage = coverage(&spec, "// @spec contract a#ac-one\n");
        assert_eq!(
            coverage.criteria[0].missing,
            vec![CoverageLayer::Impl, CoverageLayer::Test]
        );
        assert_eq!(
            coverage.criteria[1].missing,
            vec![
                CoverageLayer::Contract,
                CoverageLayer::Impl,
                CoverageLayer::Test
            ]
        );
    }

    #[test]
    fn for_each_ac_contract_can_be_waived_for_one_criterion() {
        let spec = Spec {
            contract: ContractLevel::ForEachAc,
            coverage_ignore: vec![ignoring(CoverageLayer::Contract, Some("ac-one"))],
            ..spec(&["ac-one"])
        };
        let coverage = coverage(&spec, "");
        assert_eq!(
            coverage.criteria[0].missing,
            vec![CoverageLayer::Impl, CoverageLayer::Test]
        );
    }

    #[test]
    fn criterion_is_missing_the_layers_without_an_ac_qualified_annotation() {
        let spec = spec(&["ac-a", "ac-b"]);
        let coverage = coverage(&spec, FULL_COVERAGE_OF_AC_A);
        assert!(coverage.criteria[0].is_covered());
        assert!(!coverage.criteria[1].is_covered());
        assert_eq!(
            coverage.criteria[1].missing,
            vec![CoverageLayer::Impl, CoverageLayer::Test]
        );
    }

    #[test]
    fn per_ac_ignore_waives_one_layer_of_one_criterion() {
        let spec = Spec {
            coverage_ignore: vec![ignoring(CoverageLayer::Impl, Some("ac-a"))],
            ..spec(&["ac-a"])
        };
        let coverage = coverage(&spec, "");
        assert_eq!(coverage.criteria[0].missing, vec![CoverageLayer::Test]);
        assert!(!coverage.criteria[0].is_covered());
    }

    #[test]
    fn whole_layer_ignore_waives_the_layer_for_every_criterion() {
        let spec = Spec {
            coverage_ignore: vec![ignoring(CoverageLayer::Test, None)],
            ..spec(&["ac-a"])
        };
        let coverage = coverage(&spec, "");
        assert_eq!(coverage.criteria[0].missing, vec![CoverageLayer::Impl]);
    }

    #[test]
    fn unimplemented_criterion_marked_done_is_a_gap() {
        let spec = spec(&["ac-a", "ac-b"]);
        let coverage = coverage(&spec, FULL_COVERAGE_OF_AC_A);
        assert_eq!(coverage.health(), SpecHealth::Gap);
        assert_eq!(coverage.implemented_count(), 1);
        assert_eq!(coverage.criteria.len(), 2);
        assert!(!coverage.is_fully_implemented());
    }

    #[test]
    fn implemented_count_ignores_missing_tests() {
        let spec = spec(&["ac-a", "ac-b"]);
        let coverage = coverage(&spec, "// @spec contract a\n// @spec impl a#ac-a\n");
        assert_eq!(coverage.implemented_count(), 1);
        assert_eq!(coverage.health(), SpecHealth::Gap);
    }

    #[test]
    fn spec_with_every_layer_present_is_complete() {
        let spec = spec(&["ac-a"]);
        let coverage = coverage(&spec, FULL_COVERAGE_OF_AC_A);
        assert_eq!(coverage.health(), SpecHealth::Complete);
        assert!(coverage.is_fully_implemented());
    }

    #[test]
    fn unimplemented_criterion_not_done_is_pending_not_a_gap() {
        let spec = Spec {
            acceptance_criteria: vec![testing::criterion("ac-a"), not_done("ac-b")],
            ..spec(&[])
        };
        let coverage = coverage(&spec, FULL_COVERAGE_OF_AC_A);
        assert_eq!(coverage.health(), SpecHealth::Pending);
        assert!(!coverage.has_gap());
        assert!(coverage.has_pending());
    }

    #[test]
    fn unimplemented_criterion_marked_done_is_not_pending() {
        let spec = spec(&["ac-a"]);
        let coverage = coverage(&spec, "// @spec contract a\n");
        assert!(coverage.has_gap());
        assert!(!coverage.has_pending());
    }

    #[test]
    fn missing_per_ac_contract_is_a_gap_under_for_each_ac() {
        let spec = Spec {
            contract: ContractLevel::ForEachAc,
            ..spec(&["ac-a", "ac-b"])
        };
        let annotated_but_for_contract_of_ac_b = "// @spec contract a#ac-a\n// @spec impl a#ac-a\n// @spec test a#ac-a\n// @spec impl a#ac-b\n// @spec test a#ac-b\n";
        let coverage = coverage(&spec, annotated_but_for_contract_of_ac_b);
        assert_eq!(coverage.health(), SpecHealth::Gap);
        assert_eq!(coverage.criteria[1].missing, vec![CoverageLayer::Contract]);
    }

    #[test]
    fn complete_per_ac_contracts_leave_no_gap_under_for_each_ac() {
        let spec = Spec {
            contract: ContractLevel::ForEachAc,
            ..spec(&["ac-a"])
        };
        let coverage = coverage(
            &spec,
            "// @spec contract a#ac-a\n// @spec impl a#ac-a\n// @spec test a#ac-a\n",
        );
        assert_eq!(coverage.health(), SpecHealth::Complete);
    }

    #[test]
    fn missing_per_ac_contract_is_pending_when_not_done_under_for_each_ac() {
        let spec = Spec {
            contract: ContractLevel::ForEachAc,
            acceptance_criteria: vec![not_done("ac-a")],
            ..spec(&[])
        };
        let coverage = coverage(&spec, "// @spec impl a#ac-a\n// @spec test a#ac-a\n");
        assert_eq!(coverage.health(), SpecHealth::Pending);
        assert_eq!(coverage.pending_criteria().len(), 1);
    }

    #[test]
    fn gapped_and_pending_criteria_are_split_by_done_flag() {
        let spec = Spec {
            acceptance_criteria: vec![testing::criterion("ac-a"), not_done("ac-b")],
            ..spec(&[])
        };
        let coverage = coverage(&spec, "// @spec contract a\n");
        let ids = |criteria: Vec<&CriterionCoverage>| -> Vec<String> {
            criteria
                .iter()
                .map(|criterion| criterion.id.as_str().to_string())
                .collect()
        };
        assert_eq!(ids(coverage.gapped_criteria()), vec!["ac-a"]);
        assert_eq!(ids(coverage.pending_criteria()), vec!["ac-b"]);
    }
}
