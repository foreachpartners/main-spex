#[cfg(test)]
mod tests {
    use super::super::*;

    fn finding(severity: Severity, check: &str, location: Option<&str>) -> Finding {
        Finding {
            severity,
            rule_id: "X".to_string(),
            check: check.to_string(),
            location: location.map(str::to_string),
            message: "boom".to_string(),
        }
    }

    fn preexisting(check: &str, files: Vec<&str>) -> PreexistingError {
        PreexistingError {
            rule_id: "X".to_string(),
            check: check.to_string(),
            files: files.into_iter().map(str::to_string).collect(),
        }
    }

    #[test]
    fn normalize_location_drops_trailing_line_number() {
        assert_eq!(Finding::normalize_location("src/main.rs:42"), "src/main.rs");
        assert_eq!(Finding::normalize_location("src/main.rs"), "src/main.rs");
        // A non-numeric suffix is part of the path, not a line number.
        assert_eq!(Finding::normalize_location("a:b/c.rs"), "a:b/c.rs");
    }

    #[test]
    fn preexisting_empty_files_suppresses_whole_check() {
        let baseline = vec![preexisting("no-cyrillic", vec![])];
        assert!(
            finding(Severity::Error, "no-cyrillic", Some("README.md:1")).is_preexisting(&baseline)
        );
        assert!(finding(Severity::Error, "no-cyrillic", None).is_preexisting(&baseline));
    }

    #[test]
    fn preexisting_matches_listed_file_only() {
        let baseline = vec![preexisting("no-cyrillic", vec!["docs/old.md"])];
        // Listed file, normalized from `file:line`, is suppressed.
        assert!(
            finding(Severity::Error, "no-cyrillic", Some("docs/old.md:3"))
                .is_preexisting(&baseline)
        );
        // A new error in another file still surfaces.
        assert!(
            !finding(Severity::Error, "no-cyrillic", Some("docs/new.md:3"))
                .is_preexisting(&baseline)
        );
    }

    #[test]
    fn preexisting_never_suppresses_warnings_or_other_checks() {
        let baseline = vec![preexisting("no-cyrillic", vec![])];
        // Warnings are never a baseline concern.
        assert!(
            !finding(Severity::Warn, "no-cyrillic", Some("README.md:1")).is_preexisting(&baseline)
        );
        // A different check name does not match.
        assert!(
            !finding(Severity::Error, "writing", Some("README.md:1")).is_preexisting(&baseline)
        );
    }
}
