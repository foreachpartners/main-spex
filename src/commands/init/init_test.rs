#[cfg(test)]
mod tests_unit {
    use super::super::*;
    use crate::error::describe;
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
    fn parse_layout_defaults_to_multi_repo_on_an_empty_answer() {
        assert_eq!(parse_layout("").expect("layout"), Layout::MultiRepo);
    }

    #[test]
    fn parse_layout_accepts_the_number_in_the_listed_order() {
        assert_eq!(parse_layout("1").expect("layout"), Layout::MultiRepo);
        assert_eq!(parse_layout("2").expect("layout"), Layout::SingleRepo);
        assert_eq!(parse_layout("3").expect("layout"), Layout::Monorepo);
    }

    #[test]
    fn parse_layout_accepts_the_name_in_any_case() {
        assert_eq!(parse_layout("monorepo").expect("layout"), Layout::Monorepo);
        assert_eq!(
            parse_layout("Single-Repo").expect("layout"),
            Layout::SingleRepo
        );
    }

    #[test]
    fn parse_layout_rejects_numbers_outside_the_list_and_unknown_names() {
        for answer in ["0", "4", "-1", "polyrepo"] {
            let error = parse_layout(answer).expect_err("unknown layout");
            assert!(
                describe(&error).contains(&format!("unknown layout `{answer}`")),
                "unexpected error for `{answer}`: {}",
                describe(&error)
            );
        }
    }

    #[test]
    fn to_kebab_case_lowercases_and_replaces_separators() {
        assert_eq!(to_kebab_case("Package Name"), "package-name");
        assert_eq!(to_kebab_case("package_name"), "package-name");
        assert_eq!(to_kebab_case("Package-Name"), "package-name");
    }

    #[test]
    fn to_kebab_case_strips_non_english_chars_without_splitting() {
        assert_eq!(to_kebab_case("répo@näme!"), "rponme");
        assert_eq!(to_kebab_case("package.name123"), "packagename123");
        assert_eq!(to_kebab_case("package_name.v2"), "package-namev2");
    }

    #[test]
    fn to_kebab_case_collapses_and_trims_separators() {
        assert_eq!(to_kebab_case("  package   name  "), "package-name");
        assert_eq!(to_kebab_case("__package__name__"), "package-name");
        assert_eq!(to_kebab_case("---"), "");
    }

    fn parsed_config(layout: Layout) -> SpexConfig {
        serde_yaml_ng::from_str(&config_from_template(layout, "pizza-shop")).expect("parse config")
    }

    #[test]
    fn multi_repo_config_names_project_and_lists_the_spex_repository() {
        let config = parsed_config(Layout::MultiRepo);

        assert_eq!(config.project_name, "pizza-shop");
        assert_eq!(config.layout, Layout::MultiRepo);
        assert_eq!(config.agent_root, ".");
        assert_eq!(config.packages.len(), 1);
        let spex = &config.packages[0];
        assert_eq!(spex.package_name, "pizza-shop-spex");
        assert_eq!(spex.package_short_name, "spex");
        assert_eq!(spex.path, ".");
    }

    #[test]
    fn single_repo_config_lists_the_package_named_after_the_project() {
        let config = parsed_config(Layout::SingleRepo);

        assert_eq!(config.layout, Layout::SingleRepo);
        assert_eq!(config.agent_root, "..");
        assert_eq!(config.packages.len(), 1);
        let package = &config.packages[0];
        assert_eq!(package.package_name, "pizza-shop");
        assert_eq!(package.package_short_name, "pizza-shop");
        assert_eq!(package.path, "..");
        assert!(
            package
                .rules
                .iter()
                .all(|rule| rule.as_str() != "naming-convention")
        );
    }

    #[test]
    fn monorepo_config_lists_no_package() {
        let config = parsed_config(Layout::Monorepo);

        assert_eq!(config.layout, Layout::Monorepo);
        assert_eq!(config.agent_root, "..");
        assert!(config.packages.is_empty());
    }

    #[test]
    fn config_from_template_records_installed_cli_version() {
        for layout in [Layout::MultiRepo, Layout::SingleRepo, Layout::Monorepo] {
            assert_eq!(parsed_config(layout).cli_version, CLI_VERSION);
        }
    }

    #[test]
    fn config_from_template_leaves_no_unnamed_placeholders() {
        for layout in [Layout::MultiRepo, Layout::SingleRepo, Layout::Monorepo] {
            let yaml = config_from_template(layout, "pizza-shop");

            for field in [
                "project_name",
                "layout",
                "agent_root",
                "package_name",
                "package_short_name",
            ] {
                assert!(
                    !yaml.contains(&format!("{field}: unknown")),
                    "`{field}` left unnamed for {layout}"
                );
            }
        }
    }

