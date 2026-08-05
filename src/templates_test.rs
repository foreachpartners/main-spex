#[cfg(test)]
mod tests_unit {
    use super::super::*;
    use std::path::Path;

    use serde_json::Value;
    use tempfile::tempdir;

    const SCHEMA: &str = "schemas/spex.schema.json";
    const SKILL: &str = ".agents/skills/spex-setup/SKILL.md";
    const SCRIPT: &str = "scripts/rs-commit-message-format/commit-spec-id-trace.sh";
    const RS_SPEC: &str = "rs-specs/rs-secret-handling.yaml";
    // Spec/script pair sharing a spec id, used to test `sync_with_upstream`
    // gating together.
    const MANAGED_SPEC: &str = "rs-specs/rs-commit-message-format.yaml";

    #[test]
    fn scaffold_writes_managed_content() {
        let spex_dir = tempdir().expect("create spex dir");

        scaffold(spex_dir.path(), BEFORE_ANY_SPEX_VERSION).expect("scaffold");

        for managed in [SCHEMA, SKILL, SCRIPT, RS_SPEC] {
            assert!(
                spex_dir.path().join(managed).is_file(),
                "missing managed file `{managed}`"
            );
        }
    }

    #[test]
    fn scaffold_overwrites_modified_managed_file() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path(), BEFORE_ANY_SPEX_VERSION).expect("scaffold");

        let schema = spex_dir.path().join(SCHEMA);
        fs::write(&schema, "tampered").expect("modify schema");

        scaffold(spex_dir.path(), BEFORE_ANY_SPEX_VERSION).expect("re-scaffold");

        let expected = TEMPLATES
            .get_file(SCHEMA)
            .expect("bundled schema")
            .contents();
        assert_eq!(fs::read(&schema).expect("read schema"), expected);
    }

    #[test]
    fn scaffold_overwrites_rs_spec_marked_managed() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path(), BEFORE_ANY_SPEX_VERSION).expect("scaffold");

        let spec = spex_dir.path().join(MANAGED_SPEC);
        fs::write(
            &spec,
            "id: RS-COMMIT-MESSAGE-FORMAT\nsync_with_upstream: true\ntampered: true\n",
        )
        .expect("tamper spec, keeping sync_with_upstream");
        let script = spex_dir.path().join(SCRIPT);
        fs::write(&script, "tampered").expect("tamper script");

        scaffold(spex_dir.path(), BEFORE_ANY_SPEX_VERSION).expect("re-scaffold");

        let expected_spec = strip_template_only_fields(
            TEMPLATES
                .get_file(MANAGED_SPEC)
                .expect("bundled spec")
                .contents(),
        );
        assert_eq!(fs::read(&spec).expect("read spec"), expected_spec);
        let expected_script = TEMPLATES
            .get_file(SCRIPT)
            .expect("bundled script")
            .contents();
        assert_eq!(fs::read(&script).expect("read script"), expected_script);
    }

    #[test]
    fn scaffold_preserves_unmanaged_rs_spec() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path(), BEFORE_ANY_SPEX_VERSION).expect("scaffold");

        let spec = spex_dir.path().join(MANAGED_SPEC);
        fs::write(&spec, "id: RS-COMMIT-MESSAGE-FORMAT\ncustomized: true\n")
            .expect("customize spec");
        let script = spex_dir.path().join(SCRIPT);
        fs::write(&script, "customized").expect("customize script");

        scaffold(spex_dir.path(), BEFORE_ANY_SPEX_VERSION).expect("re-scaffold");

        assert_eq!(
            fs::read_to_string(&spec).unwrap(),
            "id: RS-COMMIT-MESSAGE-FORMAT\ncustomized: true\n"
        );
        assert_eq!(fs::read_to_string(&script).unwrap(), "customized");
    }

    #[test]
    fn scaffold_leaves_a_deliberately_deleted_rs_spec_deleted() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path(), BEFORE_ANY_SPEX_VERSION).expect("scaffold");

        let spec = spex_dir.path().join(MANAGED_SPEC);
        fs::remove_file(&spec).expect("delete rs-spec");

        // RS-COMMIT-MESSAGE-FORMAT's `first_created_in_spex_version` is 0.1.0,
        // so re-scaffolding as of a later version must not resurrect it.
        scaffold(spex_dir.path(), "1.0.0").expect("re-scaffold");

        assert!(
            !spec.exists(),
            "deliberately deleted rs-spec should stay deleted"
        );
    }

    #[test]
    fn scaffold_deploys_an_rs_spec_new_since_the_recorded_version() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path(), BEFORE_ANY_SPEX_VERSION).expect("scaffold");

        let spec = spex_dir.path().join(MANAGED_SPEC);
        fs::remove_file(&spec).expect("delete rs-spec");

        // A version older than RS-COMMIT-MESSAGE-FORMAT's
        // `first_created_in_spex_version` (0.1.0) means the spec is new since
        // the repo's last update, so it must be (re)deployed.
        scaffold(spex_dir.path(), "0.0.5").expect("re-scaffold");

        assert!(
            spec.exists(),
            "rs-spec new since the recorded version should be deployed"
        );
    }

    #[test]
    fn scaffold_preserves_additional_files() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path(), BEFORE_ANY_SPEX_VERSION).expect("scaffold");

        let extra_skill = spex_dir
            .path()
            .join(AGENTS_DIR_NAME)
            .join(SKILLS_DIR)
            .join("custom-skill/SKILL.md");
        fs::create_dir_all(extra_skill.parent().unwrap()).expect("create custom skill dir");
        fs::write(&extra_skill, "custom").expect("write custom skill");

        let extra_script = spex_dir.path().join(SCRIPTS_DIR).join("custom.sh");
        fs::write(&extra_script, "echo custom").expect("write custom script");

        scaffold(spex_dir.path(), BEFORE_ANY_SPEX_VERSION).expect("re-scaffold");

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

        scaffold(spex_dir.path(), BEFORE_ANY_SPEX_VERSION).expect("scaffold");

        assert!(
            !spex_dir.path().join(AGENTS_FILE).exists(),
            "scaffold (update) must not create AGENTS.md"
        );
    }

    #[test]
    fn scaffold_strips_template_only_inclusion_fields_from_copied_rs_spec() {
        let spex_dir = tempdir().expect("create spex dir");

        scaffold(spex_dir.path(), BEFORE_ANY_SPEX_VERSION).expect("scaffold");

        let spec = fs::read_to_string(spex_dir.path().join(MANAGED_SPEC)).expect("read spec");
        assert!(
            !spec.contains("first_created_in_spex_version"),
            "copied rs-spec should not carry template-only inclusion fields:\n{spec}"
        );
    }

    // Every bundled rs-spec must explicitly set the template-only field
    // that gates deployment (`first_created_in_spex_version`), rather than
    // relying on a serde default, so `spex update`'s new-vs-deleted
    // deployment decision (see `copy_rule_specs`) is always deliberate, not
    // accidental.
    const REQUIRED_TEMPLATE_ONLY_FIELDS: [&str; 1] = ["first_created_in_spex_version"];

    #[test]
    fn rs_spec_templates_set_required_template_only_fields() {
        let dir = TEMPLATES
            .get_dir(RULE_SPECS_DIR)
            .expect("bundled rs-specs dir");

        for file in dir.files() {
            let text = file.contents_utf8().expect("utf8 rs-spec");
            let value: Value = serde_yaml_ng::from_str(text).expect("parse rs-spec yaml");
            let object = value.as_object().expect("rs-spec is a yaml mapping");

            for field in REQUIRED_TEMPLATE_ONLY_FIELDS {
                assert!(
                    object.contains_key(field),
                    "{} is missing required template-only field `{field}`",
                    file.path().display()
                );
            }
        }
    }

    // Every bundled rs-spec must satisfy `rs-spec.schema.json` once its
    // template-only inclusion fields are stripped, since that stripped form
    // is exactly what `spex init`/`update` write into a repository's own
    // `rs-specs/` (see `copy_rule_specs`).
    #[test]
    fn rs_spec_templates_conform_to_schema_once_template_only_fields_are_stripped() {
        let validator = crate::commands::check_schema::build_validator(Path::new(
            "templates/schemas/rs-spec.schema.json",
        ))
        .expect("build rs-spec validator");

        let dir = TEMPLATES
            .get_dir(RULE_SPECS_DIR)
            .expect("bundled rs-specs dir");

        for file in dir.files() {
            let stripped = strip_template_only_fields(file.contents());
            let text = std::str::from_utf8(&stripped).expect("utf8 rs-spec");
            let instance: Value = serde_yaml_ng::from_str(text).expect("parse rs-spec yaml");

            let messages = crate::commands::check_schema::validate_instance(&validator, &instance);
            assert!(
                messages.is_empty(),
                "{} violates rs-spec.schema.json: {messages:?}",
                file.path().display()
            );
        }
    }

    #[test]
    fn scaffold_replaces_managed_skill_removing_stale_files() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path(), BEFORE_ANY_SPEX_VERSION).expect("scaffold");

        let stale = spex_dir
            .path()
            .join(AGENTS_DIR_NAME)
            .join(SKILLS_DIR)
            .join("spex-setup/STALE.md");
        fs::write(&stale, "stale").expect("write stale file");

        scaffold(spex_dir.path(), BEFORE_ANY_SPEX_VERSION).expect("re-scaffold");

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
        scaffold(spex_dir.path(), BEFORE_ANY_SPEX_VERSION).expect("scaffold");

        let skill = spex_dir.path().join(SKILL);
        fs::write(
            &skill,
            "---\nname: spex-setup\ndescription: customized\nmetadata:\n  sync-with-upstream: false\n---\ncustomized body\n",
        )
        .expect("opt skill out of sync");

        scaffold(spex_dir.path(), BEFORE_ANY_SPEX_VERSION).expect("re-scaffold");

        assert_eq!(
            fs::read_to_string(&skill).unwrap(),
            "---\nname: spex-setup\ndescription: customized\nmetadata:\n  sync-with-upstream: false\n---\ncustomized body\n"
        );
    }

    #[test]
    fn scaffold_overwrites_skill_without_explicit_opt_out() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path(), BEFORE_ANY_SPEX_VERSION).expect("scaffold");

        let skill = spex_dir.path().join(SKILL);
        fs::write(
            &skill,
            "---\nname: spex-setup\ndescription: tampered\n---\ntampered\n",
        )
        .expect("tamper skill");

        scaffold(spex_dir.path(), BEFORE_ANY_SPEX_VERSION).expect("re-scaffold");

        let expected = TEMPLATES
            .get_file("skills/spex-setup/SKILL.md")
            .expect("bundled skill")
            .contents();
        assert_eq!(fs::read(&skill).expect("read skill"), expected);
    }
}
