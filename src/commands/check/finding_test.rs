#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::support::testing;

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

    fn repository_at(path: &str, ignored_errors: Vec<PreexistingError>) -> Repository {
        Repository {
            ignored_errors,
            is_spex_repo: path == ".",
            ..testing::repository(path, path)
        }
    }

    fn owner(finding: &Finding, repos: &[&Repository]) -> Option<(String, Option<String>)> {
        finding.owning_repo(repos, testing::spex_dir())
    }

    #[test]
    fn owning_repo_resolves_a_location_to_the_repository_containing_it() {
        let api = repository_at("../api", vec![]);
        let web = repository_at("../web", vec![]);
        let located = finding(Severity::Error, "check", Some("../web/src/main.ts:4"));

        assert_eq!(
            owner(&located, &[&api, &web]),
            Some(("../web".to_string(), Some("src/main.ts".to_string())))
        );
    }

    #[test]
    fn owning_repo_does_not_give_a_sibling_location_to_the_spex_repository() {
        let spex = repository_at(".", vec![]);
        let api = repository_at("../api", vec![]);
        let located = finding(Severity::Error, "check", Some("../api/src/main.rs"));

        assert_eq!(
            owner(&located, &[&spex, &api]),
            Some(("../api".to_string(), Some("src/main.rs".to_string())))
        );
    }

    #[test]
    fn owning_repo_gives_a_spex_dir_location_to_the_spex_repository() {
        let spex = repository_at(".", vec![]);
        let api = repository_at("../api", vec![]);
        let located = finding(Severity::Error, "check", Some("specs/a.yaml:2"));

        assert_eq!(
            owner(&located, &[&api, &spex]),
            Some((".".to_string(), Some("specs/a.yaml".to_string())))
        );
    }

    #[test]
    fn owning_repo_is_none_without_a_location_or_a_containing_repository() {
        let api = repository_at("../api", vec![]);

        assert_eq!(
            owner(&finding(Severity::Error, "check", None), &[&api]),
            None
        );
        assert_eq!(
            owner(
                &finding(Severity::Error, "check", Some("../web/x.ts")),
                &[&api]
            ),
            None
        );
    }

    #[test]
    fn preexisting_workspace_matches_the_owning_repositorys_baseline_file() {
        let api = repository_at(
            "../api",
            vec![preexisting("no-cyrillic", vec!["src/old.rs"])],
        );
        let is_preexisting = |severity, location| {
            finding(severity, "no-cyrillic", Some(location))
                .is_preexisting_workspace(&[&api], testing::spex_dir())
        };

        assert!(is_preexisting(Severity::Error, "../api/src/old.rs:9"));
        assert!(!is_preexisting(Severity::Error, "../api/src/new.rs:9"));
        assert!(!is_preexisting(Severity::Error, "../web/src/old.rs:9"));
        assert!(!is_preexisting(Severity::Warn, "../api/src/old.rs:9"));
    }
}
