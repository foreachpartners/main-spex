#[cfg(test)]
mod tests_unit {
    use super::super::*;
    use std::fs;
    use tempfile::{TempDir, tempdir};

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
    fn config_from_template_names_project_and_spex_repository() {
        let yaml = config_from_template("pizza-shop");

        let config: Config = serde_yaml_ng::from_str(&yaml).expect("parse config");
        assert_eq!(config.project_name, "pizza-shop");
        assert_eq!(config.repositories.len(), 1);
        let spex = &config.repositories[0];
        assert_eq!(spex.repository_name, "pizza-shop-spex");
        assert_eq!(spex.repository_short_name, "spex");
        assert_eq!(spex.path, ".");
    }

    #[test]
    fn config_from_template_records_installed_cli_version() {
        let yaml = config_from_template("pizza-shop");

        let config: Config = serde_yaml_ng::from_str(&yaml).expect("parse config");
        assert_eq!(config.cli_version, CLI_VERSION);
    }

    #[test]
    fn config_from_template_leaves_no_unnamed_placeholders() {
        let yaml = config_from_template("pizza-shop");

        assert!(!yaml.contains("project_name: unknown"));
        assert!(!yaml.contains("repository_name: unknown"));
    }

    #[test]
    fn write_config_writes_file_to_spex_dir() {
        let spex_dir = tempdir().expect("create spex dir");

        write_config(spex_dir.path(), "pizza-shop").expect("write config");

        let config_path = spex_dir.path().join(SPEX_CONFIG_FILE);
        let written = fs::read_to_string(&config_path).expect("read config");
        assert_eq!(written, config_from_template("pizza-shop"));
    }

    #[test]
    fn write_code_workspace_uses_project_name_file() {
        let spex_dir = tempdir().expect("create spex dir");

        write_code_workspace(spex_dir.path(), "pizza-shop").expect("write code-workspace");

        let path = spex_dir.path().join("pizza-shop.code-workspace");
        let written = fs::read_to_string(&path).expect("read code-workspace");
        assert_eq!(written, CODE_WORKSPACE_TEMPLATE);
    }

    #[test]
    fn workspace_root_defaults_to_current_directory() {
        let workspace = workspace_with(&["server"], &[]);

        assert_eq!(
            workspace_root(workspace.path()).expect("workspace root"),
            workspace.path()
        );
    }

    #[test]
    fn workspace_root_from_inside_spex_directory() {
        let workspace = workspace_with(&["pizza-shop-spex"], &[]);
        let spex_dir = workspace.path().join("pizza-shop-spex");

        assert_eq!(
            workspace_root(&spex_dir).expect("workspace root"),
            workspace.path()
        );
    }

    #[test]
    fn ensure_spex_dir_reuses_current_when_inside_spex_directory() {
        let workspace = workspace_with(&["pizza-shop-spex"], &[]);
        let spex_dir = workspace.path().join("pizza-shop-spex");

        let resolved =
            ensure_spex_dir(&spex_dir, workspace.path(), "pizza-shop").expect("ensure spex dir");

        assert_eq!(resolved, spex_dir);
    }

    #[test]
    fn ensure_spex_dir_creates_project_named_directory() {
        let workspace = workspace_with(&["server"], &[]);

        let resolved = ensure_spex_dir(workspace.path(), workspace.path(), "pizza-shop")
            .expect("ensure spex dir");

        assert_eq!(resolved, workspace.path().join("pizza-shop-spex"));
        assert!(resolved.is_dir(), "spex directory was not created");
    }

    #[test]
    fn ensure_spex_dir_reuses_existing_directory() {
        let workspace = workspace_with(&["pizza-shop-spex"], &[]);

        let resolved = ensure_spex_dir(workspace.path(), workspace.path(), "pizza-shop")
            .expect("ensure spex dir");

        assert_eq!(resolved, workspace.path().join("pizza-shop-spex"));
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
    fn copy_templates_writes_rules_and_scripts() {
        let spex_dir = tempdir().expect("create spex dir");

        copy_templates(spex_dir.path()).expect("copy templates");

        assert!(spex_dir.path().join("rules").is_dir());
        assert!(spex_dir.path().join("specs").is_dir());
        assert!(spex_dir.path().join("scripts").is_dir());
        assert!(spex_dir.path().join("plans").is_dir());
        assert!(spex_dir.path().join("contracts").is_dir());
    }

    #[test]
    fn copy_templates_writes_schemas() {
        let spex_dir = tempdir().expect("create spex dir");

        copy_templates(spex_dir.path()).expect("copy templates");

        assert!(spex_dir.path().join("schemas").is_dir());
        assert!(spex_dir.path().join("schemas/spec.schema.json").is_file());
    }

    #[test]
    fn copy_templates_relocates_skills_into_agents() {
        let spex_dir = tempdir().expect("create spex dir");

        copy_templates(spex_dir.path()).expect("copy templates");

        assert!(!spex_dir.path().join("skills").exists());
        assert!(spex_dir.path().join(".agents/skills").is_dir());
        assert!(
            spex_dir
                .path()
                .join(".agents/skills/spex-spec/SKILL.md")
                .is_file()
        );
    }

    #[cfg(unix)]
    #[test]
    fn copy_templates_links_claude_to_agents() {
        let spex_dir = tempdir().expect("create spex dir");

        copy_templates(spex_dir.path()).expect("copy templates");

        let claude = spex_dir.path().join(".claude");
        let target = fs::read_link(&claude).expect("read .claude symlink");
        assert_eq!(target, std::path::Path::new(".agents"));
        assert!(
            claude.join("skills/spex-spec/SKILL.md").is_file(),
            ".claude/skills should resolve through the symlink"
        );
    }

    #[cfg(unix)]
    #[test]
    fn copy_templates_marks_scripts_executable() {
        use std::os::unix::fs::PermissionsExt;

        let spex_dir = tempdir().expect("create spex dir");

        copy_templates(spex_dir.path()).expect("copy templates");

        let script = spex_dir
            .path()
            .join("scripts/commit-message-format/commit-spec-id-trace.sh");
        let mode = fs::metadata(&script)
            .expect("read metadata")
            .permissions()
            .mode();
        assert_ne!(mode & 0o111, 0, "script is not executable");
    }
}
