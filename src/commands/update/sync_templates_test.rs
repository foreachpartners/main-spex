#[cfg(test)]
mod tests_unit {
    use super::super::*;

    use tempfile::tempdir;

    use crate::commands::init::scaffold::scaffold;

    const SCHEMA: &str = "schemas/spex.schema.json";
    const DOC: &str = "docs/glossary.md";
    const SKILL: &str = ".agents/skills/spex-setup/SKILL.md";
    const SCRIPT: &str = "rules/commit-message-format/commit-spec-id-trace.sh";
    const RULE: &str = "rules/secret-handling.yaml";
    const MANAGED_SPEC: &str = "rules/commit-message-format.yaml";
    const MANAGED_SPEC_ID: &str = "commit-message-format";

    #[test]
    fn sync_overwrites_modified_managed_file() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path()).expect("scaffold");

        let schema = spex_dir.path().join(SCHEMA);
        fs::write(&schema, "tampered").expect("modify schema");

        sync_templates(spex_dir.path(), &[]).expect("sync");

        let expected = BUNDLED_TEMPLATES_DIR
            .get_file(SCHEMA)
            .expect("bundled schema")
            .contents();
        assert_eq!(fs::read(&schema).expect("read schema"), expected);
    }

    #[test]
    fn sync_overwrites_modified_managed_doc() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path()).expect("scaffold");

        let doc = spex_dir.path().join(DOC);
        fs::write(&doc, "tampered").expect("modify doc");

        sync_templates(spex_dir.path(), &[]).expect("sync");

        let expected = BUNDLED_TEMPLATES_DIR
            .get_file(DOC)
            .expect("bundled doc")
            .contents();
        assert_eq!(fs::read(&doc).expect("read doc"), expected);
    }

    #[test]
    fn sync_overwrites_rule_marked_managed() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path()).expect("scaffold");

        let spec = spex_dir.path().join(MANAGED_SPEC);
        fs::write(
            &spec,
            "id: commit-message-format\nsync_with_upstream: true\ntampered: true\n",
        )
        .expect("tamper spec, keeping sync_with_upstream");
        let script = spex_dir.path().join(SCRIPT);
        fs::write(&script, "tampered").expect("tamper script");

        sync_templates(spex_dir.path(), &[]).expect("sync");

        let expected_spec = BUNDLED_TEMPLATES_DIR
            .get_file(MANAGED_SPEC)
            .expect("bundled spec")
            .contents();
        assert_eq!(fs::read(&spec).expect("read spec"), expected_spec);
        let expected_script = BUNDLED_TEMPLATES_DIR
            .get_file(SCRIPT)
            .expect("bundled script")
            .contents();
        assert_eq!(fs::read(&script).expect("read script"), expected_script);
    }

    #[test]
    fn sync_preserves_unmanaged_rule() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path()).expect("scaffold");

        let spec = spex_dir.path().join(MANAGED_SPEC);
        fs::write(&spec, "id: commit-message-format\ncustomized: true\n").expect("customize spec");
        let script = spex_dir.path().join(SCRIPT);
        fs::write(&script, "customized").expect("customize script");

        sync_templates(spex_dir.path(), &[]).expect("sync");

        assert_eq!(
            fs::read_to_string(&spec).unwrap(),
            "id: commit-message-format\ncustomized: true\n"
        );
        assert_eq!(fs::read_to_string(&script).unwrap(), "customized");
    }

    #[test]
    fn sync_leaves_a_deliberately_deleted_rule_deleted() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path()).expect("scaffold");

        let spec = spex_dir.path().join(MANAGED_SPEC);
        fs::remove_file(&spec).expect("delete rule");

        sync_templates(spex_dir.path(), &[MANAGED_SPEC_ID.to_string()]).expect("sync");

        assert!(
            !spec.exists(),
            "deliberately deleted rule should stay deleted"
        );
    }

    #[test]
    fn sync_deploys_a_missing_rule_not_marked_removed() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path()).expect("scaffold");

        let spec = spex_dir.path().join(MANAGED_SPEC);
        fs::remove_file(&spec).expect("delete rule");

        sync_templates(spex_dir.path(), &[]).expect("sync");

        assert!(
            spec.exists(),
            "rule missing but not marked removed should be deployed"
        );
    }

    #[test]
    fn sync_does_not_recreate_a_rule_moved_into_a_nested_folder() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path()).expect("scaffold");

        let flat_rule = spex_dir.path().join(RULE);
        let nested_rule = spex_dir
            .path()
            .join("rules")
            .join("security")
            .join("secret-handling.yaml");
        fs::create_dir_all(nested_rule.parent().unwrap()).expect("create nested folder");
        fs::rename(&flat_rule, &nested_rule).expect("move rule into nested folder");

        sync_templates(spex_dir.path(), &[]).expect("sync");

        assert!(
            !flat_rule.exists(),
            "rule already present in a nested folder should not be recreated flat"
        );
        assert!(nested_rule.exists(), "nested rule should still be present");
    }

    #[test]
    fn sync_writes_the_scripts_of_a_nested_rule_beside_it() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path()).expect("scaffold");

        let rules = spex_dir.path().join("rules");
        let nested_rule = rules.join("policy").join("commit-message-format.yaml");
        fs::create_dir_all(nested_rule.parent().unwrap()).expect("create nested folder");
        fs::rename(spex_dir.path().join(MANAGED_SPEC), &nested_rule)
            .expect("move rule into nested folder");
        fs::remove_dir_all(rules.join(MANAGED_SPEC_ID)).expect("remove flat script folder");

        sync_templates(spex_dir.path(), &[]).expect("sync");

        assert!(
            nested_rule
                .with_extension("")
                .join("commit-spec-id-trace.sh")
                .is_file(),
            "script should be written beside the nested rule"
        );
        assert!(
            !rules.join(MANAGED_SPEC_ID).exists(),
            "script folder should not be recreated beside the flat rule path"
        );
    }

    #[test]
    fn sync_preserves_additional_files() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path()).expect("scaffold");

        let extra_skill = spex_dir
            .path()
            .join(AGENTS_DIRNAME)
            .join(SKILLS_DIRNAME)
            .join("custom-skill")
            .join(SKILL_FILE);
        fs::create_dir_all(extra_skill.parent().unwrap()).expect("create custom skill dir");
        fs::write(&extra_skill, "custom").expect("write custom skill");

        let extra_script = spex_dir
            .path()
            .join("rules/commit-message-format/custom.sh");
        fs::write(&extra_script, "echo custom").expect("write custom script");

        sync_templates(spex_dir.path(), &[]).expect("sync");

        assert_eq!(fs::read_to_string(&extra_skill).unwrap(), "custom");
        assert_eq!(fs::read_to_string(&extra_script).unwrap(), "echo custom");
    }

    #[test]
    fn sync_does_not_seed_agents_md() {
        let spex_dir = tempdir().expect("create spex dir");

        sync_templates(spex_dir.path(), &[]).expect("sync");

        assert!(
            !spex_dir.path().join(AGENTS_FILENAME).exists(),
            "sync (update) must not create AGENTS.md"
        );
    }

    #[test]
    fn sync_syncs_managed_section_of_agents_md_preserving_project_data() {
        let spex_dir = tempdir().expect("create spex dir");
        let agents = spex_dir.path().join(AGENTS_FILENAME);
        fs::write(
            &agents,
            "stale managed content\n\n---\n\n# Project\n\nmy custom notes\n",
        )
        .expect("seed AGENTS.md");

        sync_templates(spex_dir.path(), &[]).expect("sync");

        let updated = fs::read_to_string(&agents).expect("read AGENTS.md");
        let template = BUNDLED_TEMPLATES_DIR
            .get_file(AGENTS_FILENAME)
            .expect("bundled AGENTS.md")
            .contents_utf8()
            .expect("utf8 AGENTS.md");
        let (managed, _) = template
            .split_once("\n---\n\n# Project")
            .expect("bundled AGENTS.md has the project marker");

        assert!(updated.starts_with(managed), "managed section not synced");
        assert!(
            updated.ends_with("# Project\n\nmy custom notes\n"),
            "project-specific section not preserved"
        );
    }

    #[test]
    fn sync_overwrites_agents_md_without_marker() {
        let spex_dir = tempdir().expect("create spex dir");
        let agents = spex_dir.path().join(AGENTS_FILENAME);
        fs::write(&agents, "a pre-split AGENTS.md with no marker\n").expect("seed AGENTS.md");

        sync_templates(spex_dir.path(), &[]).expect("sync");

        let expected = BUNDLED_TEMPLATES_DIR
            .get_file(AGENTS_FILENAME)
            .expect("bundled AGENTS.md")
            .contents();
        assert_eq!(
            fs::read(&agents).expect("read AGENTS.md"),
            expected,
            "AGENTS.md without the marker must be fully overwritten"
        );
    }

    #[test]
    fn sync_restores_bundled_skill_files_and_keeps_extra_ones() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path()).expect("scaffold");

        let skill_dir = spex_dir
            .path()
            .join(AGENTS_DIRNAME)
            .join(SKILLS_DIRNAME)
            .join("spex-setup");
        let bundled_reference = skill_dir.join("references/GENERIC-REPO-SKILL-TEMPLATE.md");
        let extra = skill_dir.join("notes.md");
        fs::write(&bundled_reference, "tampered").expect("tamper bundled reference");
        fs::write(&extra, "my notes").expect("write extra file");

        sync_templates(spex_dir.path(), &[]).expect("sync");

        let expected = BUNDLED_TEMPLATES_DIR
            .get_file("skills/spex-setup/references/GENERIC-REPO-SKILL-TEMPLATE.md")
            .expect("bundled reference")
            .contents();
        assert_eq!(fs::read(&bundled_reference).unwrap(), expected);
        assert_eq!(fs::read_to_string(&extra).unwrap(), "my notes");
    }

    #[test]
    fn bundled_skills_have_a_project_section() {
        let skills = BUNDLED_TEMPLATES_DIR
            .get_dir(SKILLS_DIRNAME)
            .expect("bundled skills dir");

        for skill in skills.dirs() {
            let skill_md = skill
                .get_file(skill.path().join(SKILL_FILE))
                .expect("bundled SKILL.md")
                .contents_utf8()
                .expect("utf8 SKILL.md");
            assert!(
                skill_md.contains(PROJECT_SECTION_MARKER),
                "{} lacks the project section marker",
                skill.path().display()
            );
        }
    }

    #[test]
    fn sync_syncs_managed_section_of_skill_preserving_project_data() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path()).expect("scaffold");

        let skill = spex_dir.path().join(SKILL);
        fs::write(
            &skill,
            "stale managed content\n\n---\n\n# Project\n\nmy custom notes\n",
        )
        .expect("seed SKILL.md");

        sync_templates(spex_dir.path(), &[]).expect("sync");

        let updated = fs::read_to_string(&skill).expect("read SKILL.md");
        let template = BUNDLED_TEMPLATES_DIR
            .get_file("skills/spex-setup/SKILL.md")
            .expect("bundled skill")
            .contents_utf8()
            .expect("utf8 SKILL.md");
        let (managed, _) = template
            .split_once(PROJECT_SECTION_MARKER)
            .expect("bundled SKILL.md has the project marker");

        assert!(updated.starts_with(managed), "managed section not synced");
        assert!(
            updated.ends_with("# Project\n\nmy custom notes\n"),
            "project-specific section not preserved"
        );
    }

    #[test]
    fn sync_preserves_skill_opted_out_of_sync_with_upstream() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path()).expect("scaffold");

        let skill = spex_dir.path().join(SKILL);
        fs::write(
            &skill,
            "---\nname: spex-setup\ndescription: customized\nmetadata:\n  sync-with-upstream: false\n---\ncustomized body\n",
        )
        .expect("opt skill out of sync");

        sync_templates(spex_dir.path(), &[]).expect("sync");

        assert_eq!(
            fs::read_to_string(&skill).unwrap(),
            "---\nname: spex-setup\ndescription: customized\nmetadata:\n  sync-with-upstream: false\n---\ncustomized body\n"
        );
    }

    #[test]
    fn sync_overwrites_skill_without_explicit_opt_out() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path()).expect("scaffold");

        let skill = spex_dir.path().join(SKILL);
        fs::write(
            &skill,
            "---\nname: spex-setup\ndescription: tampered\n---\ntampered\n",
        )
        .expect("tamper skill");

        sync_templates(spex_dir.path(), &[]).expect("sync");

        let expected = BUNDLED_TEMPLATES_DIR
            .get_file("skills/spex-setup/SKILL.md")
            .expect("bundled skill")
            .contents();
        assert_eq!(fs::read(&skill).expect("read skill"), expected);
    }
}
