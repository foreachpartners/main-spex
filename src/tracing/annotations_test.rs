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
    fn parse_reads_ids_with_digits() {
        let annotations = parse("// @spec impl oauth2-login#2fa-step, s3-upload\n");

        assert_eq!(annotations.len(), 2);
        assert_eq!(annotations[0].spec_id.as_str(), "oauth2-login");
        assert_eq!(
            annotations[0].ac_id.as_ref().map(AcId::as_str),
            Some("2fa-step")
        );
        assert_eq!(annotations[1].spec_id.as_str(), "s3-upload");
    }

    #[test]
    fn parse_skips_an_id_with_an_uppercase_letter_instead_of_cutting_it_short() {
        assert!(parse("// @spec impl delivery-timeX\n").is_empty());
    }

    #[test]
    fn parse_skips_an_id_over_the_maximum_length() {
        let content = format!("// @spec impl {}\n", "a".repeat(41));

        assert!(parse(&content).is_empty());
    }

    #[test]
    fn parse_skips_the_whole_annotation_when_a_listed_id_is_invalid() {
        let content = "// @spec impl alpha, Delivery-Time, beta#step-one\n";

        assert!(parse(content).is_empty());
        assert_eq!(malformed_lines(content), vec![1]);
    }

    #[test]
    fn malformed_lines_reports_an_id_that_continues_past_its_valid_part() {
        let content =
            "// @spec impl a_b\n// @spec impl a, B\n// @spec impl a#b#c\n// @spec impl aB\n";

        assert_eq!(malformed_lines(content), vec![1, 2, 3, 4]);
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
            "// @spec impl delivery-timeX\n// @spec impl a#Step\n// @spec impl {long_id}\n// @spec impl a#step-one\n"
        );

        assert_eq!(malformed_lines(&content), vec![1, 2, 3]);
    }
}
