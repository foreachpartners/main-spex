#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn parse_reads_the_spec_and_acceptance_criterion_ids() {
        let annotations = parse("// @spec impl delivery-time#step-one\n");

        assert_eq!(annotations.len(), 1);
        assert_eq!(annotations[0].spec_id.as_str(), "delivery-time");
        assert_eq!(
            annotations[0].ac_id.as_ref().map(AcId::as_str),
            Some("step-one")
        );
    }

    #[test]
    fn parse_skips_an_id_with_digits_instead_of_cutting_it_short() {
        assert!(parse("// @spec impl delivery-time2\n").is_empty());
    }

    #[test]
    fn parse_skips_an_id_over_the_maximum_length() {
        let content = format!("// @spec impl {}\n", "a".repeat(41));

        assert!(parse(&content).is_empty());
    }

    #[test]
    fn parse_keeps_the_valid_ids_beside_an_invalid_one() {
        let annotations = parse("// @spec impl alpha, delivery-time2, beta#step-one\n");

        let spec_ids: Vec<&str> = annotations
            .iter()
            .map(|annotation| annotation.spec_id.as_str())
            .collect();
        assert_eq!(spec_ids, vec!["alpha", "beta"]);
    }

    #[test]
    fn malformed_lines_reports_lines_with_an_unparseable_annotation() {
        let content = "// @spec impl a\n// @spec bogus a\nlet spec = 1;\n// @spec test\n";

        assert_eq!(malformed_lines(content), vec![2, 4]);
    }

    #[test]
    fn malformed_lines_reports_an_id_that_breaks_the_id_grammar() {
        let long_id = "a".repeat(41);
        let content = format!(
            "// @spec impl delivery-time2\n// @spec impl a#step-1\n// @spec impl {long_id}\n// @spec impl a#step-one\n"
        );

        assert_eq!(malformed_lines(&content), vec![1, 2, 3]);
    }
}
