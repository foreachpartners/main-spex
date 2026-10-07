#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn spec_id_accepts_lowercase_words_separated_by_single_hyphens() {
        assert!(SpecId::try_from("place-pizza-order".to_string()).is_ok());
        assert!(SpecId::try_from("a".to_string()).is_ok());
    }

    #[test]
    fn spec_id_accepts_an_id_of_the_maximum_length() {
        assert!(SpecId::try_from("a".repeat(ID_MAX_LENGTH)).is_ok());
    }

    #[test]
    fn spec_id_rejects_an_id_over_the_maximum_length() {
        assert!(SpecId::try_from("a".repeat(ID_MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn spec_id_rejects_ids_outside_the_grammar() {
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
                SpecId::try_from(invalid.to_string()).is_err(),
                "`{invalid}` should be rejected"
            );
        }
    }

    #[test]
    fn spec_id_validates_on_deserialize_and_writes_the_bare_id_on_serialize() {
        let id: SpecId = serde_yaml_ng::from_str("place-pizza-order").expect("valid id");

        assert_eq!(id.as_str(), "place-pizza-order");
        assert_eq!(id.to_string(), "place-pizza-order");
        assert_eq!(
            serde_yaml_ng::to_string(&id).expect("serialize"),
            "place-pizza-order\n"
        );
        assert!(serde_yaml_ng::from_str::<SpecId>("Bad_ID").is_err());
    }

    #[test]
    fn ac_id_validates_on_deserialize_and_writes_the_bare_id_on_serialize() {
        let id: AcId = serde_yaml_ng::from_str("in-stock").expect("valid id");

        assert_eq!(id.as_str(), "in-stock");
        assert_eq!(id.to_string(), "in-stock");
        assert_eq!(
            serde_yaml_ng::to_string(&id).expect("serialize"),
            "in-stock\n"
        );
        assert!(serde_yaml_ng::from_str::<AcId>("Bad_ID").is_err());
    }

    #[test]
    fn id_error_names_the_kind_and_the_value() {
        let error = AcId::try_from("Bad_ID".to_string()).expect_err("invalid id");

        assert!(
            error
                .to_string()
                .starts_with("invalid acceptance criterion id `Bad_ID`")
        );
    }

    #[test]
    fn grammar_matches_the_id_schemas() {
        for schema in ["spec-id", "ac-id"] {
            let file = crate::templates::BUNDLED_TEMPLATES_DIR
                .get_file(format!("schemas/{schema}.schema.json"))
                .expect("bundled id schema");
            let schema_json: serde_json::Value =
                serde_json::from_str(file.contents_utf8().expect("utf8 schema")).expect("json");

            assert_eq!(schema_json["pattern"], ID_PATTERN, "{schema} pattern");
            assert_eq!(
                schema_json["maxLength"], ID_MAX_LENGTH,
                "{schema} maxLength"
            );
        }
    }
}
