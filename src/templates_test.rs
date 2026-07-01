#[cfg(test)]
mod tests_unit {
    use super::super::*;
    use tempfile::tempdir;

    const SCHEMA: &str = "schemas/spex.schema.json";
    const SKILL: &str = ".agents/skills/spex-setup/SKILL.md";
    const SCRIPT: &str = "scripts/ES-COMMIT-MESSAGE-FORMAT/commit-spec-id-trace.sh";
    const ES_SPEC: &str = "es-specs/ES-SECRET-HANDLING.yaml";
    // Managed spec/script pair sharing a spec id, used to test `managed`
    // gating together.
    const MANAGED_SPEC: &str = "es-specs/ES-COMMIT-MESSAGE-FORMAT.yaml";

    #[test]
    fn scaffold_writes_managed_content() {
        let spex_dir = tempdir().expect("create spex dir");

        scaffold(spex_dir.path()).expect("scaffold");

        for managed in [SCHEMA, SKILL, SCRIPT, ES_SPEC] {
            assert!(
                spex_dir.path().join(managed).is_file(),
                "missing managed file `{managed}`"
            );
        }
    }

    #[test]
    fn scaffold_overwrites_modified_managed_file() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path()).expect("scaffold");

        let schema = spex_dir.path().join(SCHEMA);
        fs::write(&schema, "tampered").expect("modify schema");

        scaffold(spex_dir.path()).expect("re-scaffold");

        let expected = TEMPLATES
            .get_file(SCHEMA)
            .expect("bundled schema")
            .contents();
        assert_eq!(fs::read(&schema).expect("read schema"), expected);
    }

    #[test]
    fn scaffold_overwrites_es_spec_marked_managed() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path()).expect("scaffold");

        let spec = spex_dir.path().join(MANAGED_SPEC);
        fs::write(
            &spec,
            "id: ES-COMMIT-MESSAGE-FORMAT\nmanaged: true\ntampered: true\n",
        )
        .expect("tamper spec, keeping managed");
        let script = spex_dir.path().join(SCRIPT);
        fs::write(&script, "tampered").expect("tamper script");

        scaffold(spex_dir.path()).expect("re-scaffold");

        let expected_spec = TEMPLATES
            .get_file(MANAGED_SPEC)
            .expect("bundled spec")
            .contents();
        assert_eq!(fs::read(&spec).expect("read spec"), expected_spec);
        let expected_script = TEMPLATES
            .get_file(SCRIPT)
            .expect("bundled script")
            .contents();
        assert_eq!(fs::read(&script).expect("read script"), expected_script);
    }

    #[test]
    fn scaffold_preserves_unmanaged_es_spec() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path()).expect("scaffold");

        let spec = spex_dir.path().join(MANAGED_SPEC);
        fs::write(&spec, "id: ES-COMMIT-MESSAGE-FORMAT\ncustomized: true\n")
            .expect("customize spec");
        let script = spex_dir.path().join(SCRIPT);
        fs::write(&script, "customized").expect("customize script");

        scaffold(spex_dir.path()).expect("re-scaffold");

        assert_eq!(
            fs::read_to_string(&spec).unwrap(),
            "id: ES-COMMIT-MESSAGE-FORMAT\ncustomized: true\n"
        );
        assert_eq!(fs::read_to_string(&script).unwrap(), "customized");
    }

    #[test]
    fn scaffold_preserves_additional_files() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path()).expect("scaffold");

        let extra_skill = spex_dir
            .path()
            .join(AGENTS_DIR_NAME)
            .join(SKILLS_DIR)
            .join("custom-skill/SKILL.md");
        fs::create_dir_all(extra_skill.parent().unwrap()).expect("create custom skill dir");
        fs::write(&extra_skill, "custom").expect("write custom skill");

        let extra_script = spex_dir.path().join(SCRIPTS_DIR).join("custom.sh");
        fs::write(&extra_script, "echo custom").expect("write custom script");

        scaffold(spex_dir.path()).expect("re-scaffold");

        assert_eq!(fs::read_to_string(&extra_skill).unwrap(), "custom");
        assert_eq!(fs::read_to_string(&extra_script).unwrap(), "echo custom");
    }

    #[test]
    fn ensure_agents_md_seeds_file_and_claude_link() {
        let spex_dir = tempdir().expect("create spex dir");

        ensure_agents_md(spex_dir.path()).expect("ensure agents md");

        let agents = spex_dir.path().join(AGENTS_FILE);
        let expected = TEMPLATES
            .get_file(AGENTS_FILE)
            .expect("bundled AGENTS.md")
            .contents();
        assert_eq!(fs::read(&agents).expect("read AGENTS.md"), expected);

        #[cfg(unix)]
        {
            let claude = spex_dir.path().join(CLAUDE_FILE);
            let target = fs::read_link(&claude).expect("read CLAUDE.md symlink");
            assert_eq!(target, Path::new(AGENTS_FILE));
        }
    }

    #[test]
    fn ensure_agents_md_preserves_user_edits() {
        let spex_dir = tempdir().expect("create spex dir");
        let agents = spex_dir.path().join(AGENTS_FILE);
        fs::write(&agents, "custom").expect("seed user AGENTS.md");

        ensure_agents_md(spex_dir.path()).expect("ensure agents md");

        assert_eq!(fs::read_to_string(&agents).unwrap(), "custom");
    }

    #[test]
    fn scaffold_leaves_agents_md_untouched() {
        let spex_dir = tempdir().expect("create spex dir");

        scaffold(spex_dir.path()).expect("scaffold");

        assert!(
            !spex_dir.path().join(AGENTS_FILE).exists(),
            "scaffold (update) must not create AGENTS.md"
        );
    }

    #[test]
    fn scaffold_replaces_managed_skill_removing_stale_files() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path()).expect("scaffold");

        let stale = spex_dir
            .path()
            .join(AGENTS_DIR_NAME)
            .join(SKILLS_DIR)
            .join("spex-setup/STALE.md");
        fs::write(&stale, "stale").expect("write stale file");

        scaffold(spex_dir.path()).expect("re-scaffold");

        assert!(
            !stale.exists(),
            "stale managed skill file should be removed"
        );
        assert!(
            spex_dir.path().join(SKILL).is_file(),
            "managed skill should be restored"
        );
    }
}
