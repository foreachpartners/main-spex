#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::rule::Severity;
    use crate::support::testing;

    fn spec_with_blockers(blockers: &[&str]) -> Spec {
        Spec {
            blocked_by: blockers.iter().map(|id| testing::spec_id(id)).collect(),
            ..testing::spec("a")
        }
    }

    fn spec_with_status(id: &str, status: Status) -> Spec {
        Spec {
            status,
            ..testing::spec(id)
        }
    }

    #[test]
    fn validate_blocked_by_flags_missing_and_active_references() {
        let specs = [
            spec_with_status("done", Status::Active),
            spec_with_status("pending", Status::Planned),
            spec_with_status("still-pending", Status::Blocked),
        ];
        let spec = spec_with_blockers(&["missing", "done", "pending", "still-pending"]);

        let findings = validate_blocked_by(&spec, &status_by_id(&specs));

        assert_eq!(findings.len(), 2);

        let missing = &findings[0];
        assert_eq!(missing.severity, Severity::Error);
        assert_eq!(missing.check, "blocked-by");
        assert!(missing.message.contains("missing"));
        assert!(missing.message.contains("non-existing spec"));

        let active = &findings[1];
        assert_eq!(active.severity, Severity::Warn);
        assert!(active.message.contains("done"));
        assert!(active.message.contains("already active"));
    }

    #[test]
    fn validate_blocked_by_flags_a_rule_id_as_a_non_existing_spec() {
        let specs = [testing::spec("a-spec")];
        let spec = spec_with_blockers(&["a-rule"]);

        let findings = validate_blocked_by(&spec, &status_by_id(&specs));

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Error);
        assert!(findings[0].message.contains("a-rule"));
        assert!(findings[0].message.contains("non-existing spec"));
    }

    #[test]
    fn status_by_id_maps_each_spec_to_its_status() {
        let specs = [
            spec_with_status("a-spec", Status::Blocked),
            spec_with_status("b-spec", Status::Active),
        ];

        let status_by_id = status_by_id(&specs);

        assert_eq!(
            status_by_id.get(&testing::spec_id("a-spec")),
            Some(&Status::Blocked)
        );
        assert_eq!(
            status_by_id.get(&testing::spec_id("b-spec")),
            Some(&Status::Active)
        );
    }
}
