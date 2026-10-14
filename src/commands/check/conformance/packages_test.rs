#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::rule::Severity;
    use crate::support::testing;

    #[test]
    fn validate_packages_flags_unknown_package_short_name() {
        let spec = Spec {
            packages: vec!["api".to_string(), "unknown".to_string()],
            ..testing::spec("a")
        };
        let packages = vec![testing::package("api", "../api")];

        let findings = validate_packages(&spec, &packages);

        assert_eq!(findings.len(), 1);
        let finding = &findings[0];
        assert_eq!(finding.severity, Severity::Error);
        assert!(matches!(&finding.source, Source::Spec { check, .. } if check == "packages"));
        assert!(finding.message.contains("unknown"));
        assert!(finding.message.contains("package_short_name"));
    }
}
