#[cfg(test)]
mod tests {
    use super::super::*;
    use std::fs;
    use tempfile::tempdir;

    const ROOT_SCHEMA: &str = r#"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://spex.test/schemas/root.schema.json",
        "type": "object",
        "properties": {
            "id": { "$ref": "id.schema.json" }
        },
        "required": ["id"],
        "additionalProperties": false
    }"#;

    const ID_SCHEMA: &str = r#"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://spex.test/schemas/id.schema.json",
        "type": "string",
        "pattern": "^ES-[A-Z]+$"
    }"#;

    fn schema_dir() -> tempfile::TempDir {
        let dir = tempdir().expect("tempdir");
        fs::write(dir.path().join("root.schema.json"), ROOT_SCHEMA).expect("root");
        fs::write(dir.path().join("id.schema.json"), ID_SCHEMA).expect("id");
        dir
    }

    fn validator(dir: &tempfile::TempDir) -> Validator {
        build_validator(&dir.path().join("root.schema.json")).expect("validator")
    }

    fn write(dir: &tempfile::TempDir, name: &str, contents: &str) -> PathBuf {
        let path = dir.path().join(name);
        fs::write(&path, contents).expect("write instance");
        path
    }

    #[test]
    fn accepts_valid_instance_resolving_cross_file_ref() {
        let dir = schema_dir();
        let validator = validator(&dir);
        let instance = write(&dir, "ok.yaml", "id: ES-SECRET\n");

        let messages = validate_file(&validator, &instance).expect("validate");

        assert!(
            messages.is_empty(),
            "expected no violations, got {messages:?}"
        );
    }

    #[test]
    fn reports_pattern_violation_from_referenced_schema() {
        let dir = schema_dir();
        let validator = validator(&dir);
        let instance = write(&dir, "bad.yaml", "id: not-an-id\n");

        let messages = validate_file(&validator, &instance).expect("validate");

        assert!(
            messages
                .iter()
                .any(|message| message.contains("does not match")),
            "expected a pattern violation, got {messages:?}"
        );
    }

    #[test]
    fn reports_missing_required_and_unexpected_properties() {
        let dir = schema_dir();
        let validator = validator(&dir);
        let instance = write(&dir, "bad.yaml", "extra: true\n");

        let messages = validate_file(&validator, &instance).expect("validate");

        assert!(
            messages.iter().any(|message| message.contains("required")),
            "expected a required-property violation, got {messages:?}"
        );
        assert!(
            messages
                .iter()
                .any(|message| message.contains("Additional properties")),
            "expected an additional-properties violation, got {messages:?}"
        );
    }

    #[test]
    fn accepts_json_instance() {
        let dir = schema_dir();
        let validator = validator(&dir);
        let instance = write(&dir, "ok.json", r#"{ "id": "ES-SECRET" }"#);

        let messages = validate_file(&validator, &instance).expect("validate");

        assert!(
            messages.is_empty(),
            "expected no violations, got {messages:?}"
        );
    }

    #[test]
    fn errors_when_schema_is_missing() {
        let dir = tempdir().expect("tempdir");
        let result = build_validator(&dir.path().join("absent.schema.json"));

        assert!(result.is_err());
    }
}
