#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::support::testing;

    fn spec(id: &str, status: Status) -> Spec {
        Spec {
            status,
            ..testing::spec(id)
        }
    }

    fn scoped(id: &str, scope: &[&str]) -> Spec {
        Spec {
            scope: scope.iter().map(|name| name.to_string()).collect(),
            ..spec(id, Status::Active)
        }
    }

    fn report(specs: &[Spec]) -> CoverageReport<'_> {
        CoverageReport::new(specs, &AnnotationIndex::new())
    }

    fn ids<'a>(coverages: &[&SpecCoverage<'a>]) -> Vec<&'a str> {
        coverages
            .iter()
            .map(|coverage| coverage.spec.id.as_str())
            .collect()
    }

    #[test]
    fn report_keeps_one_coverage_per_spec_in_order() {
        let specs = vec![
            spec("a", Status::Active),
            spec("b", Status::Planned),
            spec("c", Status::InProgress),
        ];
        let report = report(&specs);
        let order: Vec<&str> = report
            .specs
            .iter()
            .map(|coverage| coverage.spec.id.as_str())
            .collect();
        assert_eq!(order, vec!["a", "b", "c"]);
    }

    #[test]
    fn count_tallies_specs_per_status() {
        let specs = vec![
            spec("a", Status::Active),
            spec("b", Status::Planned),
            spec("c", Status::Planned),
        ];
        let report = report(&specs);
        assert_eq!(report.count(Status::Planned), 2);
        assert_eq!(report.count(Status::Active), 1);
        assert_eq!(report.count(Status::Deprecated), 0);
    }

    #[test]
    fn has_tracked_specs_needs_an_active_or_in_progress_spec() {
        let planned_only = vec![spec("a", Status::Planned)];
        assert!(!report(&planned_only).has_tracked_specs());

        let with_in_progress = vec![spec("a", Status::Planned), spec("b", Status::InProgress)];
        assert!(report(&with_in_progress).has_tracked_specs());
    }

    #[test]
    fn status_groups_lists_specs_per_non_empty_status_in_order() {
        let specs = vec![
            spec("a", Status::Active),
            spec("b", Status::Planned),
            spec("c", Status::Planned),
            spec("d", Status::Deprecated),
        ];
        let report = report(&specs);
        let groups: Vec<(Status, Vec<&str>)> = report
            .status_groups()
            .iter()
            .map(|(status, group)| (*status, ids(group)))
            .collect();
        assert_eq!(
            groups,
            vec![
                (Status::Active, vec!["a"]),
                (Status::Planned, vec!["b", "c"]),
                (Status::Deprecated, vec!["d"]),
            ]
        );
    }

    #[test]
    fn status_groups_skips_empty_statuses() {
        let specs = vec![spec("a", Status::Planned)];
        let report = report(&specs);
        let groups: Vec<(Status, Vec<&str>)> = report
            .status_groups()
            .iter()
            .map(|(status, group)| (*status, ids(group)))
            .collect();
        assert_eq!(groups, vec![(Status::Planned, vec!["a"])]);
    }

    #[test]
    fn gapped_lists_only_specs_with_a_gap() {
        let specs = vec![
            spec("a", Status::Active),
            spec("b", Status::Planned),
            Spec {
                contract: crate::spec::ContractLevel::None,
                ..spec("c", Status::Active)
            },
        ];
        let report = report(&specs);
        assert_eq!(ids(&report.gapped()), vec!["a"]);
    }

    #[test]
    fn gapped_by_scope_duplicates_multi_scope_specs_and_lists_unscoped_last() {
        let specs = vec![
            scoped("a", &["web", "api"]),
            scoped("b", &["web"]),
            scoped("c", &[]),
        ];
        let report = report(&specs);
        let groups = report.gapped_by_scope();

        let scopes: Vec<Option<&str>> = groups.iter().map(|(scope, _)| *scope).collect();
        assert_eq!(scopes, vec![Some("api"), Some("web"), None]);

        let grouped_ids: Vec<Vec<&str>> = groups.iter().map(|(_, group)| ids(group)).collect();
        assert_eq!(grouped_ids, vec![vec!["a"], vec!["a", "b"], vec!["c"]]);
    }

    #[test]
    fn active_gap_count_ignores_in_progress_gaps() {
        let specs = vec![spec("a", Status::Active), spec("b", Status::InProgress)];
        let report = report(&specs);
        assert_eq!(report.gapped().len(), 2);
        assert_eq!(report.active_gap_count(), 1);
    }

    #[test]
    fn exit_code_fails_only_when_an_active_spec_has_a_gap() {
        let in_progress_gap = vec![spec("a", Status::InProgress)];
        assert_eq!(report(&in_progress_gap).exit_code(), ExitCode::SUCCESS);

        let active_gap = vec![spec("a", Status::Active)];
        assert_eq!(report(&active_gap).exit_code(), ExitCode::FAILURE);
    }
}
