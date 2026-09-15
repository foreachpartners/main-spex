#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::annotations::AcCoverage;
    use crate::spec::{CoverageLayer, Spec, Status};

    fn spec(id: &str, status: Status) -> Spec {
        Spec {
            id: id.to_string(),
            path: format!("{id}.yaml"),
            acceptance_criteria: Vec::new(),
            blocked_by: Vec::new(),
            scope: Vec::new(),
            status,
            coverage_ignore: Vec::new(),
        }
    }

    fn ac(id: &str, impl_present: bool, test_present: bool) -> AcCoverage {
        AcCoverage {
            id: id.to_string(),
            impl_required: true,
            impl_present,
            test_required: true,
            test_present,
        }
    }

    fn row(
        id: &str,
        contract_required: bool,
        contract_present: bool,
        ac_rows: Vec<AcCoverage>,
    ) -> SpecRow {
        SpecRow {
            id: id.to_string(),
            status: Status::Active,
            scope: Vec::new(),
            contract_required,
            contract_present,
            ac_rows,
        }
    }

    #[test]
    fn tracked_rows_includes_active_and_in_progress_and_ignores_the_rest() {
        let specs = vec![
            spec("A", Status::Active),
            spec("B", Status::Planned),
            spec("C", Status::Deprecated),
            spec("D", Status::InProgress),
        ];
        let found = Found::new();
        let rows = tracked_rows(&specs, &found);
        let ids: Vec<&str> = rows.iter().map(|row| row.id.as_str()).collect();
        assert_eq!(ids, vec!["A", "D"]);
    }

    #[test]
    fn omitting_status_defaults_to_planned_and_is_ignored() {
        let spec: Spec = serde_yaml_ng::from_str("id: A\n").expect("parse spec");
        assert_eq!(spec.status, Status::Planned);
        assert!(spec.coverage_ignore.is_empty());
        let rows = tracked_rows(&[spec], &Found::new());
        assert!(rows.is_empty());
    }

    #[test]
    fn tracked_rows_carries_coverage_ignore() {
        let spec: Spec = serde_yaml_ng::from_str(
            "id: A\nstatus: active\ncoverage_ignore:\n  - layer: contract\n  - layer: test\n",
        )
        .expect("parse spec");
        assert!(spec.layer_ignored(CoverageLayer::Contract));
        assert!(spec.layer_ignored(CoverageLayer::Test));
        assert!(!spec.layer_ignored(CoverageLayer::Impl));
        let rows = tracked_rows(&[spec], &Found::new());
        assert!(!rows[0].contract_required);
    }

    #[test]
    fn tracked_rows_parses_per_ac_ignore() {
        let spec: Spec = serde_yaml_ng::from_str(
            "id: A\nstatus: active\nacceptance_criteria:\n  - id: ac-1\n    must: \"do a\"\ncoverage_ignore:\n  - layer: impl\n    ac: ac-1\n",
        )
        .expect("parse spec");
        let rows = tracked_rows(&[spec], &Found::new());
        assert!(!rows[0].ac_rows[0].impl_required);
        assert!(rows[0].ac_rows[0].test_required);
    }

    #[test]
    fn missing_summary_lists_contract_then_impl_then_test() {
        let row = row(
            "A",
            true,
            false,
            vec![ac("ac-1", false, false), ac("ac-2", true, false)],
        );
        assert_eq!(
            missing_summary(&row),
            "contract, impl(ac-1), test(ac-1, ac-2)"
        );
    }

    #[test]
    fn missing_contract_is_a_gap_by_default() {
        let row = row("A", true, false, vec![ac("ac-1", true, true)]);
        assert!(row.has_gap());
        assert_eq!(missing_summary(&row), "contract");
    }

    #[test]
    fn ignored_contract_is_not_a_gap() {
        let row = row("A", false, false, vec![ac("ac-1", true, true)]);
        assert!(!row.has_gap());
        assert_eq!(missing_summary(&row), "");
    }

    #[test]
    fn unimplemented_ac_is_a_gap() {
        let row = row(
            "A",
            true,
            true,
            vec![ac("ac-1", true, true), ac("ac-2", false, true)],
        );
        assert!(row.has_gap());
        assert_eq!(row.ac_done(), 1);
        assert_eq!(row.ac_total(), 2);
    }

    #[test]
    fn all_layers_and_ac_complete_has_no_gap() {
        let row = row(
            "A",
            true,
            true,
            vec![ac("ac-1", true, true), ac("ac-2", true, true)],
        );
        assert!(!row.has_gap());
    }

    #[test]
    fn status_groups_lists_ids_per_non_empty_status_in_order() {
        let specs = vec![
            spec("A", Status::Active),
            spec("B", Status::Planned),
            spec("C", Status::Planned),
            spec("D", Status::Deprecated),
        ];
        let groups = status_groups(&specs);
        assert_eq!(
            groups,
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
        let groups = status_groups(&specs);
        assert_eq!(groups, vec![(Status::Planned, vec!["A"])]);
    }

    #[test]
    fn scope_groups_duplicates_multi_scope_rows_and_lists_unscoped_last() {
        let mut a = row("A", true, true, Vec::new());
        a.scope = vec!["web".to_string(), "api".to_string()];
        let mut b = row("B", true, true, Vec::new());
        b.scope = vec!["web".to_string()];
        let c = row("C", true, true, Vec::new());

        let rows = [&a, &b, &c];
        let groups = scope_groups(&rows);

        let scopes: Vec<Option<&str>> = groups.iter().map(|(scope, _)| *scope).collect();
        assert_eq!(scopes, vec![Some("api"), Some("web"), None]);

        let ids: Vec<Vec<&str>> = groups
            .iter()
            .map(|(_, rows)| rows.iter().map(|row| row.id.as_str()).collect())
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

        assert!(scope_selected(&a, &["api".to_string()]));
        assert!(!scope_selected(&b, &["api".to_string()]));
        assert!(scope_selected(&c, &[]));
    }

    #[test]
    fn active_gaps_ignores_in_progress_gaps() {
        let mut active = row("A", true, false, Vec::new());
        active.status = Status::Active;

        let mut in_progress = row("B", true, false, Vec::new());
        in_progress.status = Status::InProgress;

        assert_eq!(active_gaps(&[active, in_progress]), 1);
    }
}
