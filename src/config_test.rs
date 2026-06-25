#[cfg(test)]
mod tests {
    use super::super::*;
    use indoc::indoc;
    use std::collections::HashMap;
    use tempfile::tempdir;

    // The spex repository directory is named `<project-name>-spex`, so discovery
    // must find it by the `spex.yaml` it holds, not by a fixed directory name.
    const SPEX_REPO_DIR: &str = "demo-spex";

    fn workspace() -> tempfile::TempDir {
        let workspace = tempdir().expect("workspace");
        let spex = workspace.path().join(SPEX_REPO_DIR);
        std::fs::create_dir_all(workspace.path().join("demo-api/src")).expect("repo");
        std::fs::create_dir_all(&spex).expect("spex dir");
        std::fs::write(spex.join(SPEX_CONFIG_FILE), "project_name: demo\n").expect("config");
        workspace
    }

    #[test]
    fn finds_from_workspace_root() {
        let workspace = workspace();
        let found = find_dir(workspace.path()).expect("found");
        assert_eq!(found, workspace.path().join(SPEX_REPO_DIR));
    }

    #[test]
    fn finds_from_spex_dir() {
        let workspace = workspace();
        let spex = workspace.path().join(SPEX_REPO_DIR);
        let found = find_dir(&spex).expect("found");
        assert_eq!(found, spex);
    }

    #[test]
    fn finds_from_nested_repo_subdirectory() {
        let workspace = workspace();
        let found = find_dir(&workspace.path().join("demo-api/src")).expect("found");
        assert_eq!(found, workspace.path().join(SPEX_REPO_DIR));
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
    fn finds_spex_dir_directly_when_run_inside_it() {
        let dir = tempdir().expect("dir");
        std::fs::write(dir.path().join(SPEX_CONFIG_FILE), "project_name: demo\n").expect("config");
        assert_eq!(find_dir(dir.path()).expect("found"), dir.path());
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
        let raw = indoc! {"
            project_name: demo
            cli_version: 0.0.1
            repositories: []
        "};
        assert_eq!(
            set_cli_version(raw, "0.2.0"),
            indoc! {"
                project_name: demo
                cli_version: 0.2.0
                repositories: []
            "}
        );
    }

    #[test]
    fn set_cli_version_inserts_after_project_name() {
        let raw = indoc! {"
            project_name: demo
            repositories: []
        "};
        assert_eq!(
            set_cli_version(raw, "0.2.0"),
            indoc! {"
                project_name: demo
                cli_version: 0.2.0
                repositories: []
            "}
        );
    }

    #[test]
    fn set_cli_version_preserves_comments() {
        let raw = indoc! {"
            project_name: demo
            cli_version: 0.0.1
            # keep me
            repositories: []
        "};
        assert_eq!(
            set_cli_version(raw, "0.2.0"),
            indoc! {"
                project_name: demo
                cli_version: 0.2.0
                # keep me
                repositories: []
            "}
        );
    }

    fn entry(spec: &str, check: &str, files: &[&str]) -> PreexistingEntry {
        PreexistingEntry {
            spec: spec.to_string(),
            check: check.to_string(),
            files: files.iter().map(|file| file.to_string()).collect(),
        }
    }

    const TWO_REPOS: &str = indoc! {"
        project_name: demo
        repositories:
          - repository_name: api
            path: ../api
            tech_specs:
              - TECH-X
          - repository_name: web
            path: ../web
            tech_specs:
              - TECH-X
    "};

    #[test]
    fn set_preexisting_errors_appends_block_with_files() {
        let by_repo = HashMap::from([(
            "api".to_string(),
            vec![entry(
                "TECH-X",
                "no-cyrillic",
                &["README.md", "docs/intro.md"],
            )],
        )]);
        let out = set_preexisting_errors(TWO_REPOS, &by_repo);
        assert_eq!(
            out,
            indoc! {"
                project_name: demo
                repositories:
                  - repository_name: api
                    path: ../api
                    tech_specs:
                      - TECH-X
                    checks_preexisting_errors:
                      - spec: TECH-X
                        check: no-cyrillic
                        files:
                          - README.md
                          - docs/intro.md
                  - repository_name: web
                    path: ../web
                    tech_specs:
                      - TECH-X
            "}
        );
    }

    #[test]
    fn set_preexisting_errors_renders_whole_check_without_files() {
        let by_repo = HashMap::from([(
            "web".to_string(),
            vec![entry("TECH-Y", "commit-format", &[])],
        )]);
        let out = set_preexisting_errors(TWO_REPOS, &by_repo);
        // A mid-document fragment keeps its 2-space indent, which indoc would strip.
        assert!(out.ends_with(
            "  - repository_name: web
    path: ../web
    tech_specs:
      - TECH-X
    checks_preexisting_errors:
      - spec: TECH-Y
        check: commit-format
"
        ));
        // The api repository is out of scope, so it keeps no block.
        assert!(!out.contains("check: commit-format\n  - repository_name: web"));
        assert_eq!(out.matches("checks_preexisting_errors:").count(), 1);
    }

    #[test]
    fn set_preexisting_errors_replaces_existing_block() {
        let raw = indoc! {"
            project_name: demo
            repositories:
              - repository_name: api
                path: ../api
                checks_preexisting_errors:
                  - spec: TECH-X
                    check: stale
                    files:
                      - old.md
                tech_specs:
                  - TECH-X
        "};
        let by_repo = HashMap::from([(
            "api".to_string(),
            vec![entry("TECH-X", "no-cyrillic", &["README.md"])],
        )]);
        let out = set_preexisting_errors(raw, &by_repo);
        assert!(!out.contains("stale"), "old block survived:\n{out}");
        assert!(
            out.contains("check: no-cyrillic"),
            "new block missing:\n{out}"
        );
        // The block is regenerated once, not duplicated.
        assert_eq!(out.matches("checks_preexisting_errors:").count(), 1);
    }

    #[test]
    fn set_preexisting_errors_removes_stale_block_when_empty() {
        let raw = indoc! {"
            project_name: demo
            repositories:
              - repository_name: api
                path: ../api
                checks_preexisting_errors:
                  - spec: TECH-X
                    check: gone
        "};
        // api is in scope but has no current errors, so its block is dropped.
        let by_repo = HashMap::from([("api".to_string(), Vec::new())]);
        let out = set_preexisting_errors(raw, &by_repo);
        assert_eq!(
            out,
            indoc! {"
                project_name: demo
                repositories:
                  - repository_name: api
                    path: ../api
            "}
        );
    }
}
