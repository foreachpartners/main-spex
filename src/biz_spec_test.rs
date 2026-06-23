#[cfg(test)]
mod tests {
    use super::super::*;

    fn spec(criteria: &[&str]) -> BizSpec {
        BizSpec {
            id: "BIZ-A".to_string(),
            title: String::new(),
            description: String::new(),
            acceptance_criteria: criteria.iter().map(|c| c.to_string()).collect(),
            status: Status::Active,
            coverage_ignore: Vec::new(),
        }
    }

    #[test]
    fn ac_counts_counts_checked_criteria() {
        let spec = spec(&["[x] MUST do a", "[ ] MUST do b", "[X] MUST do c"]);
        assert_eq!(spec.ac_counts(), (2, 3));
    }

    #[test]
    fn ac_counts_treats_missing_checkbox_as_unimplemented() {
        let spec = spec(&["MUST do a", "[x] MUST do b"]);
        assert_eq!(spec.ac_counts(), (1, 2));
    }

    #[test]
    fn ac_counts_with_no_criteria_is_zero() {
        let spec = spec(&[]);
        assert_eq!(spec.ac_counts(), (0, 0));
    }
}
