#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::commands::check::finding::Source;
    use crate::rule::Severity;
    use crate::support::testing;

    fn package_with_rules(name: &str, rules: &[&str]) -> Package {
        Package {
            rules: rules.iter().map(|id| testing::rule_id(id)).collect(),
            ..testing::package(name, name)
        }
    }

    #[test]
    fn rule_listed_by_a_package_is_not_flagged() {
        let packages = [package_with_rules("api", &["a-rule"])];

        assert!(validate_unused_rules(&packages, &[testing::rule("a-rule")]).is_empty());
    }

    #[test]
    fn rule_listed_by_only_one_of_several_packages_is_not_flagged() {
        let packages = [
            package_with_rules("api", &[]),
            package_with_rules("frontend", &["a-rule"]),
        ];

        assert!(validate_unused_rules(&packages, &[testing::rule("a-rule")]).is_empty());
    }

    #[test]
    fn rule_listed_by_no_package_is_flagged_at_its_file_with_a_deletion_suggestion() {
        let packages = [package_with_rules("api", &["used-rule"])];
        let rules = [testing::rule("used-rule"), testing::rule("unused-rule")];

        let findings = validate_unused_rules(&packages, &rules);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Warn);
        assert!(matches!(
            &findings[0].source,
            Source::Internal { check, .. } if check == "unused-rule"
        ));
        assert_eq!(
            findings[0].location,
            Some(Location::in_file(&rules[1].path))
        );
        assert!(findings[0].message.contains("`unused-rule`"));
        assert!(findings[0].message.contains("delete"));
        assert!(findings[0].message.contains("rules_unused"));
    }

    #[test]
    fn every_rule_is_flagged_when_there_are_no_packages() {
        let rules = [testing::rule("a-rule"), testing::rule("b-rule")];

        assert_eq!(validate_unused_rules(&[], &rules).len(), 2);
    }
}
