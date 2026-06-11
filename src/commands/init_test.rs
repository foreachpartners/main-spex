#[cfg(test)]
mod tests_unit {
    use super::super::*;
    use tempfile::{TempDir, tempdir};

    fn repo(name: &str) -> Repository {
        Repository {
            name: name.to_string(),
            path: format!("../{name}"),
        }
    }

    fn workspace_with(dirs: &[&str], files: &[&str]) -> TempDir {
        let root = tempdir().expect("create temp workspace");
        for dir in dirs {
            fs::create_dir(root.path().join(dir)).expect("create dir");
        }
        for file in files {
            fs::write(root.path().join(file), "").expect("create file");
        }
        root
    }

    fn repo_names(repositories: &[Repository]) -> Vec<&str> {
        repositories.iter().map(|repo| repo.name.as_str()).collect()
    }

    #[test]
    fn discover_repositories_excludes_hidden_and_files() {
        let workspace = workspace_with(&["server", "client", "spex", ".git"], &["notes.txt"]);

        let repositories = discover_repositories(workspace.path()).expect("discover");

        assert_eq!(repo_names(&repositories), vec!["client", "server", "spex"]);
    }

    #[test]
    fn discover_repositories_includes_spex_with_dot_path() {
        let workspace = workspace_with(&["server", "spex"], &[]);

        let repositories = discover_repositories(workspace.path()).expect("discover");

        let spex = repositories
            .iter()
            .find(|repo| repo.name == "spex")
            .expect("spex repository");
        assert_eq!(spex.path, ".");
    }

    #[test]
    fn discover_repositories_sorts_by_name() {
        let workspace = workspace_with(&["zeta", "alpha", "mu"], &[]);

        let repositories = discover_repositories(workspace.path()).expect("discover");

        assert_eq!(repo_names(&repositories), vec!["alpha", "mu", "zeta"]);
    }

    #[test]
    fn discover_repositories_sets_relative_path() {
        let workspace = workspace_with(&["server"], &[]);

        let repositories = discover_repositories(workspace.path()).expect("discover");

        assert_eq!(repositories[0].path, "../server");
    }

    #[test]
    fn discover_repositories_kebab_cases_name_but_keeps_path() {
        let workspace = workspace_with(&["My_Repo Name!"], &[]);

        let repositories = discover_repositories(workspace.path()).expect("discover");

        assert_eq!(repositories[0].name, "my-repo-name");
        assert_eq!(repositories[0].path, "../My_Repo Name!");
    }

    #[test]
    fn to_kebab_case_lowercases_and_replaces_separators() {
        assert_eq!(to_kebab_case("Repo Name"), "repo-name");
        assert_eq!(to_kebab_case("repo_name"), "repo-name");
        assert_eq!(to_kebab_case("Repo-Name"), "repo-name");
    }

    #[test]
    fn to_kebab_case_strips_non_english_chars_without_splitting() {
        assert_eq!(to_kebab_case("répo@näme!"), "rponme");
        assert_eq!(to_kebab_case("repo.name123"), "reponame123");
        assert_eq!(to_kebab_case("repo_name.v2"), "repo-namev2");
    }

    #[test]
    fn to_kebab_case_collapses_and_trims_separators() {
        assert_eq!(to_kebab_case("  repo   name  "), "repo-name");
        assert_eq!(to_kebab_case("__repo__name__"), "repo-name");
        assert_eq!(to_kebab_case("---"), "");
    }

    #[test]
    fn render_config_lists_repositories() {
        let yaml = config_to_yaml("pizza-shop", &[repo("server"), repo("client")]);

        let expected = "\
name: pizza-shop
repositories:
  - name: server
    path: ../server
    arch-specs: []
  - name: client
    path: ../client
    arch-specs: []
";
        assert_eq!(yaml, expected);
    }

    #[test]
    fn render_config_handles_no_repositories() {
        let yaml = config_to_yaml("pizza-shop", &[]);

        assert_eq!(yaml, "name: pizza-shop\nrepositories: []\n");
    }

    #[test]
    fn write_config_writes_file_to_spex_dir() {
        let spex_dir = tempdir().expect("create spex dir");

        let config_path =
            write_config(spex_dir.path(), "pizza-shop", &[repo("server")]).expect("write config");

        assert_eq!(config_path, spex_dir.path().join(SPEX_CONFIG_FILE));
        let written = fs::read_to_string(&config_path).expect("read config");
        assert_eq!(written, config_to_yaml("pizza-shop", &[repo("server")]));
    }

    #[test]
    fn verify_spex_dir_accepts_spex_directory() {
        let workspace = workspace_with(&["spex"], &[]);

        assert!(verify_spex_dir(&workspace.path().join("spex")).is_ok());
    }

    #[test]
    fn verify_spex_dir_rejects_other_directory() {
        let workspace = workspace_with(&["server"], &[]);

        assert!(verify_spex_dir(&workspace.path().join("server")).is_err());
    }

    #[test]
    fn verify_not_initialized_fails_when_config_exists() {
        let spex_dir = tempdir().expect("create spex dir");
        fs::write(spex_dir.path().join(SPEX_CONFIG_FILE), "").expect("create config");

        assert!(verify_not_initialized(spex_dir.path()).is_err());
    }

    #[test]
    fn verify_not_initialized_passes_on_empty_directory() {
        let spex_dir = tempdir().expect("create spex dir");

        assert!(verify_not_initialized(spex_dir.path()).is_ok());
    }

    #[test]
    fn copy_templates_writes_arch_specs_and_scripts() {
        let spex_dir = tempdir().expect("create spex dir");

        copy_templates(spex_dir.path()).expect("copy templates");

        assert!(spex_dir.path().join("arch-specs").is_dir());
        assert!(spex_dir.path().join("scripts").is_dir());
    }

    #[cfg(unix)]
    #[test]
    fn copy_templates_marks_scripts_executable() {
        use std::os::unix::fs::PermissionsExt;

        let spex_dir = tempdir().expect("create spex dir");

        copy_templates(spex_dir.path()).expect("copy templates");

        let script = spex_dir.path().join("scripts/commit-id-format.sh");
        let mode = fs::metadata(&script)
            .expect("read metadata")
            .permissions()
            .mode();
        assert_ne!(mode & 0o111, 0, "script is not executable");
    }

    #[test]
    fn is_yes_treats_empty_input_as_yes() {
        assert!(is_yes(""));
        assert!(is_yes("   "));
    }

    #[test]
    fn is_yes_parses_affirmative_and_negative() {
        assert!(is_yes("y"));
        assert!(is_yes("YES"));
        assert!(!is_yes("n"));
        assert!(!is_yes("nope"));
    }
}
