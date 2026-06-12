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
}
