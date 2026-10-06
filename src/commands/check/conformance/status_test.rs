#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::rule::Severity;
    use crate::spec::AcceptanceCriterion;
    use crate::support::testing;

    fn criteria(ids: &[&str], not_done: &[&str]) -> Vec<AcceptanceCriterion> {
        ids.iter()
            .map(|id| AcceptanceCriterion {
                is_done: not_done.contains(id).then_some(false),
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
    fn status_finding_flags_fully_done_unimplemented_spec() {
        let spec = spec("a", Status::Planned, criteria(&["a", "b"], &[]));
        let result = status_finding(&spec);
        let finding = result.expect("expected a status finding");
        assert_eq!(finding.severity, Severity::Warn);
        assert_eq!(finding.check, "status");
        assert_eq!(finding.location.as_deref(), Some("specs/a.yaml"));
    }

    #[test]
    fn status_finding_is_silent_when_consistent_or_incomplete() {
        let full = || criteria(&["a", "b"], &[]);
        // Fully done and already implemented: nothing to flag.
        assert!(status_finding(&spec("A", Status::Active, full())).is_none());
        assert!(status_finding(&spec("A", Status::Deprecated, full())).is_none());
        // A not-done criterion means the spec isn't done, so any status is allowed.
        let partial = criteria(&["a", "b"], &["b"]);
        assert!(status_finding(&spec("A", Status::Planned, partial)).is_none());
        // No criteria at all: nothing to infer.
        assert!(status_finding(&spec("A", Status::Planned, Vec::new())).is_none());
    }

    #[test]
    fn active_incomplete_finding_flags_active_with_not_done_criteria() {
        let partial = criteria(&["a", "b"], &["b"]);
        let finding = active_incomplete_finding(&spec("a", Status::Active, partial))
            .expect("expected a finding");
        assert_eq!(finding.severity, Severity::Error);
        assert_eq!(finding.check, "status");
        assert_eq!(finding.location.as_deref(), Some("specs/a.yaml"));
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
