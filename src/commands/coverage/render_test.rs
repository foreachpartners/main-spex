#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::commands::coverage::annotation_index::AnnotationIndex;
    use crate::spec::{AcceptanceCriterion, ContractLevel, CoverageIgnore, Statement};
    use crate::support::testing;

    fn spec(id: &str, status: Status) -> Spec {
        Spec {
            status,
            ..testing::spec(id)
        }
    }

    fn criterion(id: &str, impl_present: bool, test_present: bool) -> CriterionCoverage {
        pending_criterion(id, impl_present, test_present, true)
    }

    fn pending_criterion(
        id: &str,
        impl_present: bool,
        test_present: bool,
        is_done: bool,
    ) -> CriterionCoverage {
        let missing = [
            (CoverageLayer::Impl, impl_present),
            (CoverageLayer::Test, test_present),
        ]
        .into_iter()
        .filter_map(|(layer, present)| (!present).then_some(layer))
        .collect();
        CriterionCoverage {
            id: testing::ac_id(id),
            text: String::new(),
            is_done,
            is_started: impl_present,
            missing,
        }
    }

    fn contract_criterion(id: &str, contract_present: bool) -> CriterionCoverage {
        CriterionCoverage {
            id: testing::ac_id(id),
            text: String::new(),
            is_done: true,
            is_started: false,
            missing: if contract_present {
                Vec::new()
            } else {
                vec![CoverageLayer::Contract]
            },
        }
    }

    fn coverage(
        spec: &Spec,
        contract_missing: bool,
        criteria: Vec<CriterionCoverage>,
    ) -> SpecCoverage<'_> {
        SpecCoverage {
            spec,
            contract_missing,
            criteria,
        }
    }

    #[test]
    fn missing_summary_lists_contract_then_impl_then_test() {
        let spec = spec("a", Status::Active);
        let coverage = coverage(
            &spec,
            true,
            vec![
                criterion("ac-a", false, false),
                criterion("ac-b", true, false),
            ],
        );
        assert_eq!(
            missing_summary(&coverage),
            "contract, impl(ac-a), test(ac-a, ac-b)"
        );
    }

    #[test]
    fn missing_summary_names_a_missing_contract() {
        let spec = spec("a", Status::Active);
        let coverage = coverage(&spec, true, vec![criterion("ac-a", true, true)]);
        assert_eq!(missing_summary(&coverage), "contract");
    }

    #[test]
    fn missing_summary_is_empty_without_gaps() {
        let spec = spec("a", Status::Active);
        let coverage = coverage(&spec, false, vec![criterion("ac-a", true, true)]);
        assert_eq!(missing_summary(&coverage), "");
    }

    #[test]
    fn missing_summary_names_impl_and_test_per_criterion() {
        let spec = spec("a", Status::Active);
        let coverage = coverage(
            &spec,
            false,
            vec![
                criterion("ac-a", true, false),
                criterion("ac-b", false, false),
            ],
        );
        assert_eq!(missing_summary(&coverage), "impl(ac-b), test(ac-a, ac-b)");
    }

    #[test]
    fn missing_summary_names_per_ac_contracts() {
        let spec = spec("a", Status::Active);
        let coverage = coverage(
            &spec,
            false,
            vec![
                contract_criterion("ac-a", true),
                contract_criterion("ac-b", false),
            ],
        );
        assert_eq!(missing_summary(&coverage), "contract(ac-b)");
    }

    #[test]
    fn missing_summary_ignores_pending_criteria() {
        let spec = spec("a", Status::Active);
        let coverage = coverage(
            &spec,
            false,
            vec![
                criterion("ac-a", true, true),
                pending_criterion("ac-b", false, false, false),
            ],
        );
        assert_eq!(missing_summary(&coverage), "");
        assert_eq!(pending_summary(&coverage), "impl(ac-b)");
    }

    #[test]
    fn pending_summary_ignores_gapped_criteria() {
        let spec = spec("a", Status::Active);
        let coverage = coverage(
            &spec,
            false,
            vec![pending_criterion("ac-a", false, false, true)],
        );
        assert_eq!(missing_summary(&coverage), "impl(ac-a), test(ac-a)");
        assert_eq!(pending_summary(&coverage), "");
    }

    #[test]
    fn pending_summary_names_only_the_impl_layer_of_a_pending_criterion() {
        let spec = spec("a", Status::Active);
        let mut criterion = pending_criterion("ac-a", false, false, false);
        criterion.missing.push(CoverageLayer::Contract);
        let coverage = coverage(&spec, false, vec![criterion]);
        assert_eq!(pending_summary(&coverage), "impl(ac-a)");
    }

    #[test]
    fn single_contract_is_pending_while_no_criterion_is_started_or_done() {
        let spec = spec("a", Status::Active);
        let coverage = coverage(
            &spec,
            true,
            vec![pending_criterion("ac-a", false, false, false)],
        );
        assert!(coverage.has_contract_pending());
        assert_eq!(missing_summary(&coverage), "");
        assert_eq!(pending_summary(&coverage), "impl(ac-a)");
    }

    #[test]
    fn started_criterion_missing_its_test_is_listed_as_missing_not_pending() {
        let spec = spec("a", Status::Active);
        let coverage = coverage(
            &spec,
            false,
            vec![pending_criterion("ac-a", true, false, false)],
        );
        assert_eq!(missing_summary(&coverage), "test(ac-a)");
        assert_eq!(pending_summary(&coverage), "");
    }

    #[test]
    fn missing_label_lists_contract_with_impl_and_test() {
        let mut criterion = contract_criterion("ac-a", false);
        criterion.missing.push(CoverageLayer::Test);
        assert_eq!(missing_label(&criterion), "missing contract & test");
    }

    fn rendered(width: usize, verbose: bool, specs: &[Spec]) -> String {
        let report = CoverageReport::new(specs, &AnnotationIndex::new());
        let text = render(&report, width, verbose);
        anstream::adapter::strip_str(&text).to_string()
    }

    fn gapped_spec() -> Spec {
        let mut spec = spec("api-thing", Status::Active);
        spec.description = "Does the thing.".to_string();
        spec.packages = vec!["api".to_string()];
        spec.acceptance_criteria = vec![AcceptanceCriterion {
            statement: Statement::Must(
                "do the first thing in a sentence long enough to wrap".to_string(),
            ),
            ..testing::criterion("ac-a")
        }];
        spec
    }

    #[test]
    fn render_lists_a_gapped_spec_with_what_is_missing() {
        let text = rendered(100, false, &[gapped_spec()]);

        assert_eq!(
            text,
            format!(
                "spex {}\n\nSpecs (1)\n  active (1)\n\nGaps (1)\n  api\n    ✗  api-thing  0/1 AC  missing: contract, impl(ac-a), test(ac-a)\n\nplanned (0), in-progress (0), active (1), gaps (1)\n",
                env!("CARGO_PKG_VERSION")
            )
        );
    }

    #[test]
    fn render_lists_a_gapped_spec_without_packages_under_no_package() {
        let mut spec = gapped_spec();
        spec.packages = Vec::new();

        let text = rendered(100, false, &[spec]);

        assert!(
            text.contains("Gaps (1)\n  no package\n    ✗  api-thing"),
            "got:\n{text}"
        );
    }

    #[test]
    fn render_wraps_verbose_detail_to_the_given_width() {
        let text = rendered(40, true, &[gapped_spec()]);

        let pad = " ".repeat(36);
        assert!(text.contains("        Does the thing.\n"));
        assert!(
            text.contains(&format!(
                "        ac-a (missing impl & test): do the first thing\n{pad}in a sentence long\n{pad}enough to wrap\n"
            )),
            "got:\n{text}"
        );
    }

    fn in_progress_spec_with_waived_impl() -> Spec {
        let mut spec = spec("api-thing", Status::InProgress);
        spec.contract = ContractLevel::None;
        spec.acceptance_criteria = vec![AcceptanceCriterion {
            is_done: false,
            ..testing::criterion("ac-a")
        }];
        spec.coverage_ignore = vec![CoverageIgnore {
            layer: CoverageLayer::Impl,
            ac: Some(testing::ac_id("ac-a")),
        }];
        spec
    }

    #[test]
    fn render_lists_an_in_progress_spec_with_its_unimplemented_criteria() {
        let mut spec = spec("api-thing", Status::InProgress);
        spec.contract = ContractLevel::None;
        spec.acceptance_criteria = vec![
            AcceptanceCriterion {
                is_done: false,
                ..testing::criterion("ac-a")
            },
            AcceptanceCriterion {
                is_done: false,
                ..testing::criterion("ac-b")
            },
        ];

        let text = rendered(100, false, &[spec]);

        assert!(
            text.contains(
                "  in-progress (1)\n    ●  api-thing  0/2 AC  in-progress: impl(ac-a, ac-b)\n"
            ),
            "got:\n{text}"
        );
    }

    #[test]
    fn render_omits_the_in_progress_line_when_no_impl_is_pending() {
        let text = rendered(100, false, &[in_progress_spec_with_waived_impl()]);

        assert!(
            text.contains("  in-progress (1)\n    ●  api-thing  1/1 AC\n"),
            "got:\n{text}"
        );
    }

    fn in_progress_spec_with_gap_and_pending_work() -> Spec {
        let mut spec = spec("api-thing", Status::InProgress);
        spec.description = "Does the thing.".to_string();
        spec.packages = vec!["api".to_string()];
        spec.contract = ContractLevel::None;
        spec.acceptance_criteria = vec![
            testing::criterion("ac-a"),
            AcceptanceCriterion {
                is_done: false,
                ..testing::criterion("ac-b")
            },
        ];
        spec
    }

    #[test]
    fn render_lists_a_gapped_in_progress_spec_in_both_blocks_with_pending_work_only_in_status() {
        let text = rendered(100, false, &[in_progress_spec_with_gap_and_pending_work()]);

        assert_eq!(
            text,
            format!(
                "spex {}\n\nSpecs (1)\n  in-progress (1)\n    ✗  api-thing  0/2 AC  in-progress: impl(ac-b)\n\nGaps (1)\n  api\n    ✗  api-thing  0/2 AC  missing: impl(ac-a), test(ac-a)\n\nplanned (0), in-progress (1), active (0), gaps (1)\n",
                env!("CARGO_PKG_VERSION")
            )
        );
    }

    #[test]
    fn render_shows_verbose_detail_in_each_block_for_its_own_criteria() {
        let text = rendered(100, true, &[in_progress_spec_with_gap_and_pending_work()]);

        assert!(
            text.contains(
                "    ✗  api-thing  0/2 AC\n        Does the thing.\n        ac-b: ac-b\n\n"
            ),
            "got:\n{text}"
        );
        assert!(
            text.contains(
                "    ✗  api-thing  0/2 AC\n        Does the thing.\n        ac-a (missing impl & test): ac-a\n\n"
            ),
            "got:\n{text}"
        );
    }

    #[test]
    fn render_reports_no_tracked_specs() {
        let text = rendered(100, false, &[spec("only-planned", Status::Planned)]);

        assert!(text.contains("Gaps (0)\n  no active or in-progress specs\n"));
    }
}
