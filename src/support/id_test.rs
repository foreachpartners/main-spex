#[cfg(test)]
mod tests {
    use super::super::*;

    use serde_json::Value;

    id_type!(TestId, "test id");

    #[test]
    fn parse_accepts_lowercase_words_separated_by_single_hyphens() {
        let id: TestId = "place-pizza-order".parse().expect("valid id");

        assert_eq!(id.as_str(), "place-pizza-order");
        assert!("a".parse::<TestId>().is_ok());
    }

    #[test]
    fn parse_accepts_an_id_of_the_maximum_length() {
        assert!("a".repeat(ID_MAX_LENGTH).parse::<TestId>().is_ok());
    }

    #[test]
    fn parse_rejects_an_id_over_the_maximum_length() {
        assert!("a".repeat(ID_MAX_LENGTH + 1).parse::<TestId>().is_err());
    }

    #[test]
    fn parse_rejects_ids_outside_the_grammar() {
        for invalid in [
            "",
            "Bad-Id",
            "bad_id",
            "delivery-time2",
            "-leading",
            "trailing-",
            "double--hyphen",
            "with space",
        ] {
            assert!(
                invalid.parse::<TestId>().is_err(),
                "`{invalid}` should be rejected"
            );
        }
    }

    #[test]
    fn parse_error_names_the_kind_and_the_value() {
        let error = "Bad_ID".parse::<TestId>().expect_err("invalid id");

        assert!(error.to_string().starts_with("invalid test id `Bad_ID`"));
    }

    #[test]
    fn deserialize_validates_the_id() {
        assert!(serde_yaml_ng::from_str::<TestId>("place-pizza-order").is_ok());
        assert!(serde_yaml_ng::from_str::<TestId>("Bad_ID").is_err());
    }

    #[test]
    fn serialize_writes_the_bare_id() {
        let id: TestId = "place-pizza-order".parse().expect("valid id");

        assert_eq!(
            serde_yaml_ng::to_string(&id).expect("serialize"),
            "place-pizza-order\n"
        );
    }

    #[test]
    fn grammar_matches_the_id_schemas() {
        for schema in ["spec-id", "ac-id", "rule-id"] {
            let file = crate::templates::BUNDLED_TEMPLATES_DIR
                .get_file(format!("schemas/{schema}.schema.json"))
                .expect("bundled id schema");
            let schema_json: Value =
                serde_json::from_str(file.contents_utf8().expect("utf8 schema")).expect("json");

            assert_eq!(schema_json["pattern"], ID_PATTERN, "{schema} pattern");
            assert_eq!(
                schema_json["maxLength"], ID_MAX_LENGTH,
                "{schema} maxLength"
            );
        }
    }
}
