#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::rule::Severity;
    use crate::spec::AcceptanceCriterion;
    use crate::support::testing;
    use strum::IntoEnumIterator;

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum Completeness {
        Complete,
        Partial,
        NotStarted,
    }

    const ALL_COMPLETENESS: [Completeness; 3] = [
        Completeness::Complete,
        Completeness::Partial,
        Completeness::NotStarted,
    ];

    fn criteria_of(completeness: Completeness) -> Vec<AcceptanceCriterion> {
        match completeness {
            Completeness::Complete => criteria(&["a", "b"], &[]),
            Completeness::Partial => criteria(&["a", "b"], &["b"]),
            Completeness::NotStarted => criteria(&["a", "b"], &["a", "b"]),
        }
    }

    fn severities(spec: &Spec) -> Vec<Severity> {
        [status_finding(spec), active_incomplete_finding(spec)]
            .into_iter()
            .flatten()
            .map(|finding| finding.severity)
            .collect()
    }

    fn expected_severities(status: Status, completeness: Completeness) -> Vec<Severity> {
        use Completeness::*;
        let table: &[(Status, Completeness, &[Severity])] = &[
            (Status::Planned, Complete, &[Severity::Warn]),
            (Status::Planned, Partial, &[]),
            (Status::Planned, NotStarted, &[]),
            (Status::InProgress, Complete, &[Severity::Warn]),
            (Status::InProgress, Partial, &[]),
            (Status::InProgress, NotStarted, &[]),
            (Status::Blocked, Complete, &[Severity::Warn]),
            (Status::Blocked, Partial, &[]),
            (Status::Blocked, NotStarted, &[]),
            (Status::Active, Complete, &[]),
            (Status::Active, Partial, &[Severity::Error]),
            (Status::Active, NotStarted, &[Severity::Error]),
            (Status::Deprecated, Complete, &[]),
            (Status::Deprecated, Partial, &[]),
            (Status::Deprecated, NotStarted, &[]),
        ];
        table
            .iter()
            .find(|(row_status, row_completeness, _)| {
                *row_status == status && *row_completeness == completeness
            })
            .unwrap_or_else(|| panic!("no expectation for {status:?} with {completeness:?}"))
            .2
            .to_vec()
    }

    fn criteria(ids: &[&str], not_done: &[&str]) -> Vec<AcceptanceCriterion> {
        ids.iter()
            .map(|id| AcceptanceCriterion {
                is_done: !not_done.contains(id),
                ..testing::criterion(id)
            })
            .collect()
    }

    fn spec(id: &str, status: Status, criteria: Vec<AcceptanceCriterion>) -> Spec {
        Spec {
            acceptance_criteria: criteria,
            status,
            ..testing::spec(id)
        }
    }

    #[test]
    fn every_status_and_completeness_pair_reports_the_expected_severities() {
        for status in Status::iter() {
            for completeness in ALL_COMPLETENESS {
                let spec = spec("a", status, criteria_of(completeness));
                assert_eq!(
                    severities(&spec),
                    expected_severities(status, completeness),
                    "{status:?} with {completeness:?}"
                );
            }
        }
    }

    #[test]
    fn status_finding_flags_fully_done_unimplemented_spec() {
        let spec = spec("a", Status::Planned, criteria(&["a", "b"], &[]));
        let result = status_finding(&spec);
        let finding = result.expect("expected a status finding");
        assert_eq!(finding.severity, Severity::Warn);
        assert!(matches!(&finding.source, Source::Spec { check, .. } if check == "status"));
        assert_eq!(
            finding.location,
            Some(Location::in_file(testing::spex_dir().join("specs/a.yaml")))
        );
    }

    #[test]
    fn status_finding_is_silent_when_consistent_or_incomplete() {
        let full = || criteria(&["a", "b"], &[]);
        assert!(status_finding(&spec("a", Status::Active, full())).is_none());
        assert!(status_finding(&spec("a", Status::Deprecated, full())).is_none());
        let partial = criteria(&["a", "b"], &["b"]);
        assert!(status_finding(&spec("a", Status::Planned, partial)).is_none());
    }

    #[test]
    fn active_incomplete_finding_flags_active_with_not_done_criteria() {
        let partial = criteria(&["a", "b"], &["b"]);
        let finding = active_incomplete_finding(&spec("a", Status::Active, partial))
            .expect("expected a finding");
        assert_eq!(finding.severity, Severity::Error);
        assert!(matches!(&finding.source, Source::Spec { check, .. } if check == "status"));
        assert_eq!(
            finding.location,
            Some(Location::in_file(testing::spex_dir().join("specs/a.yaml")))
        );
        assert!(finding.message.contains("1/2"));
    }

    #[test]
    fn active_incomplete_finding_is_silent_when_complete_or_not_active() {
        let full = criteria(&["a", "b"], &[]);
        assert!(active_incomplete_finding(&spec("a", Status::Active, full)).is_none());

        let partial = criteria(&["a", "b"], &["b"]);
        assert!(active_incomplete_finding(&spec("a", Status::InProgress, partial)).is_none());
    }
}
