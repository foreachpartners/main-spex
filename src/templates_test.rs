#[cfg(test)]
mod tests_unit {
    use super::super::*;
    use tempfile::tempdir;

    const SCHEMA: &str = "schemas/spex.schema.json";
    const SKILL: &str = ".agents/skills/spex-setup/SKILL.md";
    const SCRIPT: &str = "scripts/TECH-COMMIT-MESSAGE-FORMAT/commit-spec-id-trace.sh";
    const TECH_SPEC: &str = "tech-specs/TECH-SECRET-HANDLING.yaml";

    #[test]
    fn scaffold_writes_managed_content() {
        let spex_dir = tempdir().expect("create spex dir");

        scaffold(spex_dir.path()).expect("scaffold");

        for managed in [SCHEMA, SKILL, SCRIPT, TECH_SPEC] {
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
