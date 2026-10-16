#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::error::describe;
    use crate::spec::CoverageLayer;
    use indoc::indoc;
    use tempfile::tempdir;

    #[test]
    fn load_returns_an_empty_config_when_the_package_has_no_file() {
        let package = tempdir().expect("package dir");

        let config = PackageConfig::load(package.path()).expect("load");

        assert!(config.coverage_ignore.is_empty());
        assert!(config.check_ignore.is_empty());
    }

    #[test]
    fn load_returns_an_empty_config_when_the_package_path_does_not_exist() {
        let config = PackageConfig::load(Path::new("/no/such/package")).expect("load");

        assert!(config.coverage_ignore.is_empty());
        assert!(config.check_ignore.is_empty());
    }

    #[test]
    fn load_reads_coverage_ignore_and_check_ignore() {
        let package = tempdir().expect("package dir");
        fs::write(
            package.path().join(PACKAGE_CONFIG_FILE),
            indoc! {"
                coverage_ignore:
                  - spec_id: place-order
                    layer: contract
                  - spec_id: place-order
                    layer: impl
                    ac: rejects-empty-cart
                check_ignore:
                  - rule_id: writing-style
                    check: no-cyrillic
                    files: [README.md]
            "},
        )
        .expect("write package config");

        let config = PackageConfig::load(package.path()).expect("load");

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
        assert_eq!(config.check_ignore.len(), 1);
        assert_eq!(config.check_ignore[0].rule_id, "writing-style");
        assert_eq!(config.check_ignore[0].files, vec!["README.md"]);
    }

    #[test]
    fn load_accepts_a_file_holding_only_comments() {
        let package = tempdir().expect("package dir");
        fs::write(
            package.path().join(PACKAGE_CONFIG_FILE),
            "# nothing to ignore yet\n",
        )
        .expect("write package config");

        let config = PackageConfig::load(package.path()).expect("load");

        assert!(config.coverage_ignore.is_empty());
        assert!(config.check_ignore.is_empty());
    }

    #[test]
    fn load_accepts_an_empty_file() {
        let package = tempdir().expect("package dir");
        fs::write(package.path().join(PACKAGE_CONFIG_FILE), "").expect("write package config");

        assert!(PackageConfig::load(package.path()).is_ok());
    }

    #[test]
    fn load_errors_naming_the_file_when_it_is_not_valid_yaml() {
        let package = tempdir().expect("package dir");
        let path = package.path().join(PACKAGE_CONFIG_FILE);
        fs::write(&path, "coverage_ignore: [unclosed\n").expect("write package config");

        let error = PackageConfig::load(package.path()).expect_err("invalid yaml");

        assert!(
            describe(&error).starts_with(&format!("could not parse `{}`: ", path.display())),
            "{}",
            describe(&error)
        );
    }

    #[test]
    fn load_errors_when_an_entry_names_an_invalid_spec_id() {
        let package = tempdir().expect("package dir");
        fs::write(
            package.path().join(PACKAGE_CONFIG_FILE),
            "coverage_ignore:\n  - spec_id: Not_Valid\n    layer: impl\n",
        )
        .expect("write package config");

        assert!(PackageConfig::load(package.path()).is_err());
    }
}
