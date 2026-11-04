#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::error::describe;
    use crate::templates::{RULE_SCHEMA_FILENAME, SPEC_SCHEMA_FILENAME};
    use serde_json::json;
    use std::path::PathBuf;

    fn file(item: Value) -> YamlFile<Value> {
        YamlFile {
            item,
            path: PathBuf::from("/work/specs/a.yaml"),
            raw: String::new(),
        }
    }

    fn names(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    fn keys(value: &Value) -> Vec<&str> {
        value
            .as_object()
            .expect("object")
            .keys()
            .map(String::as_str)
            .collect()
    }

    #[test]
    fn with_path_adds_the_file_path_after_the_id() {
        let view = with_path(&file(json!({"title": "T", "id": "a", "description": "D"})));

        assert_eq!(keys(&view), ["title", "id", "path", "description"]);
        assert_eq!(view["path"], "/work/specs/a.yaml");
    }

    #[test]
    fn with_path_keeps_a_non_object_item_as_it_is() {
        let view = with_path(&file(json!(["a"])));

        assert_eq!(view, json!(["a"]));
    }

    #[test]
    fn keep_fields_keeps_the_named_fields_in_their_own_order() {
        let view = json!({"id": "a", "path": "/p", "title": "T"});

        let kept = keep_fields(view, &names(&["title", "id"]));

        assert_eq!(keys(&kept), ["id", "title"]);
    }

    #[test]
    fn entries_to_yaml_prints_each_entry_as_a_list_item_separated_by_a_blank_line() {
        let entries = [json!({"id": "a", "items": ["x"]}), json!({"id": "b"})];

        let yaml = entries_to_yaml(&entries, "specs").expect("yaml");

        assert_eq!(yaml, "- id: a\n  items:\n  - x\n\n- id: b\n");
    }

    #[test]
    fn entries_to_yaml_prints_an_empty_list_for_no_entries() {
        assert_eq!(entries_to_yaml(&[], "specs").expect("yaml"), "[]\n");
    }

    #[test]
    fn keep_fields_without_fields_keeps_everything() {
        let view = json!({"id": "a", "title": "T"});

        assert_eq!(keep_fields(view.clone(), &[]), view);
    }

    #[test]
    fn ensure_schema_fields_accepts_schema_fields_and_path() {
        let fields = names(&["id", "path", "acceptance_criteria", "blocked_by"]);

        ensure_schema_fields(&fields, "spec", SPEC_SCHEMA_FILENAME).expect("known fields");
    }

    #[test]
    fn ensure_schema_fields_rejects_a_field_of_another_kind() {
        let error = ensure_schema_fields(&names(&["checks"]), "spec", SPEC_SCHEMA_FILENAME)
            .expect_err("rule field on a spec");

        assert!(
            describe(&error).starts_with("unknown field `checks` — a spec has: id, "),
            "{}",
            describe(&error)
        );
        assert!(describe(&error).ends_with(", path"));
    }

    #[test]
    fn ensure_schema_fields_accepts_a_rule_field() {
        ensure_schema_fields(&names(&["checks"]), "rule", RULE_SCHEMA_FILENAME)
            .expect("rule field on a rule");
    }

    #[test]
    fn keywords_without_any_match_every_value() {
        assert!(Keywords::new(&[]).matches(&json!({"id": "a"})));
    }

    #[test]
    fn keywords_match_any_keyword_ignoring_case_in_nested_values() {
        let value = json!({"acceptance_criteria": [{"must": "Hash the Password"}]});

        assert!(Keywords::new(&names(&["nope", "PASSWORD"])).matches(&value));
        assert!(!Keywords::new(&names(&["nope"])).matches(&value));
    }

    #[test]
    fn keywords_do_not_match_field_names() {
        let value = json!({"acceptance_criteria": ["MUST hash"]});

        assert!(!Keywords::new(&names(&["acceptance_criteria"])).matches(&value));
    }
}
