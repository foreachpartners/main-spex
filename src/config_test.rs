#[cfg(test)]
mod tests {
    use super::super::*;
    use tempfile::tempdir;

    fn workspace() -> tempfile::TempDir {
        let workspace = tempdir().expect("workspace");
        let spex = workspace.path().join(SPEX_DIR_NAME);
        std::fs::create_dir_all(workspace.path().join("api/src")).expect("repo");
        std::fs::create_dir_all(&spex).expect("spex dir");
        std::fs::write(spex.join(SPEX_CONFIG_FILE), "project_name: demo\n").expect("config");
        workspace
    }

    #[test]
    fn finds_from_workspace_root() {
        let workspace = workspace();
        let found = find_dir(workspace.path()).expect("found");
        assert_eq!(found, workspace.path().join(SPEX_DIR_NAME));
    }

    #[test]
    fn finds_from_spex_dir() {
        let workspace = workspace();
        let spex = workspace.path().join(SPEX_DIR_NAME);
        let found = find_dir(&spex).expect("found");
        assert_eq!(found, spex);
    }

    #[test]
    fn finds_from_nested_repo_subdirectory() {
        let workspace = workspace();
        let found = find_dir(&workspace.path().join("api/src")).expect("found");
        assert_eq!(found, workspace.path().join(SPEX_DIR_NAME));
    }

    #[test]
    fn errors_when_not_in_a_workspace() {
        let dir = tempdir().expect("dir");
        assert!(find_dir(dir.path()).is_err());
    }

    fn config_with_version(version: &str) -> Config {
        Config {
            project_name: "demo".to_string(),
            cli_version: version.to_string(),
            repositories: Vec::new(),
        }
    }

    #[test]
    fn version_warning_is_none_when_matching() {
        assert!(version_warning(&config_with_version(CLI_VERSION)).is_none());
    }

    #[test]
    fn version_warning_reports_mismatch() {
        let warning = version_warning(&config_with_version("0.0.1")).expect("warning");
        assert!(warning.contains("0.0.1"));
        assert!(warning.contains(CLI_VERSION));
    }

    #[test]
    fn set_cli_version_replaces_existing_line() {
        let raw = "project_name: demo\ncli_version: 0.0.1\nrepositories: []\n";
        assert_eq!(
            set_cli_version(raw, "0.2.0"),
            "project_name: demo\ncli_version: 0.2.0\nrepositories: []\n"
        );
    }

    #[test]
    fn set_cli_version_inserts_after_project_name() {
        let raw = "project_name: demo\nrepositories: []\n";
        assert_eq!(
            set_cli_version(raw, "0.2.0"),
            "project_name: demo\ncli_version: 0.2.0\nrepositories: []\n"
        );
    }

    #[test]
    fn set_cli_version_preserves_comments() {
        let raw = "project_name: demo\ncli_version: 0.0.1\n# keep me\nrepositories: []\n";
        assert_eq!(
            set_cli_version(raw, "0.2.0"),
            "project_name: demo\ncli_version: 0.2.0\n# keep me\nrepositories: []\n"
        );
    }
}
