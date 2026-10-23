#[cfg(test)]
mod tests {
    use super::super::*;
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
}
