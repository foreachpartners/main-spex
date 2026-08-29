#[cfg(test)]
mod tests {
    use super::super::*;

    fn spec(criteria: &[&str]) -> Spec {
        Spec {
            id: "A".to_string(),
            acceptance_criteria: criteria.iter().map(|c| c.to_string()).collect(),
            blocked_by: Vec::new(),
            scope: Vec::new(),
            status: Status::Active,
            coverage_ignore: Vec::new(),
        }
    }

    #[test]
    fn ac_counts_counts_checked_criteria() {
        // Only lowercase `[x]` counts as implemented; `[X]` does not.
        let spec = spec(&["[x] MUST do a", "[ ] MUST do b", "[X] MUST do c"]);
        assert_eq!(spec.ac_counts(), (1, 3));
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
