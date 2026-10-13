#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn parse_splits_a_trailing_line_number_from_the_file() {
        assert_eq!(
            Location::parse("src/main.rs:42"),
            Location::at_line("src/main.rs", 42)
        );
        assert_eq!(
            Location::parse("src/main.rs"),
            Location::in_file("src/main.rs")
        );
    }

    #[test]
    fn parse_keeps_a_non_numeric_suffix_in_the_file() {
        assert_eq!(Location::parse("a:b/c.rs"), Location::in_file("a:b/c.rs"));
        assert_eq!(Location::parse("a.rs:"), Location::in_file("a.rs:"));
    }

    #[test]
    fn a_location_displays_as_file_and_optional_line() {
        assert_eq!(Location::in_file("src/main.rs").to_string(), "src/main.rs");
        assert_eq!(
            Location::at_line("src/main.rs", 42).to_string(),
            "src/main.rs:42"
        );
    }

    #[test]
    fn relative_to_strips_the_base_and_keeps_the_line() {
        assert_eq!(
            Location::at_line("/spex/specs/a.yaml", 3).relative_to(Path::new("/spex")),
            Location::at_line("specs/a.yaml", 3)
        );
    }

    #[test]
    fn relative_to_keeps_a_file_outside_the_base() {
        assert_eq!(
            Location::in_file("/elsewhere/a.yaml").relative_to(Path::new("/spex")),
            Location::in_file("/elsewhere/a.yaml")
        );
    }

    #[test]
    fn a_finding_rebases_its_location() {
        let finding = Finding::error(
            Source::rule("x", "check"),
            Some(Location::in_file("/spex/specs/a.yaml")),
            "boom".to_string(),
        )
        .relative_to(Path::new("/spex"));

        assert_eq!(finding.location, Some(Location::in_file("specs/a.yaml")));
    }

    #[test]
    fn a_check_source_displays_as_owner_and_check() {
        assert_eq!(
            Source::rule("writing", "no-cyrillic").to_string(),
            "writing/no-cyrillic"
        );
        assert_eq!(
            Source::spec("place-order", "status").to_string(),
            "place-order/status"
        );
        assert_eq!(
            Source::internal("spex-internal-conformance", "ac-ordered").to_string(),
            "spex-internal-conformance/ac-ordered"
        );
    }

    #[test]
    fn a_source_without_a_check_displays_its_label_alone() {
        let unknown_rule = Source::UnknownRule {
            rule_id: "writing".to_string(),
        };
        let unreachable = Source::UnreachablePackage {
            name: "proj-api".to_string(),
        };

        assert_eq!(unknown_rule.to_string(), "writing");
        assert_eq!(unreachable.to_string(), "proj-api");
    }

    #[test]
    fn only_check_sources_have_a_baseline_key() {
        assert_eq!(
            Source::rule("writing", "no-cyrillic").baseline_key(),
            Some(("writing", "no-cyrillic"))
        );
        assert_eq!(
            Source::spec("place-order", "status").baseline_key(),
            Some(("place-order", "status"))
        );
        assert_eq!(
            Source::internal("spex-internal-conformance", "ac-ordered").baseline_key(),
            Some(("spex-internal-conformance", "ac-ordered"))
        );
        assert_eq!(
            Source::UnknownRule {
                rule_id: "writing".to_string()
            }
            .baseline_key(),
            None
        );
        assert_eq!(
            Source::UnreachablePackage {
                name: "proj-api".to_string()
            }
            .baseline_key(),
            None
        );
    }
}
