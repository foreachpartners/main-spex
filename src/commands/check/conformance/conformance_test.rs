#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::rule::Severity;
    use crate::spec::{AcceptanceCriterion, Status};
    use crate::support::testing;
    use tempfile::tempdir;

    #[test]
    fn finding_locations_are_relative_to_the_spex_dir() {
        let spex_dir = tempdir().expect("spex dir");
        let spec = Spec {
            path: spex_dir.path().join("specs/a.yaml"),
            status: Status::Active,
            acceptance_criteria: vec![AcceptanceCriterion {
                is_done: false,
                ..testing::criterion("one")
            }],
            ..testing::spec("a")
        };

        let findings = validate_conformance(
            spex_dir.path(),
            &spex_dir.path().join("docs/prd.md"),
            &[],
            &[spec],
            &[],
        )
        .expect("conformance");

        let locations: Vec<Location> = findings
            .iter()
            .filter_map(|finding| finding.location.clone())
            .collect();
        assert!(
            locations.contains(&Location::in_file("specs/a.yaml")),
            "got: {locations:?}"
        );
        assert!(locations.iter().all(|location| location.file.is_relative()));
    }

    #[test]
    fn unused_rule_is_reported_as_a_warning_at_its_relative_file() {
        let spex_dir = tempdir().expect("spex dir");
        let rule = Rule {
            path: spex_dir.path().join("rules/a.yaml"),
            ..testing::rule("a")
        };

        let findings = validate_conformance(
            spex_dir.path(),
            &spex_dir.path().join("docs/prd.md"),
            &[],
            &[],
            &[rule],
        )
        .expect("conformance");

        let unused: Vec<&Finding> = findings
            .iter()
            .filter(|finding| {
                matches!(&finding.source, Source::Internal { check, .. } if check == "unused-rule")
            })
            .collect();
        assert_eq!(unused.len(), 1);
        assert_eq!(unused[0].severity, Severity::Warn);
        assert_eq!(unused[0].location, Some(Location::in_file("rules/a.yaml")));
    }
}
