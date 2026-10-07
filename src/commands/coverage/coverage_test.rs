#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::spec::{AcceptanceCriterion, CoverageLayer, Spec, Statement, Status};
    use crate::support::style::Style;
    use crate::support::testing;
    use std::path::Path;

    fn spec(id: &str, status: Status) -> Spec {
        Spec {
            status,
            ..testing::spec(id)
        }
    }

    fn ac(id: &str, impl_present: bool, test_present: bool) -> AcCoverage {
        pending_ac(id, impl_present, test_present, true)
    }

    fn pending_ac(id: &str, impl_present: bool, test_present: bool, is_done: bool) -> AcCoverage {
        let missing = [
            (CoverageLayer::Impl, impl_present),
            (CoverageLayer::Test, test_present),
        ]
        .into_iter()
        .filter_map(|(layer, present)| (!present).then_some(layer))
        .collect();
        AcCoverage {
            id: id.to_string(),
            text: String::new(),
            is_done,
            missing,
        }
    }

    fn contract_ac(id: &str, contract_present: bool) -> AcCoverage {
        AcCoverage {
            id: id.to_string(),
            text: String::new(),
            is_done: true,
            missing: if contract_present {
                Vec::new()
            } else {
                vec![CoverageLayer::Contract]
            },
        }
    }

    fn row(spec: &Spec, contract_missing: bool, ac_rows: Vec<AcCoverage>) -> SpecRow<'_> {
        SpecRow {
            spec,
            contract_missing,
            ac_rows,
        }
    }

    #[test]
    fn rows_track_coverage_only_for_active_and_in_progress() {
        let specs = vec![
            spec("A", Status::Active),
            spec("B", Status::Planned),
            spec("C", Status::Deprecated),
            spec("D", Status::InProgress),
        ];
        let rows = SpecRow::all(&specs, &Found::new());
        assert_eq!(rows.len(), 4);
        let tracked: Vec<&str> = rows
            .iter()
            .filter(|row| row.is_tracked())
            .map(|row| row.spec.id.as_str())
            .collect();
        assert_eq!(tracked, vec!["A", "D"]);
        assert!(
            rows.iter()
                .filter(|row| !row.is_tracked())
                .all(|row| { !row.has_gap() && !row.has_pending() && row.ac_rows.is_empty() })
        );
    }

    #[test]
    fn spec_link_targets_the_spec_file() {
        let spec = spec("A", Status::Active);
        let link = spec_link(&Style::colored(), Path::new("/workspace"), &spec);
        assert!(
            link.contains("file:///workspace/specs/A.yaml"),
            "got: {link}"
        );
    }

    #[test]
    fn omitting_status_defaults_to_planned_and_is_ignored() {
        let spec: Spec = serde_yaml_ng::from_str("id: A\ncontract: single\n").expect("parse spec");
        assert_eq!(spec.status, Status::Planned);
        assert!(spec.coverage_ignore.is_empty());
        let specs = [spec];
        let rows = SpecRow::all(&specs, &Found::new());
        assert!(!rows[0].is_tracked());
    }

    #[test]
    fn tracked_rows_carries_coverage_ignore() {
        let spec: Spec = serde_yaml_ng::from_str(
            "id: A\ncontract: single\nstatus: active\ncoverage_ignore:\n  - layer: contract\n  - layer: test\n",
        )
        .expect("parse spec");
        assert!(spec.layer_ignored(CoverageLayer::Contract));
        assert!(spec.layer_ignored(CoverageLayer::Test));
        assert!(!spec.layer_ignored(CoverageLayer::Impl));
        let specs = [spec];
        let rows = SpecRow::all(&specs, &Found::new());
        assert!(!rows[0].contract_missing);
    }

    #[test]
    fn tracked_rows_parses_per_ac_ignore() {
        let spec: Spec = serde_yaml_ng::from_str(
            "id: A\ncontract: single\nstatus: active\nacceptance_criteria:\n  - id: ac-1\n    must: \"do a\"\ncoverage_ignore:\n  - layer: impl\n    ac: ac-1\n",
        )
        .expect("parse spec");
        let specs = [spec];
        let rows = SpecRow::all(&specs, &Found::new());
        assert_eq!(rows[0].ac_rows[0].missing, vec![CoverageLayer::Test]);
    }

    #[test]
    fn missing_summary_lists_contract_then_impl_then_test() {
        let spec = spec("A", Status::Active);
        let row = row(
            &spec,
            true,
            vec![ac("ac-1", false, false), ac("ac-2", true, false)],
        );
        assert_eq!(
            row.missing_summary(),
            "contract, impl(ac-1), test(ac-1, ac-2)"
        );
    }

    #[test]
    fn missing_contract_is_a_gap_by_default() {
        let spec = spec("A", Status::Active);
        let row = row(&spec, true, vec![ac("ac-1", true, true)]);
        assert!(row.has_gap());
        assert_eq!(row.missing_summary(), "contract");
    }

    #[test]
    fn ignored_contract_is_not_a_gap() {
        let spec = spec("A", Status::Active);
        let row = row(&spec, false, vec![ac("ac-1", true, true)]);
        assert!(!row.has_gap());
        assert_eq!(row.missing_summary(), "");
    }

    #[test]
    fn unimplemented_ac_is_a_gap() {
        let spec = spec("A", Status::Active);
        let row = row(
            &spec,
            false,
            vec![ac("ac-1", true, true), ac("ac-2", false, true)],
        );
        assert!(row.has_gap());
        assert_eq!(row.ac_done(), 1);
        assert_eq!(row.ac_total(), 2);
    }

    #[test]
    fn ac_done_counts_impl_only_ignoring_missing_test() {
        let spec = spec("A", Status::Active);
        let row = row(
            &spec,
            false,
            vec![ac("ac-1", true, false), ac("ac-2", false, false)],
        );
        assert_eq!(row.ac_done(), 1);
        assert_eq!(row.ac_total(), 2);
        assert!(row.has_gap());
        assert_eq!(row.missing_summary(), "impl(ac-2), test(ac-1, ac-2)");
    }

    #[test]
    fn all_layers_and_ac_complete_has_no_gap() {
        let spec = spec("A", Status::Active);
        let row = row(
            &spec,
            false,
            vec![ac("ac-1", true, true), ac("ac-2", true, true)],
        );
        assert!(!row.has_gap());
    }

    #[test]
    fn unimplemented_ac_flagged_not_done_is_pending_not_a_gap() {
        let spec = spec("A", Status::Active);
        let row = row(
            &spec,
            false,
            vec![
                ac("ac-1", true, true),
                pending_ac("ac-2", false, false, false),
            ],
        );
        assert!(!row.has_gap());
        assert!(row.has_pending());
        assert_eq!(row.missing_summary(), "");
        assert_eq!(row.pending_summary(), "impl(ac-2), test(ac-2)");
    }

    #[test]
    fn unimplemented_ac_not_flagged_is_still_a_gap() {
        let spec = spec("A", Status::Active);
        let row = row(&spec, false, vec![pending_ac("ac-1", false, false, true)]);
        assert!(row.has_gap());
        assert!(!row.has_pending());
        assert_eq!(row.missing_summary(), "impl(ac-1), test(ac-1)");
    }

    #[test]
    fn missing_per_ac_contract_is_a_gap_under_for_each_ac() {
        let spec = spec("A", Status::Active);
        let row = row(
            &spec,
            false,
            vec![contract_ac("ac-1", true), contract_ac("ac-2", false)],
        );
        assert!(row.has_gap());
        assert_eq!(row.missing_summary(), "contract(ac-2)");
    }

    #[test]
    fn per_ac_contract_complete_has_no_gap_under_for_each_ac() {
        let spec = spec("A", Status::Active);
        let row = row(
            &spec,
            false,
            vec![contract_ac("ac-1", true), contract_ac("ac-2", true)],
        );
        assert!(!row.has_gap());
        assert_eq!(row.missing_summary(), "");
    }

    #[test]
    fn missing_contract_is_pending_not_a_gap_when_not_done_under_for_each_ac() {
        let mut ac = contract_ac("ac-1", false);
        ac.is_done = false;
        let spec = spec("A", Status::Active);
        let row = row(&spec, false, vec![ac]);
        assert!(!row.has_gap());
        assert!(row.has_pending());
        assert_eq!(row.pending_summary(), "contract(ac-1)");
    }

    #[test]
    fn ac_gap_label_lists_contract_with_impl_and_test() {
        let mut ac = contract_ac("ac-1", false);
        ac.missing.push(CoverageLayer::Test);
        assert_eq!(ac_gap_label(&ac), "missing contract & test".to_string());
    }

    #[test]
    fn status_groups_lists_ids_per_non_empty_status_in_order() {
        let specs = vec![
            spec("A", Status::Active),
            spec("B", Status::Planned),
            spec("C", Status::Planned),
            spec("D", Status::Deprecated),
        ];
        let rows = SpecRow::all(&specs, &Found::new());
        let groups = status_groups(&rows);
        let ids: Vec<(Status, Vec<&str>)> = groups
            .iter()
            .map(|(status, rows)| (*status, rows.iter().map(|r| r.spec.id.as_str()).collect()))
            .collect();
        assert_eq!(
            ids,
            vec![
                (Status::Active, vec!["A"]),
                (Status::Planned, vec!["B", "C"]),
                (Status::Deprecated, vec!["D"]),
            ]
        );
    }

    #[test]
    fn status_groups_skips_empty_statuses() {
        let specs = vec![spec("A", Status::Planned)];
        let rows = SpecRow::all(&specs, &Found::new());
        let groups = status_groups(&rows);
        let ids: Vec<(Status, Vec<&str>)> = groups
            .iter()
            .map(|(status, rows)| (*status, rows.iter().map(|r| r.spec.id.as_str()).collect()))
            .collect();
        assert_eq!(ids, vec![(Status::Planned, vec!["A"])]);
    }

    #[test]
    fn scope_groups_duplicates_multi_scope_rows_and_lists_unscoped_last() {
        let mut spec_a = spec("A", Status::Active);
        spec_a.scope = vec!["web".to_string(), "api".to_string()];
        let mut spec_b = spec("B", Status::Active);
        spec_b.scope = vec!["web".to_string()];
        let spec_c = spec("C", Status::Active);
        let a = row(&spec_a, false, Vec::new());
        let b = row(&spec_b, false, Vec::new());
        let c = row(&spec_c, false, Vec::new());

        let rows = [&a, &b, &c];
        let groups = scope_groups(&rows);

        let scopes: Vec<Option<&str>> = groups.iter().map(|(scope, _)| *scope).collect();
        assert_eq!(scopes, vec![Some("api"), Some("web"), None]);

        let ids: Vec<Vec<&str>> = groups
            .iter()
            .map(|(_, rows)| rows.iter().map(|row| row.spec.id.as_str()).collect())
            .collect();
        assert_eq!(ids, vec![vec!["A"], vec!["A", "B"], vec!["C"]]);
    }

    #[test]
    fn scope_selected_matches_any_requested_scope() {
        let mut a = spec("A", Status::Active);
        a.scope = vec!["api".to_string(), "web".to_string()];
        let mut b = spec("B", Status::Active);
        b.scope = vec!["web".to_string()];
        let c = spec("C", Status::Active);

        assert!(a.scope_selected(&["api".to_string()]));
        assert!(!b.scope_selected(&["api".to_string()]));
        assert!(c.scope_selected(&[]));
    }

    #[test]
    fn active_gaps_ignores_in_progress_gaps() {
        let active_spec = spec("A", Status::Active);
        let in_progress_spec = spec("B", Status::InProgress);
        let active = row(&active_spec, true, Vec::new());
        let in_progress = row(&in_progress_spec, true, Vec::new());

        assert_eq!(active_gaps(&[active, in_progress]), 1);
    }

    fn rendered(width: usize, verbose: bool, specs: &[Spec]) -> String {
        let rows = SpecRow::all(specs, &Found::new());
        render(
            &Style::plain(),
            width,
            verbose,
            Path::new("/workspace"),
            &rows,
        )
    }

    fn gapped_spec() -> Spec {
        let mut spec = spec("api-thing", Status::Active);
        spec.description = "Does the thing.".to_string();
        spec.scope = vec!["api".to_string()];
        spec.acceptance_criteria = vec![AcceptanceCriterion {
            statement: Statement::Must(
                "do the first thing in a sentence long enough to wrap".to_string(),
            ),
            ..testing::criterion("ac-1")
        }];
        spec
    }

    #[test]
    fn render_lists_a_gapped_spec_with_what_is_missing() {
        let text = rendered(100, false, &[gapped_spec()]);

        assert_eq!(
            text,
            format!(
                "spex {}\n\nSpecs (1)\n  active (1)\n\nGaps (1)\n  api\n    ✗  api-thing  0/1 AC  missing: contract, impl(ac-1), test(ac-1)\n\nplanned (0), in-progress (0), active (1), gaps (1)\n",
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
                "        ac-1 (missing impl & test): do the first thing\n{pad}in a sentence long\n{pad}enough to wrap\n"
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
