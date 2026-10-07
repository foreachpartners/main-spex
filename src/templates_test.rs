#[cfg(test)]
mod tests_unit {
    use super::super::*;
    use std::path::Path;

    use serde_json::Value;

    use crate::rule::RULES_DIR;

    #[test]
    fn bundled_docs_match_the_single_source_of_truth() {
        let root_docs = include_dir::include_dir!("$CARGO_MANIFEST_DIR/docs");
        let bundled = TEMPLATES_DIR
            .get_dir(DOCS_DIRNAME)
            .expect("bundled docs dir");

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
    fn rule_templates_have_no_leftover_template_only_fields() {
        let dir = TEMPLATES_DIR.get_dir(RULES_DIR).expect("bundled rules dir");

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

        let dir = TEMPLATES_DIR.get_dir(RULES_DIR).expect("bundled rules dir");

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
}
