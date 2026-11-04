#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::error::describe;

    fn entry(id: &str, text: &'static str) -> Entry {
        Entry {
            id: id.to_string(),
            text,
        }
    }

    fn sample_entries() -> Vec<Entry> {
        vec![
            entry("a", "id: a\n"),
            entry("b", "id: b\n"),
            entry("c", "id: c\n"),
        ]
    }

    #[test]
    fn entries_are_ordered_by_id_and_keep_their_file_text() {
        for dirname in [SPECS_LIBRARY_DIRNAME, RULES_LIBRARY_DIRNAME] {
            let entries = entries(dirname);

            let ids: Vec<&str> = entries.iter().map(|entry| entry.id.as_str()).collect();
            let mut sorted = ids.clone();
            sorted.sort_unstable();
            assert!(!ids.is_empty(), "the `{dirname}` library has no entries");
            assert_eq!(ids, sorted);
            for entry in &entries {
                assert!(entry.text.contains(&format!("id: {}\n", entry.id)));
            }
        }
    }

    #[test]
    fn rule_entries_hold_only_rule_files_not_their_script_folders() {
        let ids: Vec<String> = entries(RULES_LIBRARY_DIRNAME)
            .into_iter()
            .map(|entry| entry.id)
            .collect();

        assert!(ids.contains(&"secret-handling".to_string()));
        assert!(!ids.contains(&"commit-message-format/commit-spec-id-trace".to_string()));
        assert!(ids.iter().all(|id| !id.contains('/') && !id.contains('.')));
    }

    #[test]
    fn rules_accept_the_fields_of_the_rule_schema() {
        let fields = LibKind::Rules.fields();

        for field in ["id", "title", "sync_with_upstream", "checks", "examples"] {
            assert!(fields.contains(&field.to_string()), "{field}");
        }
        assert!(!fields.contains(&"path".to_string()));
    }

    #[test]
    fn specs_accept_only_the_partial_spec_fields() {
        assert_eq!(
            LibKind::Specs.fields(),
            ["id", "title", "description", "acceptance_criteria"]
        );
    }

    fn rule_ids(ids: &[&str]) -> Vec<String> {
        ids.iter().map(|id| id.to_string()).collect()
    }

    #[test]
    fn copy_writes_the_rule_and_its_executable_scripts_into_the_directory() {
        let dir = tempfile::tempdir().expect("tempdir");
        let rules = dir.path().join("new/rules");
        let entries = entries(RULES_LIBRARY_DIRNAME);

        copy_rules(
            LibKind::Rules,
            &entries,
            &rule_ids(&["secret-handling", "commit-message-format"]),
            &rules,
        )
        .expect("copy");

        let bundled = |name: &str| {
            BUNDLED_TEMPLATES_DIR
                .get_file(format!("{RULES_LIBRARY_DIRNAME}/{name}"))
                .expect("bundled file")
                .contents()
        };
        assert_eq!(
            std::fs::read(rules.join("secret-handling.yaml")).expect("read rule"),
            bundled("secret-handling.yaml")
        );
        assert!(!rules.join("secret-handling").exists());
        let script = rules.join("commit-message-format/commit-spec-id-trace.sh");
        assert_eq!(
            std::fs::read(&script).expect("read script"),
            bundled("commit-message-format/commit-spec-id-trace.sh")
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&script)
                .expect("metadata")
                .permissions()
                .mode();
            assert_ne!(mode & 0o111, 0, "script is not executable");
        }
    }

    #[test]
    fn copy_refuses_to_overwrite_and_writes_nothing() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("secret-handling.yaml"), "mine\n").expect("seed rule");
        let entries = entries(RULES_LIBRARY_DIRNAME);

        let error = copy_rules(
            LibKind::Rules,
            &entries,
            &rule_ids(&["commit-message-format", "secret-handling"]),
            dir.path(),
        )
        .expect_err("existing rule");

        assert!(
            describe(&error).contains("secret-handling.yaml"),
            "{error:?}"
        );
        assert!(describe(&error).contains("already exists"));
        assert_eq!(
            std::fs::read_to_string(dir.path().join("secret-handling.yaml")).unwrap(),
            "mine\n"
        );
        assert!(!dir.path().join("commit-message-format.yaml").exists());
    }

    #[test]
    fn copy_refuses_an_existing_script_folder_of_a_rule_with_scripts() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir(dir.path().join("naming-convention")).expect("seed folder");
        let entries = entries(RULES_LIBRARY_DIRNAME);

        let error = copy_rules(
            LibKind::Rules,
            &entries,
            &rule_ids(&["naming-convention"]),
            dir.path(),
        )
        .expect_err("existing script folder");

        assert!(describe(&error).contains("already exists"));
        assert!(!dir.path().join("naming-convention.yaml").exists());
    }

    #[test]
    fn copy_ignores_a_folder_named_like_a_rule_without_scripts() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir(dir.path().join("secret-handling")).expect("seed folder");
        let entries = entries(RULES_LIBRARY_DIRNAME);

        copy_rules(
            LibKind::Rules,
            &entries,
            &rule_ids(&["secret-handling"]),
            dir.path(),
        )
        .expect("copy");

        assert!(dir.path().join("secret-handling.yaml").is_file());
    }

    #[test]
    fn copy_rejects_an_unknown_id() {
        let dir = tempfile::tempdir().expect("tempdir");
        let entries = entries(RULES_LIBRARY_DIRNAME);

        let error = copy_rules(LibKind::Rules, &entries, &rule_ids(&["nope"]), dir.path())
            .expect_err("unknown id");

        assert_eq!(
            describe(&error),
            "unknown id `nope` — not found in the library"
        );
    }

    #[test]
    fn copy_rejects_specs() {
        let dir = tempfile::tempdir().expect("tempdir");
        let entries = entries(SPECS_LIBRARY_DIRNAME);

        let error = copy_rules(
            LibKind::Specs,
            &entries,
            &rule_ids(&["password-policy"]),
            dir.path(),
        )
        .expect_err("specs are not copied");

        assert!(describe(&error).contains("`--copy` copies rules only"));
    }

    #[test]
    fn render_without_ids_prints_every_entry_in_full_as_a_list_separated_by_a_blank_line() {
        let printed = render(&sample_entries(), &[], &[], &[]).expect("render");

        assert_eq!(printed, "- id: a\n\n- id: b\n\n- id: c\n");
    }

    #[test]
    fn render_with_ids_prints_entries_in_library_order_separated_by_a_blank_line() {
        let ids = vec!["c".to_string(), "a".to_string()];

        let printed = render(&sample_entries(), &ids, &[], &[]).expect("render");

        assert_eq!(printed, "- id: a\n\n- id: c\n");
    }

    #[test]
    fn render_rejects_an_unknown_id() {
        let ids = vec!["a".to_string(), "nope".to_string()];

        let error = render(&sample_entries(), &ids, &[], &[]).expect_err("unknown id");

        assert_eq!(
            describe(&error),
            "unknown id `nope` — not found in the library"
        );
    }

    fn described_entries() -> Vec<Entry> {
        vec![
            entry(
                "a",
                "id: a\ndescription: first\nacceptance_criteria:\n  - one\n",
            ),
            entry(
                "b",
                "id: b\ndescription: second\nacceptance_criteria:\n  - two\n",
            ),
        ]
    }

    fn names(fields: &[&str]) -> Vec<String> {
        fields.iter().map(|field| field.to_string()).collect()
    }

    #[test]
    fn render_with_fields_prints_only_those_fields_in_entry_order_separated_by_a_blank_line() {
        let printed = render(
            &described_entries(),
            &[],
            &[],
            &names(&["description", "id"]),
        )
        .expect("render");

        assert_eq!(
            printed,
            "- id: a\n  description: first\n\n- id: b\n  description: second\n"
        );
    }

    #[test]
    fn render_with_fields_and_ids_prints_only_the_named_entries() {
        let printed =
            render(&described_entries(), &names(&["b"]), &[], &names(&["id"])).expect("render");

        assert_eq!(printed, "- id: b\n");
    }

    #[test]
    fn a_field_a_library_entry_does_not_have_is_rejected_listing_the_kinds_fields() {
        let error = ensure_known_fields(
            &names(&["id", "packages"]),
            "library entry",
            &LibKind::Specs.fields(),
        )
        .expect_err("unknown field");

        assert_eq!(
            describe(&error),
            "unknown field `packages` — a library entry has: id, title, description, acceptance_criteria"
        );
    }

    #[test]
    fn render_accepts_an_entry_field_that_no_entry_sets() {
        let entries = vec![entry("a", "id: a\n"), entry("b", "id: b\ntitle: Second\n")];

        let printed = render(&entries, &names(&["a"]), &[], &names(&["title"])).expect("render");

        assert_eq!(printed, "- {}\n");
    }

    fn searchable_entries() -> Vec<Entry> {
        vec![
            entry(
                "email-policy",
                "id: email-policy\ndescription: Validates an address\ncriteria:\n  - must: lowercase the Domain\n",
            ),
            entry(
                "password-policy",
                "id: password-policy\ndescription: Hashes a secret\ncriteria:\n  - must: use Argon2id\n",
            ),
        ]
    }

    #[test]
    fn render_with_search_prints_entries_containing_a_keyword_ignoring_case_in_any_field_value() {
        let printed = render(&searchable_entries(), &[], &names(&["DOMAIN"]), &[]).expect("render");

        assert_eq!(
            printed,
            "- id: email-policy\n  description: Validates an address\n  criteria:\n  - must: lowercase the Domain\n"
        );
    }

    #[test]
    fn render_with_several_search_keywords_prints_entries_containing_any_of_them() {
        let printed = render(
            &searchable_entries(),
            &[],
            &names(&["argon2id", "address"]),
            &names(&["id"]),
        )
        .expect("render");

        assert_eq!(printed, "- id: email-policy\n\n- id: password-policy\n");
    }

    #[test]
    fn render_with_search_does_not_match_field_names() {
        let printed =
            render(&searchable_entries(), &[], &names(&["criteria"]), &[]).expect("render");

        assert_eq!(printed, "[]\n");
    }

    #[test]
    fn render_with_search_and_ids_prints_only_entries_in_both() {
        let printed = render(
            &searchable_entries(),
            &names(&["email-policy"]),
            &names(&["policy"]),
            &names(&["id"]),
        )
        .expect("render");

        assert_eq!(printed, "- id: email-policy\n");
    }

    #[test]
    fn render_with_search_matching_nothing_prints_an_empty_list() {
        let printed = render(&searchable_entries(), &[], &names(&["nope"]), &[]).expect("render");

        assert_eq!(printed, "[]\n");
    }
}
