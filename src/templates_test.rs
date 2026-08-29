#[cfg(test)]
mod tests_unit {
    use super::super::*;
    use std::path::Path;

    use serde_json::Value;
    use tempfile::tempdir;

    const SCHEMA: &str = "schemas/spex.schema.json";
    const SKILL: &str = ".agents/skills/spex-setup/SKILL.md";
    const SCRIPT: &str = "scripts/commit-message-format/commit-spec-id-trace.sh";
    const RULE: &str = "rules/secret-handling.yaml";
    // Spec/script pair sharing a spec id, used to test `sync_with_upstream`
    // gating together.
    const MANAGED_SPEC: &str = "rules/commit-message-format.yaml";
    const MANAGED_SPEC_ID: &str = "COMMIT-MESSAGE-FORMAT";

    #[test]
    fn scaffold_writes_managed_content() {
        let spex_dir = tempdir().expect("create spex dir");

        scaffold(spex_dir.path(), &[]).expect("scaffold");

        for managed in [SCHEMA, SKILL, SCRIPT, RULE] {
            assert!(
                spex_dir.path().join(managed).is_file(),
                "missing managed file `{managed}`"
            );
        }
    }

    #[test]
    fn scaffold_overwrites_modified_managed_file() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path(), &[]).expect("scaffold");

        let schema = spex_dir.path().join(SCHEMA);
        fs::write(&schema, "tampered").expect("modify schema");

        scaffold(spex_dir.path(), &[]).expect("re-scaffold");

        let expected = TEMPLATES
            .get_file(SCHEMA)
            .expect("bundled schema")
            .contents();
        assert_eq!(fs::read(&schema).expect("read schema"), expected);
    }

    #[test]
    fn scaffold_overwrites_rule_marked_managed() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path(), &[]).expect("scaffold");

        let spec = spex_dir.path().join(MANAGED_SPEC);
        fs::write(
            &spec,
            "id: COMMIT-MESSAGE-FORMAT\nsync_with_upstream: true\ntampered: true\n",
        )
        .expect("tamper spec, keeping sync_with_upstream");
        let script = spex_dir.path().join(SCRIPT);
        fs::write(&script, "tampered").expect("tamper script");

        scaffold(spex_dir.path(), &[]).expect("re-scaffold");

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
    fn scaffold_preserves_unmanaged_rule() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path(), &[]).expect("scaffold");

        let spec = spex_dir.path().join(MANAGED_SPEC);
        fs::write(&spec, "id: COMMIT-MESSAGE-FORMAT\ncustomized: true\n").expect("customize spec");
        let script = spex_dir.path().join(SCRIPT);
        fs::write(&script, "customized").expect("customize script");

        scaffold(spex_dir.path(), &[]).expect("re-scaffold");

        assert_eq!(
            fs::read_to_string(&spec).unwrap(),
            "id: COMMIT-MESSAGE-FORMAT\ncustomized: true\n"
        );
        assert_eq!(fs::read_to_string(&script).unwrap(), "customized");
    }

    #[test]
    fn scaffold_leaves_a_deliberately_deleted_rule_deleted() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path(), &[]).expect("scaffold");

        let spec = spex_dir.path().join(MANAGED_SPEC);
        fs::remove_file(&spec).expect("delete rule");

        // Listing the spec id in `rules_unused` records the deletion as
        // deliberate, so re-scaffolding must not resurrect it.
        scaffold(spex_dir.path(), &[MANAGED_SPEC_ID.to_string()]).expect("re-scaffold");

        assert!(
            !spec.exists(),
            "deliberately deleted rule should stay deleted"
        );
    }

    #[test]
    fn scaffold_deploys_a_missing_rule_not_marked_removed() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path(), &[]).expect("scaffold");

        let spec = spex_dir.path().join(MANAGED_SPEC);
        fs::remove_file(&spec).expect("delete rule");

        // Without the spec id in `rules_unused`, its absence is treated
        // as newly available upstream, so it is (re)deployed.
        scaffold(spex_dir.path(), &[]).expect("re-scaffold");

        assert!(
            spec.exists(),
            "rule missing but not marked removed should be deployed"
        );
    }

    #[test]
    fn scaffold_does_not_recreate_a_rule_moved_into_a_nested_folder() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path(), &[]).expect("scaffold");

        let flat_rule = spex_dir.path().join(RULE);
        let nested_rule = spex_dir
            .path()
            .join("rules")
            .join("security")
            .join("secret-handling.yaml");
        fs::create_dir_all(nested_rule.parent().unwrap()).expect("create nested folder");
        fs::rename(&flat_rule, &nested_rule).expect("move rule into nested folder");

        scaffold(spex_dir.path(), &[]).expect("re-scaffold");

        assert!(
            !flat_rule.exists(),
            "rule already present in a nested folder should not be recreated flat"
        );
        assert!(nested_rule.exists(), "nested rule should still be present");
    }

    #[test]
    fn scaffold_preserves_additional_files() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path(), &[]).expect("scaffold");

        let extra_skill = spex_dir
            .path()
            .join(AGENTS_DIR_NAME)
            .join(SKILLS_DIR)
            .join("custom-skill/SKILL.md");
        fs::create_dir_all(extra_skill.parent().unwrap()).expect("create custom skill dir");
        fs::write(&extra_skill, "custom").expect("write custom skill");

        let extra_script = spex_dir.path().join(SCRIPTS_DIR).join("custom.sh");
        fs::write(&extra_script, "echo custom").expect("write custom script");

        scaffold(spex_dir.path(), &[]).expect("re-scaffold");

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

        scaffold(spex_dir.path(), &[]).expect("scaffold");

        assert!(
            !spex_dir.path().join(AGENTS_FILE).exists(),
            "scaffold (update) must not create AGENTS.md"
        );
    }

    #[test]
    fn rule_templates_have_no_leftover_template_only_fields() {
        let dir = TEMPLATES.get_dir(RULES_DIR).expect("bundled rules dir");

        for file in dir.files() {
            let text = file.contents_utf8().expect("utf8 rule");
            assert!(
                !text.contains("first_created_in_spex_version"),
                "{} still carries the removed `first_created_in_spex_version` field",
                file.path().display()
            );
        }
    }

    #[test]
    fn rule_templates_conform_to_schema() {
        let schema_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("templates/schemas/rule.schema.json");
        let validator = crate::commands::check_schema::build_validator(&schema_path)
            .expect("build rule validator");

        let dir = TEMPLATES.get_dir(RULES_DIR).expect("bundled rules dir");

        for file in dir.files() {
            let text = file.contents_utf8().expect("utf8 rule");
            let instance: Value = serde_yaml_ng::from_str(text).expect("parse rule yaml");

            let messages = crate::commands::check_schema::validate_instance(&validator, &instance);
            assert!(
                messages.is_empty(),
                "{} violates rule.schema.json: {messages:?}",
                file.path().display()
            );
        }
    }

    #[test]
    fn scaffold_replaces_managed_skill_removing_stale_files() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path(), &[]).expect("scaffold");

        let stale = spex_dir
            .path()
            .join(AGENTS_DIR_NAME)
            .join(SKILLS_DIR)
            .join("spex-setup/STALE.md");
        fs::write(&stale, "stale").expect("write stale file");

        scaffold(spex_dir.path(), &[]).expect("re-scaffold");

        assert!(
            !stale.exists(),
            "stale managed skill file should be removed"
        );
        assert!(
            spex_dir.path().join(SKILL).is_file(),
            "managed skill should be restored"
        );
    }

    #[test]
    fn scaffold_preserves_skill_opted_out_of_sync_with_upstream() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path(), &[]).expect("scaffold");

        let skill = spex_dir.path().join(SKILL);
        fs::write(
            &skill,
            "---\nname: spex-setup\ndescription: customized\nmetadata:\n  sync-with-upstream: false\n---\ncustomized body\n",
        )
        .expect("opt skill out of sync");

        scaffold(spex_dir.path(), &[]).expect("re-scaffold");

        assert_eq!(
            fs::read_to_string(&skill).unwrap(),
            "---\nname: spex-setup\ndescription: customized\nmetadata:\n  sync-with-upstream: false\n---\ncustomized body\n"
        );
    }

    #[test]
    fn scaffold_overwrites_skill_without_explicit_opt_out() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path(), &[]).expect("scaffold");

        let skill = spex_dir.path().join(SKILL);
        fs::write(
            &skill,
            "---\nname: spex-setup\ndescription: tampered\n---\ntampered\n",
        )
        .expect("tamper skill");

        scaffold(spex_dir.path(), &[]).expect("re-scaffold");

        let expected = TEMPLATES
            .get_file("skills/spex-setup/SKILL.md")
            .expect("bundled skill")
            .contents();
        assert_eq!(fs::read(&skill).expect("read skill"), expected);
    }
}
