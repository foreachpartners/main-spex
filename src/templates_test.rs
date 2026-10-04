#[cfg(test)]
mod tests_unit {
    use super::super::*;
    use std::path::Path;

    use serde_json::Value;
    use tempfile::tempdir;

    const SCHEMA: &str = "schemas/spex.schema.json";
    const DOC: &str = "docs/glossary.md";
    const SKILL: &str = ".agents/skills/spex-setup/SKILL.md";
    const SCRIPT: &str = "scripts/commit-message-format/commit-spec-id-trace.sh";
    const RULE: &str = "rules/secret-handling.yaml";
    // Spec/script pair sharing a spec id, used to test `sync_with_upstream`
    // gating together.
    const MANAGED_SPEC: &str = "rules/commit-message-format.yaml";
    const MANAGED_SPEC_ID: &str = "commit-message-format";

    #[test]
    fn scaffold_writes_managed_content() {
        let spex_dir = tempdir().expect("create spex dir");

        scaffold(spex_dir.path(), &[]).expect("scaffold");

        for managed in [SCHEMA, DOC, SKILL, SCRIPT, RULE] {
            assert!(
                spex_dir.path().join(managed).is_file(),
                "missing managed file `{managed}`"
            );
        }
    }

    #[test]
    fn scaffold_writes_bundled_docs_only() {
        let spex_dir = tempdir().expect("create spex dir");

        scaffold(spex_dir.path(), &[]).expect("scaffold");

        let docs = TEMPLATES.get_dir(DOCS_DIR).expect("bundled docs dir");
        for file in docs.files() {
            let name = file.path().file_name().expect("doc file name");
            let dest = spex_dir.path().join(DOCS_DIR).join(name);
            assert!(dest.is_file(), "missing bundled doc `{}`", dest.display());
            assert_eq!(
                fs::read(&dest).expect("read doc"),
                file.contents(),
                "`{}` differs from the bundled doc",
                dest.display()
            );
        }
        for name in [
            "conventions.md",
            "faq.md",
            "specification.md",
            "tutorial.md",
        ] {
            assert!(
                !spex_dir.path().join(DOCS_DIR).join(name).exists(),
                "`{name}` is not a bundled doc and must not scaffold"
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
    fn bundled_docs_match_the_single_source_of_truth() {
        let root_docs = include_dir::include_dir!("$CARGO_MANIFEST_DIR/docs");
        let bundled = TEMPLATES.get_dir(DOCS_DIR).expect("bundled docs dir");

        for file in bundled.files() {
            let name = file.path().file_name().expect("doc file name");
            let source = root_docs.get_file(name).unwrap_or_else(|| {
                panic!(
                    "bundled `docs/{}` has no source in the `docs/` folder",
                    name.to_string_lossy()
                )
            });
            assert_eq!(
                file.contents(),
                source.contents(),
                "bundled `docs/{}` differs from `docs/{}`",
                name.to_string_lossy(),
                name.to_string_lossy()
            );
        }
    }

    #[test]
    fn scaffold_overwrites_modified_managed_doc() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path(), &[]).expect("scaffold");

        let doc = spex_dir.path().join(DOC);
        fs::write(&doc, "tampered").expect("modify doc");

        scaffold(spex_dir.path(), &[]).expect("re-scaffold");

        let expected = TEMPLATES.get_file(DOC).expect("bundled doc").contents();
        assert_eq!(fs::read(&doc).expect("read doc"), expected);
    }

    #[test]
    fn scaffold_overwrites_rule_marked_managed() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path(), &[]).expect("scaffold");

        let spec = spex_dir.path().join(MANAGED_SPEC);
        fs::write(
            &spec,
            "id: commit-message-format\nsync_with_upstream: true\ntampered: true\n",
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
        fs::write(&spec, "id: commit-message-format\ncustomized: true\n").expect("customize spec");
        let script = spex_dir.path().join(SCRIPT);
        fs::write(&script, "customized").expect("customize script");

        scaffold(spex_dir.path(), &[]).expect("re-scaffold");

        assert_eq!(
            fs::read_to_string(&spec).unwrap(),
            "id: commit-message-format\ncustomized: true\n"
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
    fn scaffold_syncs_managed_section_of_agents_md_preserving_project_data() {
        let spex_dir = tempdir().expect("create spex dir");
        let agents = spex_dir.path().join(AGENTS_FILE);
        fs::write(
            &agents,
            "stale managed content\n\n---\n\n# Project\n\nmy custom notes\n",
        )
        .expect("seed AGENTS.md");

        scaffold(spex_dir.path(), &[]).expect("scaffold");

        let updated = fs::read_to_string(&agents).expect("read AGENTS.md");
        let template = TEMPLATES
            .get_file(AGENTS_FILE)
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
    fn scaffold_overwrites_agents_md_without_marker() {
        let spex_dir = tempdir().expect("create spex dir");
        let agents = spex_dir.path().join(AGENTS_FILE);
        fs::write(&agents, "a pre-split AGENTS.md with no marker\n").expect("seed AGENTS.md");

        scaffold(spex_dir.path(), &[]).expect("scaffold");

        let expected = TEMPLATES
            .get_file(AGENTS_FILE)
            .expect("bundled AGENTS.md")
            .contents();
        assert_eq!(
            fs::read(&agents).expect("read AGENTS.md"),
            expected,
            "AGENTS.md without the marker must be fully overwritten"
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
    fn scaffold_restores_bundled_skill_files_and_keeps_extra_ones() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path(), &[]).expect("scaffold");

        let skill_dir = spex_dir
            .path()
            .join(AGENTS_DIR_NAME)
            .join(SKILLS_DIR)
            .join("spex-setup");
        let bundled_reference = skill_dir.join("references/GENERIC-REPO-SKILL-TEMPLATE.md");
        let extra = skill_dir.join("notes.md");
        fs::write(&bundled_reference, "tampered").expect("tamper bundled reference");
        fs::write(&extra, "my notes").expect("write extra file");

        scaffold(spex_dir.path(), &[]).expect("re-scaffold");

        let expected = TEMPLATES
            .get_file("skills/spex-setup/references/GENERIC-REPO-SKILL-TEMPLATE.md")
            .expect("bundled reference")
            .contents();
        assert_eq!(fs::read(&bundled_reference).unwrap(), expected);
        assert_eq!(fs::read_to_string(&extra).unwrap(), "my notes");
    }

    #[test]
    fn bundled_skills_have_a_project_section() {
        let skills = TEMPLATES.get_dir(SKILLS_DIR).expect("bundled skills dir");

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
    fn scaffold_syncs_managed_section_of_skill_preserving_project_data() {
        let spex_dir = tempdir().expect("create spex dir");
        scaffold(spex_dir.path(), &[]).expect("scaffold");

        let skill = spex_dir.path().join(SKILL);
        fs::write(
            &skill,
            "stale managed content\n\n---\n\n# Project\n\nmy custom notes\n",
        )
        .expect("seed SKILL.md");

        scaffold(spex_dir.path(), &[]).expect("re-scaffold");

        let updated = fs::read_to_string(&skill).expect("read SKILL.md");
        let template = TEMPLATES
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
