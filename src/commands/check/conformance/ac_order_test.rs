#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::commands::check::conformance::RULE_ID;
    use crate::spec;
    use indoc::indoc;
    use tempfile::tempdir;

    const OUT_OF_ORDER_SUFFIX: &str = "(order: MUST, MUST NOT, SHOULD, SHOULD NOT, MAY)";

    fn rebased(findings: Vec<Finding>, base: &Path) -> Vec<Finding> {
        findings
            .into_iter()
            .map(|finding| finding.relative_to(base))
            .collect()
    }

    fn spec_findings(yaml: &str) -> Vec<Finding> {
        let dir = tempdir().expect("temp dir");
        let specs_dir = dir.path().join(spec::SPECS_DIR);
        std::fs::create_dir_all(&specs_dir).expect("specs dir");
        std::fs::write(specs_dir.join("sample.yaml"), yaml).expect("spec");

        let specs = Spec::load_all(&specs_dir).expect("specs");
        rebased(validate_ac_order(&specs, &[]), dir.path())
    }

    fn rule_findings(yaml: &str) -> Vec<Finding> {
        let dir = tempdir().expect("temp dir");
        let rules_dir = dir.path().join(crate::rule::RULES_DIR);
        std::fs::create_dir_all(&rules_dir).expect("rules dir");
        std::fs::write(rules_dir.join("sample.yaml"), yaml).expect("rule");

        let rules = Rule::load_all(&rules_dir).expect("rules");
        rebased(validate_ac_order(&[], &rules), dir.path())
    }

    fn assert_out_of_order(finding: &Finding, location: &str, label: &str, prev_label: &str) {
        assert_eq!(
            finding.source.baseline_key(),
            Some((RULE_ID, AC_ORDER_CHECK))
        );
        assert_eq!(finding.location, Some(Location::parse(location)));
        assert_eq!(
            finding.message,
            format!(
                "acceptance criteria out of order: {label} must not follow {prev_label} {OUT_OF_ORDER_SUFFIX}"
            )
        );
    }

    #[test]
    fn spec_in_commitment_order_passes() {
        let findings = spec_findings(indoc! {"
            id: sample
            contract: none
            acceptance_criteria:
              - id: a
                must: a
              - id: b
                must_not: b
              - id: c
                should: c
              - id: d
                should_not: d
              - id: e
                may: e
        "});

        assert!(findings.is_empty());
    }

    #[test]
    fn spec_repeating_one_commitment_passes() {
        let findings = spec_findings(indoc! {"
            id: sample
            contract: none
            acceptance_criteria:
              - id: a
                must: a
              - id: b
                must: b
        "});

        assert!(findings.is_empty());
    }

    #[test]
    fn spec_flags_the_criterion_that_breaks_the_order() {
        let findings = spec_findings(indoc! {"
            id: sample
            contract: none
            acceptance_criteria:
              - id: first
                should: a
              - id: second
                must: b
        "});

        assert_eq!(findings.len(), 1);
        assert_out_of_order(&findings[0], "specs/sample.yaml:6", "MUST", "SHOULD");
    }

    #[test]
    fn spec_flags_every_criterion_that_breaks_the_order() {
        let findings = spec_findings(indoc! {"
            id: sample
            contract: none
            acceptance_criteria:
              - id: a
                may: a
              - id: b
                should: b
              - id: c
                must: c
        "});

        assert_eq!(findings.len(), 2);
        assert_out_of_order(&findings[0], "specs/sample.yaml:6", "SHOULD", "MAY");
        assert_out_of_order(&findings[1], "specs/sample.yaml:8", "MUST", "SHOULD");
    }

    #[test]
    fn rule_in_commitment_order_passes() {
        let findings = rule_findings(indoc! {"
            id: sample
            acceptance_criteria:
              - MUST a
              - MUST NOT b
              - SHOULD c
              - SHOULD NOT d
              - MAY e
        "});

        assert!(findings.is_empty());
    }

    #[test]
    fn rule_flags_the_criterion_that_breaks_the_order() {
        let findings = rule_findings(indoc! {"
            id: sample
            acceptance_criteria:
              - SHOULD a
              - MUST b
        "});

        assert_eq!(findings.len(), 1);
        assert_out_of_order(&findings[0], "rules/sample.yaml:4", "MUST", "SHOULD");
    }

    #[test]
    fn rule_flags_every_criterion_that_breaks_the_order() {
        let findings = rule_findings(indoc! {"
            id: sample
            acceptance_criteria:
              - MAY a
              - SHOULD b
              - MUST c
        "});

        assert_eq!(findings.len(), 2);
        assert_out_of_order(&findings[0], "rules/sample.yaml:4", "SHOULD", "MAY");
        assert_out_of_order(&findings[1], "rules/sample.yaml:5", "MUST", "SHOULD");
    }

    #[test]
    fn rule_criteria_without_a_commitment_are_skipped_and_do_not_reset_the_order() {
        let findings = rule_findings(indoc! {"
            id: sample
            acceptance_criteria:
              - SHOULD a
              - plain statement
              - MUST c
        "});

        assert_eq!(findings.len(), 1);
        assert_out_of_order(&findings[0], "rules/sample.yaml:5", "MUST", "SHOULD");
    }

    #[test]
    fn rule_must_not_is_read_as_its_own_commitment_not_as_must() {
        let in_order = rule_findings(indoc! {"
            id: sample
            acceptance_criteria:
              - MUST a
              - MUST NOT b
        "});
        assert!(in_order.is_empty());

        let out_of_order = rule_findings(indoc! {"
            id: sample
            acceptance_criteria:
              - MUST NOT a
              - MUST b
        "});
        assert_eq!(out_of_order.len(), 1);
        assert_out_of_order(&out_of_order[0], "rules/sample.yaml:4", "MUST", "MUST NOT");
    }
}
