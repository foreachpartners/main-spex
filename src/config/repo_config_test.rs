#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::spec::CoverageLayer;
    use indoc::indoc;
    use tempfile::tempdir;

    #[test]
    fn load_returns_an_empty_config_when_the_repository_has_no_file() {
        let repo = tempdir().expect("repo dir");

        let config = RepoConfig::load(repo.path()).expect("load");

        assert!(config.coverage_ignore.is_empty());
        assert!(config.ignored_errors.is_empty());
    }

    #[test]
    fn load_returns_an_empty_config_when_the_repository_path_does_not_exist() {
        let config = RepoConfig::load(Path::new("/no/such/repository")).expect("load");

        assert!(config.coverage_ignore.is_empty());
        assert!(config.ignored_errors.is_empty());
    }

    #[test]
    fn load_reads_coverage_ignore_and_ignored_errors() {
        let repo = tempdir().expect("repo dir");
        fs::write(
            repo.path().join(REPO_CONFIG_FILE),
            indoc! {"
                coverage_ignore:
                  - spec_id: place-order
                    layer: contract
                  - spec_id: place-order
                    layer: impl
                    ac: rejects-empty-cart
                ignored_errors:
                  - rule_id: writing-style
                    check: no-cyrillic
                    files: [README.md]
            "},
        )
        .expect("write repo config");

        let config = RepoConfig::load(repo.path()).expect("load");

        assert_eq!(config.coverage_ignore.len(), 2);
        assert_eq!(config.coverage_ignore[0].spec_id.as_str(), "place-order");
        assert_eq!(
            config.coverage_ignore[0].ignore.layer,
            CoverageLayer::Contract
        );
        assert!(config.coverage_ignore[0].ignore.ac.is_none());
        assert_eq!(
            config.coverage_ignore[1]
                .ignore
                .ac
                .as_ref()
                .map(|ac| ac.as_str()),
            Some("rejects-empty-cart")
        );
        assert_eq!(config.ignored_errors.len(), 1);
        assert_eq!(config.ignored_errors[0].rule_id, "writing-style");
        assert_eq!(config.ignored_errors[0].files, vec!["README.md"]);
    }

    #[test]
    fn load_accepts_a_file_holding_only_comments() {
        let repo = tempdir().expect("repo dir");
        fs::write(
            repo.path().join(REPO_CONFIG_FILE),
            "# nothing to ignore yet\n",
        )
        .expect("write repo config");

        let config = RepoConfig::load(repo.path()).expect("load");

        assert!(config.coverage_ignore.is_empty());
        assert!(config.ignored_errors.is_empty());
    }

    #[test]
    fn load_accepts_an_empty_file() {
        let repo = tempdir().expect("repo dir");
        fs::write(repo.path().join(REPO_CONFIG_FILE), "").expect("write repo config");

        assert!(RepoConfig::load(repo.path()).is_ok());
    }

    #[test]
    fn load_errors_naming_the_file_when_it_is_not_valid_yaml() {
        let repo = tempdir().expect("repo dir");
        let path = repo.path().join(REPO_CONFIG_FILE);
        fs::write(&path, "coverage_ignore: [unclosed\n").expect("write repo config");

        let error = RepoConfig::load(repo.path()).expect_err("invalid yaml");

        assert!(matches!(
            error.current_context(),
            CliError::ParseFile(failed) if failed == &path
        ));
    }

    #[test]
    fn load_errors_when_an_entry_names_an_invalid_spec_id() {
        let repo = tempdir().expect("repo dir");
        fs::write(
            repo.path().join(REPO_CONFIG_FILE),
            "coverage_ignore:\n  - spec_id: Not_Valid\n    layer: impl\n",
        )
        .expect("write repo config");

        assert!(RepoConfig::load(repo.path()).is_err());
    }
}
