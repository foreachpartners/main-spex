#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn parse_keeps_ids_with_digits_whole() {
        let annotations = parse("// @spec impl delivery-time2#step-1\n");

        assert_eq!(annotations.len(), 1);
        assert_eq!(annotations[0].spec_id, "delivery-time2");
        assert_eq!(annotations[0].ac_id.as_deref(), Some("step-1"));
    }

    #[test]
    fn malformed_lines_reports_lines_with_an_unparseable_annotation() {
        let content = "// @spec impl a\n// @spec bogus a\nlet spec = 1;\n// @spec test\n";

        assert_eq!(malformed_lines(content), vec![2, 4]);
    }
}
