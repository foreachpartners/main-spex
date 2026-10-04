use crate::config::{Config, Repository};

use super::*;

fn repository(name: &str, path: &str) -> Repository {
    Repository {
        description: None,
        repository_name: name.to_string(),
        repository_short_name: name.to_string(),
        path: path.to_string(),
        contracts: Vec::new(),
        rules: Vec::new(),
        ignored_errors: Vec::new(),
    }
}

fn config(repositories: Vec<Repository>) -> Config {
    Config {
        description: None,
        rules_unused: Vec::new(),
        project_name: "acme".to_string(),
        cli_version: "0.0.0".to_string(),
        repositories,
    }
}

mod repo_base_dir_tests {
    use super::*;

    #[test]
    fn resolves_to_the_named_repository_path() {
        let config = config(vec![repository("api", "services/api")]);
        let spex_dir = Path::new("/workspace");

        assert_eq!(
            config.repo_base_dir(spex_dir, "api"),
            spex_dir.join("services/api")
        );
    }

    #[test]
    fn falls_back_to_the_spex_dir_for_the_workspace_report() {
        let config = config(vec![repository("api", "services/api")]);
        let spex_dir = Path::new("/workspace");

        assert_eq!(config.repo_base_dir(spex_dir, "workspace"), spex_dir);
    }

    #[test]
    fn falls_back_to_the_spex_dir_for_an_unknown_repository_name() {
        let config = config(vec![repository("api", "services/api")]);
        let spex_dir = Path::new("/workspace");

        assert_eq!(config.repo_base_dir(spex_dir, "unknown"), spex_dir);
    }
}

mod location_url_tests {
    use super::*;

    #[test]
    fn joins_the_base_dir_with_the_location() {
        let base = Path::new("/workspace/services/api");

        assert_eq!(
            location_url(base, "CLAUDE.md"),
            "file:///workspace/services/api/CLAUDE.md"
        );
    }

    #[test]
    fn drops_a_trailing_line_number() {
        let base = Path::new("/workspace/services/api");

        assert_eq!(
            location_url(base, "CLAUDE.md:50"),
            "file:///workspace/services/api/CLAUDE.md"
        );
    }

    #[test]
    fn encodes_spaces_in_the_path() {
        let base = Path::new("/workspace/my repo");

        assert_eq!(
            location_url(base, "CLAUDE.md"),
            "file:///workspace/my%20repo/CLAUDE.md"
        );
    }
}
