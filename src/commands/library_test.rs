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

    fn entries() -> Vec<Entry> {
        vec![
            entry("a", "id: a\n"),
            entry("b", "id: b\n"),
            entry("c", "id: c\n"),
        ]
    }

    #[test]
    fn spec_entries_are_ordered_by_id_and_keep_their_file_text() {
        let entries = spec_entries();

        let ids: Vec<&str> = entries.iter().map(|entry| entry.id.as_str()).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert!(!ids.is_empty(), "the library has no entries");
        assert_eq!(ids, sorted);
        for entry in &entries {
            assert!(entry.text.contains(&format!("id: {}\n", entry.id)));
        }
    }

    #[test]
    fn render_without_ids_prints_every_entry_in_full_as_a_list_separated_by_a_blank_line() {
        let printed = render(&entries(), &[], &[], &[]).expect("render");

        assert_eq!(printed, "- id: a\n\n- id: b\n\n- id: c\n");
    }

    #[test]
    fn render_with_ids_prints_entries_in_library_order_separated_by_a_blank_line() {
        let ids = vec!["c".to_string(), "a".to_string()];

        let printed = render(&entries(), &ids, &[], &[]).expect("render");

        assert_eq!(printed, "- id: a\n\n- id: c\n");
    }

    #[test]
    fn render_rejects_an_unknown_id() {
        let ids = vec!["a".to_string(), "nope".to_string()];

        let error = render(&entries(), &ids, &[], &[]).expect_err("unknown id");

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
    fn render_rejects_a_field_a_library_entry_does_not_have() {
        let error = render(&described_entries(), &[], &[], &names(&["id", "packages"]))
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
