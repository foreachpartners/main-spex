#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::rule::Severity;
    use crate::testing;

    #[test]
    fn validate_scope_flags_unknown_repository_short_name() {
        let spec = Spec {
            scope: vec!["api".to_string(), "unknown".to_string()],
            ..testing::spec("A")
        };
        let repositories = vec![testing::repository("api", "../api")];

        let findings = validate_scope(&spec, &repositories);

        assert_eq!(findings.len(), 1);
        let finding = &findings[0];
        assert_eq!(finding.severity, Severity::Error);
        assert_eq!(finding.check, "scope");
        assert!(finding.message.contains("unknown"));
        assert!(finding.message.contains("repository_short_name"));
    }
}
