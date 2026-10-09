#[cfg(test)]
mod tests {
    use super::super::*;
    use tempfile::tempdir;

    fn config(path: &str) -> RepositoryConfig {
        serde_yaml_ng::from_str(&format!(
            "repository_name: demo-api\nrepository_short_name: api\npath: {path}\ncontracts:\n  - path: contracts/api.yaml\n    description: REST API\n"
        ))
        .expect("parse repository config")
    }

    #[test]
    fn from_config_resolves_repository_and_contract_paths_against_the_spex_dir() {
        let spex_dir = Path::new("/workspace/demo-spex");

        let repo = Repository::from_config(config("../demo-api"), spex_dir);

        assert_eq!(repo.path, spex_dir.join("../demo-api"));
        assert_eq!(repo.contracts, vec![spex_dir.join("contracts/api.yaml")]);
        assert_eq!(repo.repository_name, "demo-api");
        assert_eq!(repo.repository_short_name, "api");
    }

    #[test]
    fn from_config_flags_the_repository_that_is_the_spex_dir() {
        let dir = tempdir().expect("spex dir");

        assert!(Repository::from_config(config("."), dir.path()).is_spex_repo);
        assert!(!Repository::from_config(config("../elsewhere"), dir.path()).is_spex_repo);
    }

    #[test]
    fn matches_any_repo_name_accepts_the_name_and_the_short_name() {
        let repo = Repository::from_config(config("."), Path::new("/spex"));

        assert!(repo.matches_any_repo_name("demo-api"));
        assert!(repo.matches_any_repo_name("api"));
        assert!(!repo.matches_any_repo_name("web"));
    }
}