    #[test]
    fn config_from_template_lists_root_fields_in_schema_order() {
        let yaml = config_from_template(Layout::SingleRepo, "pizza-shop");

        let positions: Vec<usize> = ["project_name:", "cli_version:", "layout:", "agent_root:"]
            .iter()
            .map(|field| yaml.find(field).expect("root field"))
            .collect();
        assert!(positions.is_sorted());
    }

    #[test]
    fn write_config_writes_file_to_spex_dir() {
        let spex_dir = tempdir().expect("create spex dir");

        write_config(spex_dir.path(), Layout::SingleRepo, "pizza-shop").expect("write config");

        let config_path = spex_dir.path().join(SPEX_CONFIG_FILE);
        let written = fs::read_to_string(&config_path).expect("read config");
        assert_eq!(
            written,
            config_from_template(Layout::SingleRepo, "pizza-shop")
        );
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
    fn locate_spex_dir_reuses_current_when_inside_spex_directory() {
        let workspace = workspace_with(&["pizza-shop-spex"], &[]);
        let spex_dir = workspace.path().join("pizza-shop-spex");

        let located = locate_spex_dir(&spex_dir, workspace.path(), Layout::MultiRepo, "pizza-shop");

        assert_eq!(located, spex_dir);
    }

    #[test]
    fn locate_spex_dir_names_the_multi_repo_directory_after_the_project() {
        let workspace = workspace_with(&["server"], &[]);

        let located = locate_spex_dir(
            workspace.path(),
            workspace.path(),
            Layout::MultiRepo,
            "pizza-shop",
        );

        assert_eq!(located, workspace.path().join("pizza-shop-spex"));
    }

    #[test]
    fn locate_spex_dir_names_the_other_layouts_directory_spex() {
        let workspace = workspace_with(&["server"], &[]);

        for layout in [Layout::SingleRepo, Layout::Monorepo] {
            let located = locate_spex_dir(workspace.path(), workspace.path(), layout, "pizza-shop");

            assert_eq!(located, workspace.path().join("spex"));
        }
    }

    #[test]
    fn locate_spex_dir_does_not_create_the_directory() {
        let workspace = workspace_with(&[], &[]);

        let located = locate_spex_dir(
            workspace.path(),
            workspace.path(),
            Layout::SingleRepo,
            "pizza-shop",
        );

        assert!(!located.exists());
    }

    #[test]
    fn verify_empty_passes_on_a_directory_that_does_not_exist() {
        let workspace = workspace_with(&[], &[]);

        assert!(verify_empty(&workspace.path().join("spex")).is_ok());
    }

    #[test]
    fn verify_empty_fails_when_directory_has_a_file() {
        let spex_dir = tempdir().expect("create spex dir");
        fs::write(spex_dir.path().join(SPEX_CONFIG_FILE), "").expect("create config");

        assert!(verify_empty(spex_dir.path()).is_err());
    }

    #[test]
    fn verify_empty_fails_when_directory_has_a_subdirectory() {
        let spex_dir = tempdir().expect("create spex dir");
        fs::create_dir(spex_dir.path().join(".git")).expect("create subdirectory");

        assert!(verify_empty(spex_dir.path()).is_err());
    }

    #[test]
    fn verify_empty_passes_on_empty_directory() {
        let spex_dir = tempdir().expect("create spex dir");

        assert!(verify_empty(spex_dir.path()).is_ok());
    }

    #[test]
    fn copy_templates_writes_rules_and_scripts() {
        let spex_dir = tempdir().expect("create spex dir");

        copy_templates(spex_dir.path(), spex_dir.path()).expect("copy templates");

        assert!(spex_dir.path().join("rules").is_dir());
        assert!(spex_dir.path().join("specs").is_dir());
        assert!(spex_dir.path().join("rules/commit-message-format").is_dir());
        assert!(spex_dir.path().join("plans").is_dir());
        assert!(spex_dir.path().join("contracts").is_dir());
    }

    #[test]
    fn copy_templates_writes_schemas() {
        let spex_dir = tempdir().expect("create spex dir");

        copy_templates(spex_dir.path(), spex_dir.path()).expect("copy templates");

        assert!(spex_dir.path().join("schemas").is_dir());
        assert!(spex_dir.path().join("schemas/spec.schema.json").is_file());
    }

    #[test]
    fn copy_templates_relocates_skills_into_agents() {
        let spex_dir = tempdir().expect("create spex dir");

        copy_templates(spex_dir.path(), spex_dir.path()).expect("copy templates");

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

        copy_templates(spex_dir.path(), spex_dir.path()).expect("copy templates");

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

        copy_templates(spex_dir.path(), spex_dir.path()).expect("copy templates");

        let script = spex_dir
            .path()
            .join("rules/commit-message-format/commit-spec-id-trace.sh");
        let mode = fs::metadata(&script)
            .expect("read metadata")
            .permissions()
            .mode();
        assert_ne!(mode & 0o111, 0, "script is not executable");
    }
}
