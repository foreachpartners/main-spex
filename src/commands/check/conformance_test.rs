#[cfg(test)]
mod tests {
    use super::super::*;
    use tempfile::tempdir;

    #[test]
    fn no_duplicate_ids_passes_when_all_ids_are_unique() {
        let dir = tempdir().expect("temp dir");
        let rules = dir.path().join(RULES_DIR);
        std::fs::create_dir_all(rules.join("frontend")).expect("rules/frontend");
        std::fs::write(rules.join("frontend/one.yaml"), "id: ONE\n").expect("one");
        std::fs::write(rules.join("two.yaml"), "id: TWO\n").expect("two");

        let mut findings = Vec::new();
        validate_no_duplicate_ids(dir.path(), &mut findings);

        assert!(findings.is_empty());
    }

    #[test]
    fn no_duplicate_ids_flags_the_same_id_across_subfolders() {
        let dir = tempdir().expect("temp dir");
        let rules = dir.path().join(RULES_DIR);
        std::fs::create_dir_all(rules.join("frontend")).expect("rules/frontend");
        std::fs::create_dir_all(rules.join("backend")).expect("rules/backend");
        std::fs::write(rules.join("frontend/shared.yaml"), "id: SHARED\n").expect("frontend rule");
        std::fs::write(rules.join("backend/shared.yaml"), "id: SHARED\n").expect("backend rule");

        let mut findings = Vec::new();
        validate_no_duplicate_ids(dir.path(), &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].check, DUPLICATE_ID_CHECK);
        assert!(findings[0].message.contains("SHARED"));
    }

    #[test]
    fn no_duplicate_ids_flags_a_rule_id_colliding_with_a_spec_id() {
        let dir = tempdir().expect("temp dir");
        let specs = dir.path().join(SPECS_DIR);
        let rules = dir.path().join(RULES_DIR);
        std::fs::create_dir_all(&specs).expect("specs dir");
        std::fs::create_dir_all(&rules).expect("rules dir");
        std::fs::write(specs.join("shared.yaml"), "id: SHARED\n").expect("spec");
        std::fs::write(rules.join("shared.yaml"), "id: SHARED\n").expect("rule");

        let mut findings = Vec::new();
        validate_no_duplicate_ids(dir.path(), &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].check, DUPLICATE_ID_CHECK);
        assert!(findings[0].message.contains("SHARED"));
    }

    fn write_schemas(dir: &std::path::Path) {
        let schemas = dir.join(crate::templates::SCHEMAS_DIR);
        std::fs::create_dir_all(&schemas).expect("schemas dir");
        for schema in ["spec.schema.json", "rule.schema.json", "spex.schema.json"] {
            let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("templates/schemas")
                .join(schema);
            std::fs::copy(&src, schemas.join(schema)).unwrap_or_else(|_| panic!("copy {schema}"));
        }
    }

    #[test]
    fn property_order_passes_when_a_rule_matches_the_schema_order() {
        let dir = tempdir().expect("temp dir");
        write_schemas(dir.path());
        let rules = dir.path().join(RULES_DIR);
        std::fs::create_dir_all(&rules).expect("rules dir");
        std::fs::write(
            rules.join("ordered.yaml"),
            indoc::indoc! {r#"
                id: ordered
                title: Ordered rule
                description: A rule with properties in schema order.
                sync_with_upstream: true
                acceptance_criteria:
                  - "MUST do the thing"
                checks:
                  - name: check-one
                    description: Checks the thing
                    type: regexp
                    severity: warn
                    include:
                      - "**/*.rs"
                    pattern: "TODO"
            "#},
        )
        .expect("ordered rule");

        let mut findings = Vec::new();
        validate_property_order(dir.path(), &mut findings);

        assert!(findings.is_empty());
    }

    #[test]
    fn property_order_flags_a_top_level_field_out_of_order() {
        let dir = tempdir().expect("temp dir");
        write_schemas(dir.path());
        let rules = dir.path().join(RULES_DIR);
        std::fs::create_dir_all(&rules).expect("rules dir");
        std::fs::write(
            rules.join("misordered.yaml"),
            indoc::indoc! {r#"
                id: misordered
                sync_with_upstream: true
                title: Misordered rule
                description: sync_with_upstream comes before title and description here.
                acceptance_criteria:
                  - "MUST do the thing"
            "#},
        )
        .expect("misordered rule");

        let mut findings = Vec::new();
        validate_property_order(dir.path(), &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].check, PROPERTY_ORDER_CHECK);
        assert!(findings[0].message.contains("sync_with_upstream"));
    }

    #[test]
    fn property_order_ignores_a_check_items_own_field_order() {
        let dir = tempdir().expect("temp dir");
        write_schemas(dir.path());
        let rules = dir.path().join(RULES_DIR);
        std::fs::create_dir_all(&rules).expect("rules dir");
        std::fs::write(
            rules.join("misordered-check.yaml"),
            indoc::indoc! {r#"
                id: misordered-check
                title: Misordered check
                description: The check's own field order is not enforced, only the rule's top-level order.
                sync_with_upstream: true
                acceptance_criteria:
                  - "MUST do the thing"
                checks:
                  - name: check-one
                    description: Checks the thing
                    type: regexp
                    pattern: "TODO"
                    severity: warn
            "#},
        )
        .expect("misordered check");

        let mut findings = Vec::new();
        validate_property_order(dir.path(), &mut findings);

        assert!(findings.is_empty());
    }
}
