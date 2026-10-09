#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::commands::coverage::annotation_index::AnnotationIndex;
    use crate::spec::{AcceptanceCriterion, Statement};
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
            missing,
        }
    }

    fn contract_criterion(id: &str, contract_present: bool) -> CriterionCoverage {
        CriterionCoverage {
            id: testing::ac_id(id),
            text: String::new(),
            is_done: true,
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
        assert_eq!(pending_summary(&coverage), "impl(ac-b), test(ac-b)");
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
    fn pending_summary_names_a_pending_per_ac_contract() {
        let spec = spec("a", Status::Active);
        let mut criterion = contract_criterion("ac-a", false);
        criterion.is_done = false;
        let coverage = coverage(&spec, false, vec![criterion]);
        assert_eq!(pending_summary(&coverage), "contract(ac-a)");
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
        spec.scope = vec!["api".to_string()];
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

    #[test]
    fn render_reports_no_tracked_specs() {
        let text = rendered(100, false, &[spec("only-planned", Status::Planned)]);

        assert!(text.contains("Gaps (0)\n  no active or in-progress specs\n"));
    }
}
