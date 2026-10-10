#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn rule_id_accepts_lowercase_words_separated_by_single_hyphens() {
        assert!(RuleId::try_from("secret-handling".to_string()).is_ok());
        assert!(RuleId::try_from("a".to_string()).is_ok());
    }

    #[test]
    fn rule_id_accepts_digits_anywhere() {
        for valid in ["oauth2-scopes", "s3-buckets", "2fa-rules", "step-2", "123"] {
            assert!(
                RuleId::try_from(valid.to_string()).is_ok(),
                "`{valid}` should be accepted"
            );
        }
    }

    #[test]
    fn rule_id_accepts_an_id_of_the_maximum_length() {
        assert!(RuleId::try_from("a".repeat(ID_MAX_LENGTH)).is_ok());
    }

    #[test]
    fn rule_id_rejects_an_id_over_the_maximum_length() {
        assert!(RuleId::try_from("a".repeat(ID_MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn rule_id_rejects_ids_outside_the_grammar() {
        for invalid in [
            "",
            "Bad-Id",
            "bad_id",
            "-leading",
            "trailing-",
            "double--hyphen",
            "with space",
        ] {
            assert!(
                RuleId::try_from(invalid.to_string()).is_err(),
                "`{invalid}` should be rejected"
            );
        }
    }

    #[test]
    fn rule_id_validates_on_deserialize_and_writes_the_bare_id_on_serialize() {
        let id: RuleId = serde_yaml_ng::from_str("secret-handling").expect("valid id");

        assert_eq!(id.as_str(), "secret-handling");
        assert_eq!(id.to_string(), "secret-handling");
        assert_eq!(
            serde_yaml_ng::to_string(&id).expect("serialize"),
            "secret-handling\n"
        );
        assert!(serde_yaml_ng::from_str::<RuleId>("Bad_ID").is_err());
    }

    #[test]
    fn rule_id_error_names_the_kind_and_the_value() {
        let error = RuleId::try_from("Bad_ID".to_string()).expect_err("invalid id");

        assert!(error.to_string().starts_with("invalid rule id `Bad_ID`"));
    }

    #[test]
    fn grammar_matches_the_id_schemas() {
        let file = crate::templates::BUNDLED_TEMPLATES_DIR
            .get_file("schemas/rule-id.schema.json")
            .expect("bundled id schema");
        let schema_json: serde_json::Value =
            serde_json::from_str(file.contents_utf8().expect("utf8 schema")).expect("json");

        assert_eq!(schema_json["pattern"], ID_PATTERN, "rule-id pattern");
        assert_eq!(schema_json["maxLength"], ID_MAX_LENGTH, "rule-id maxLength");
    }
}
