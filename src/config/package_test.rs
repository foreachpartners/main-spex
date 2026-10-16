#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::error::describe;
    use tempfile::tempdir;

    fn config(path: &str) -> PackageEntry {
        serde_yaml_ng::from_str(&format!(
            "package_name: demo-api\npackage_short_name: api\npath: {path}\ncontracts:\n  - path: contracts/api.yaml\n    description: REST API\n"
        ))
        .expect("parse package config")
    }

    #[test]
    fn from_config_resolves_package_and_contract_paths_against_the_spex_dir() {
        let spex_dir = Path::new("/project/demo-spex");

        let package = Package::from_config(config("../demo-api"), spex_dir);

        assert_eq!(package.path, Path::new("/project/demo-api"));
        assert_eq!(package.contracts, vec![spex_dir.join("contracts/api.yaml")]);
        assert_eq!(package.package_name, "demo-api");
        assert_eq!(package.package_short_name, "api");
    }

    #[test]
    fn from_config_resolves_a_parent_path_to_the_directory_holding_the_spex_dir() {
        let spex_dir = Path::new("/project/snake/spex");

        let package = Package::from_config(config(".."), spex_dir);

        assert_eq!(package.path, Path::new("/project/snake"));
        assert!(!package.is_spex_dir);
    }

    #[test]
    fn from_config_flags_the_package_that_is_the_spex_dir() {
        let dir = tempdir().expect("spex dir");

        assert!(Package::from_config(config("."), dir.path()).is_spex_dir);
        assert!(!Package::from_config(config("../elsewhere"), dir.path()).is_spex_dir);
    }

    #[test]
    fn matches_any_package_name_accepts_the_name_and_the_short_name() {
        let package = Package::from_config(config("."), Path::new("/spex"));

        assert!(package.matches_any_package_name("demo-api"));
        assert!(package.matches_any_package_name("api"));
        assert!(!package.matches_any_package_name("web"));
    }

    #[test]
    fn from_config_keeps_the_exclude_of_the_package_entry() {
        let entry: PackageEntry = serde_yaml_ng::from_str(
            "package_name: demo-api\npackage_short_name: api\npath: .\nexclude: [vendor]\n",
        )
        .expect("parse package entry");

        let package = Package::from_config(entry, Path::new("/spex"));

        assert_eq!(package.exclude, vec!["vendor"]);
    }

    #[test]
    fn compile_exclude_names_the_package_when_a_pattern_is_not_a_valid_glob() {
        let mut package = Package::from_config(config("."), Path::new("/spex"));
        package.exclude = vec!["src/{a,b".to_string()];

        let error = package.compile_exclude().expect_err("invalid exclude");

        let message = describe(&error);
        assert!(message.contains("`demo-api`"), "{message}");
        assert!(message.contains("src/{a,b"), "{message}");
    }
}
