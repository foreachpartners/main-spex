#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::commands::check::fixtures::{check, scoped_file};
    use crate::rule::{CheckKind, Severity};
    use std::path::Path;
    use tempfile::tempdir;

    fn glob_check() -> Check {
        check(CheckKind::Glob {
            pattern: "**".to_string(),
            negate: false,
        })
    }

    #[test]
    fn build_globset_returns_none_for_empty() {
        assert!(build_globset(&[]).expect("empty globset").is_none());
    }

    #[test]
    fn build_globset_matches_patterns() {
        let set = build_globset(&["**/*.rs".to_string()])
            .expect("valid globset")
            .expect("some globset");
        assert!(set.is_match(Path::new("src/main.rs")));
        assert!(!set.is_match(Path::new("src/main.ts")));
    }

    #[test]
    fn scope_files_applies_include_and_exclude() {
        let dir = tempdir().expect("temp dir");
        let files = vec![
            scoped_file(&dir, "src/main.rs", ""),
            scoped_file(&dir, "src/app.ts", ""),
            scoped_file(&dir, "src/main_test.rs", ""),
        ];
        let check = Check {
            include: vec!["**/*.rs".to_string()],
            exclude: vec!["**/*_test.rs".to_string()],
            ..glob_check()
        };

        let scoped = scope_files(&check, &files).expect("scoped");

        let names: Vec<&Path> = scoped.iter().map(|file| file.rel.as_path()).collect();
        assert_eq!(names, vec![Path::new("src/main.rs")]);
    }

    #[test]
    fn outcome_hits_become_findings_with_check_severity_and_source() {
        let check = Check {
            severity: Severity::Warn,
            ..glob_check()
        };
        let hits = vec![Hit::new(Some(Location::at_line("src/main.rs", 3)), "boom")];

        let findings = outcome_findings("secret-handling", &check, Ok(hits));

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Warn);
        assert_eq!(
            findings[0].location,
            Some(Location::at_line("src/main.rs", 3))
        );
        assert_eq!(findings[0].message, "boom");
        assert_eq!(
            findings[0].source.baseline_key(),
            Some(("secret-handling", "sample"))
        );
    }

    #[test]
    fn outcome_error_becomes_one_error_finding_without_location() {
        let check = Check {
            severity: Severity::Warn,
            ..glob_check()
        };

        let findings = outcome_findings("secret-handling", &check, Err("bad config".to_string()));

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Error);
        assert!(findings[0].location.is_none());
        assert_eq!(findings[0].message, "bad config");
        assert_eq!(
            findings[0].source.baseline_key(),
            Some(("secret-handling", "sample"))
        );
    }
}
