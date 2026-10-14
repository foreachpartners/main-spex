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

    fn in_packages(id: &str, packages: &[&str]) -> Spec {
        Spec {
            packages: packages.iter().map(|name| name.to_string()).collect(),
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
    fn gapped_by_package_duplicates_multi_package_specs_and_lists_specs_without_packages_last() {
        let specs = vec![
            in_packages("a", &["web", "api"]),
            in_packages("b", &["web"]),
            in_packages("c", &[]),
        ];
        let report = report(&specs);
        let groups = report.gapped_by_package();

        let packages: Vec<Option<&str>> = groups.iter().map(|(package, _)| *package).collect();
        assert_eq!(packages, vec![Some("api"), Some("web"), None]);

        let grouped_ids: Vec<Vec<&str>> = groups.iter().map(|(_, group)| ids(group)).collect();
        assert_eq!(grouped_ids, vec![vec!["a"], vec!["a", "b"], vec!["c"]]);
    }

    #[test]
    fn gapped_includes_active_and_in_progress_specs() {
        let specs = vec![spec("a", Status::Active), spec("b", Status::InProgress)];
        let report = report(&specs);
        assert_eq!(ids(&report.gapped()), vec!["a", "b"]);
    }

    #[test]
    fn exit_code_fails_when_an_active_or_in_progress_spec_has_a_gap() {
        for status in [Status::Active, Status::InProgress] {
            let gapped = vec![spec("a", status)];
            assert_eq!(report(&gapped).exit_code(), ExitCode::FAILURE);
        }
    }

    #[test]
    fn exit_code_succeeds_without_a_gap() {
        let no_gap = vec![
            Spec {
                contract: crate::spec::ContractLevel::None,
                ..spec("a", Status::InProgress)
            },
            spec("b", Status::Planned),
        ];
        assert_eq!(report(&no_gap).exit_code(), ExitCode::SUCCESS);
    }
}
