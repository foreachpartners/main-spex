#[cfg(test)]
mod tests_unit {
    use super::super::*;
    use indoc::{formatdoc, indoc};
    use tempfile::{TempDir, tempdir};

    fn repo(name: &str) -> DiscoveredRepo {
        DiscoveredRepo {
            name: name.to_string(),
            short_name: name.to_string(),
            path: format!("../{name}"),
        }
    }

    fn repo_with_short(name: &str, short_name: &str) -> DiscoveredRepo {
        DiscoveredRepo {
            name: name.to_string(),
            short_name: short_name.to_string(),
            path: format!("../{name}"),
        }
    }

    fn spex_repo() -> DiscoveredRepo {
        DiscoveredRepo {
            name: "pizza-shop-spex".to_string(),
            short_name: "spex".to_string(),
            path: SPEX_REPO_PATH.to_string(),
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

    fn repo_names(repositories: &[DiscoveredRepo]) -> Vec<&str> {
        repositories.iter().map(|repo| repo.name.as_str()).collect()
    }

    #[test]
    fn discover_repositories_excludes_hidden_and_files() {
        let workspace = workspace_with(&["server", "client", "shop-spex", ".git"], &["notes.txt"]);

        let repositories =
            discover_repositories(workspace.path(), "shop-spex", "shop").expect("discover");

        assert_eq!(
            repo_names(&repositories),
            vec!["client", "server", "shop-spex"]
        );
    }

    #[test]
    fn discover_repositories_includes_spex_with_dot_path() {
        let workspace = workspace_with(&["server", "shop-spex"], &[]);

        let repositories =
            discover_repositories(workspace.path(), "shop-spex", "shop").expect("discover");

        let spex = repositories
            .iter()
            .find(|repo| repo.name == "shop-spex")
            .expect("spex repository");
        assert_eq!(spex.path, ".");
        assert_eq!(spex.short_name, "spex");
    }

    #[test]
    fn discover_repositories_derives_short_name_from_project_prefix() {
        let workspace = workspace_with(&["shop-api", "shop-spex"], &[]);

        let repositories =
            discover_repositories(workspace.path(), "shop-spex", "shop").expect("discover");

        let api = repositories
            .iter()
            .find(|repo| repo.name == "shop-api")
            .expect("api repository");
        assert_eq!(api.short_name, "api");
        assert_eq!(api.path, "../shop-api");
    }

    #[test]
    fn discover_repositories_sorts_by_name() {
        let workspace = workspace_with(&["zeta", "alpha", "mu"], &[]);

        let repositories =
            discover_repositories(workspace.path(), "shop-spex", "shop").expect("discover");

        assert_eq!(repo_names(&repositories), vec!["alpha", "mu", "zeta"]);
    }

    #[test]
    fn discover_repositories_sets_relative_path() {
        let workspace = workspace_with(&["server"], &[]);

        let repositories =
            discover_repositories(workspace.path(), "shop-spex", "shop").expect("discover");

        assert_eq!(repositories[0].path, "../server");
    }

    #[test]
    fn discover_repositories_kebab_cases_name_but_keeps_path() {
        let workspace = workspace_with(&["My_Repo Name!"], &[]);

        let repositories =
            discover_repositories(workspace.path(), "shop-spex", "shop").expect("discover");

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
        let yaml = config_to_yaml(
            "pizza-shop",
            &[
                repo_with_short("pizza-shop-server", "server"),
                repo_with_short("pizza-shop-client", "client"),
            ],
        );

        let expected = formatdoc! {"
            project_name: pizza-shop
            cli_version: {CLI_VERSION}
            repositories:
              - repository_name: pizza-shop-server
                repository_short_name: server
                path: ../pizza-shop-server
                rs_specs: []
              - repository_name: pizza-shop-client
                repository_short_name: client
                path: ../pizza-shop-client
                rs_specs: []
        "};
        assert_eq!(yaml, expected);
    }

    #[test]
    fn render_config_uses_empty_specs_for_spex_repo() {
        let yaml = config_to_yaml(
            "pizza-shop",
            &[repo_with_short("pizza-shop-server", "server"), spex_repo()],
        );

        let expected = formatdoc! {"
            project_name: pizza-shop
            cli_version: {CLI_VERSION}
            repositories:
              - repository_name: pizza-shop-server
                repository_short_name: server
                path: ../pizza-shop-server
                rs_specs: []
              - repository_name: pizza-shop-spex
                repository_short_name: spex
                description: >-
                  Spex workspace repository. Holds the specs and config that govern the
                  project: schemas/ (JSON Schemas for spex.yaml and the spec files),
                  rs-specs/ (cross-cutting conventions and rules) and fs-specs/ (the
                  functionality the system provides).
                path: .
                rs_specs: []
        "};
        assert_eq!(yaml, expected);
    }

    #[test]
    fn render_config_handles_no_repositories() {
        let yaml = config_to_yaml("pizza-shop", &[]);

        assert_eq!(
            yaml,
            format!("project_name: pizza-shop\ncli_version: {CLI_VERSION}\nrepositories: []\n")
        );
    }

    #[test]
    fn code_workspace_uses_short_name_and_icon_for_spex() {
        let json = code_workspace_json(&[
            repo_with_short("moodmart-backend", "backend"),
            repo_with_short("moodmart-proto", "proto"),
            spex_repo(),
        ]);

        let expected = indoc! {"
            {
              \"folders\": [
                { \"name\": \"backend\", \"path\": \"../moodmart-backend\" },
                { \"name\": \"proto\", \"path\": \"../moodmart-proto\" },
                { \"name\": \"\u{1f9f0} spex\", \"path\": \".\" },
              ]
            }
        "};
        assert_eq!(json, expected);
    }

    #[test]
    fn write_code_workspace_uses_project_name_file() {
        let spex_dir = tempdir().expect("create spex dir");

        write_code_workspace(spex_dir.path(), "pizza-shop", &[spex_repo()])
            .expect("write code-workspace");

        let path = spex_dir.path().join("pizza-shop.code-workspace");
        let written = fs::read_to_string(&path).expect("read code-workspace");
        assert_eq!(written, code_workspace_json(&[spex_repo()]));
    }

    #[test]
    fn write_config_writes_file_to_spex_dir() {
        let spex_dir = tempdir().expect("create spex dir");

        write_config(spex_dir.path(), "pizza-shop", &[repo("server")]).expect("write config");

        let config_path = spex_dir.path().join(SPEX_CONFIG_FILE);
        let written = fs::read_to_string(&config_path).expect("read config");
        assert_eq!(written, config_to_yaml("pizza-shop", &[repo("server")]));
    }

    #[test]
    fn write_repo_skills_writes_dummy_skill_per_repository() {
        let spex_dir = tempdir().expect("create spex dir");

        write_repo_skills(spex_dir.path(), &[repo("server")]).expect("write repo skills");

        let skill_md = spex_dir
            .path()
            .join(".agents")
            .join("skills")
            .join("server")
            .join("SKILL.md");
        let written = fs::read_to_string(&skill_md).expect("read skill");
        assert_eq!(written, repo_skill_md("server"));
    }

    #[test]
    fn write_repo_skills_skips_spex_repository() {
        let spex_dir = tempdir().expect("create spex dir");

        write_repo_skills(spex_dir.path(), &[spex_repo()]).expect("write repo skills");

        let skill_dir = spex_dir
            .path()
            .join(".agents")
            .join("skills")
            .join("pizza-shop-spex");
        assert!(!skill_dir.exists());
    }

    #[test]
    fn write_repo_skills_does_not_overwrite_existing_skill() {
        let spex_dir = tempdir().expect("create spex dir");
        let skill_dir = spex_dir.path().join(".agents").join("skills").join("server");
        fs::create_dir_all(&skill_dir).expect("create skill dir");
        fs::write(skill_dir.join("SKILL.md"), "custom content").expect("write skill");

        write_repo_skills(spex_dir.path(), &[repo("server")]).expect("write repo skills");

        let written = fs::read_to_string(skill_dir.join("SKILL.md")).expect("read skill");
        assert_eq!(written, "custom content");
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
    fn copy_templates_writes_rs_specs_and_scripts() {
        let spex_dir = tempdir().expect("create spex dir");

        copy_templates(spex_dir.path()).expect("copy templates");

        assert!(spex_dir.path().join("rs-specs").is_dir());
        assert!(spex_dir.path().join("fs-specs").is_dir());
        assert!(spex_dir.path().join("scripts").is_dir());
        assert!(spex_dir.path().join("plans").is_dir());
        assert!(spex_dir.path().join("contracts").is_dir());
    }

    #[test]
    fn copy_templates_writes_schemas() {
        let spex_dir = tempdir().expect("create spex dir");

        copy_templates(spex_dir.path()).expect("copy templates");

        assert!(spex_dir.path().join("schemas").is_dir());
        assert!(
            spex_dir
                .path()
                .join("schemas/fs-spec.schema.json")
                .is_file()
        );
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
                .join(".agents/skills/spex-functional-spec/SKILL.md")
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
            claude
                .join("skills/spex-functional-spec/SKILL.md")
                .is_file(),
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
            .join("scripts/rs-commit-message-format/commit-spec-id-trace.sh");
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
