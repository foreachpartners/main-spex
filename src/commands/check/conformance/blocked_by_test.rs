#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::rule::Severity;
    use crate::testing;

    #[test]
    fn validate_blocked_by_flags_missing_and_active_references() {
        let status_by_id = HashMap::from([
            ("DONE", Status::Active),
            ("PENDING", Status::Planned),
            ("STILL-PENDING", Status::Blocked),
        ]);
        let spec = Spec {
            blocked_by: ["MISSING", "DONE", "PENDING", "STILL-PENDING"]
                .map(str::to_string)
                .to_vec(),
            ..testing::spec("A")
        };

        let findings = validate_blocked_by(&spec, &status_by_id);

        assert_eq!(findings.len(), 2);

        let missing = &findings[0];
        assert_eq!(missing.severity, Severity::Error);
        assert_eq!(missing.check, "blocked-by");
        assert!(missing.message.contains("MISSING"));
        assert!(missing.message.contains("non-existing spec"));

        let active = &findings[1];
        assert_eq!(active.severity, Severity::Warn);
        assert!(active.message.contains("DONE"));
        assert!(active.message.contains("already active"));
    }

    #[test]
    fn status_by_id_counts_every_rule_as_active() {
        let specs = [Spec {
            status: Status::Blocked,
            ..testing::spec("a-spec")
        }];
        let rules = [testing::rule("a-rule")];

        let status_by_id = status_by_id(&specs, &rules);

        assert_eq!(status_by_id.get("a-spec"), Some(&Status::Blocked));
        assert_eq!(status_by_id.get("a-rule"), Some(&Status::Active));
    }
}
