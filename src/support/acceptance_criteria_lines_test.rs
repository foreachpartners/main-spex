#[cfg(test)]
mod tests {
    use super::super::with_source_lines;

    #[test]
    fn with_source_lines_pairs_each_list_item_with_its_source_line() {
        let raw = "id: a\nacceptance_criteria:\n  - id: one\n    must: x\n  - id: two\n    must: y\ncontract: single\n";

        let numbered = with_source_lines(vec!["one", "two"], raw, |name, line| (name, line));

        assert_eq!(numbered, vec![("one", 3), ("two", 5)]);
    }

    #[test]
    fn with_source_lines_numbers_criteria_without_a_list_item_line_as_zero() {
        let raw = "id: a\nacceptance_criteria: [x, y]\n";

        let numbered = with_source_lines(vec!["x", "y"], raw, |name, line| (name, line));

        assert_eq!(numbered, vec![("x", 0), ("y", 0)]);
    }
}
