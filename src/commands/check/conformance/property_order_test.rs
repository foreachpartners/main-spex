#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::commands::check::conformance::conformance::RULE_ID;
    use tempfile::tempdir;

    fn write_schemas(dir: &std::path::Path) {
        let schemas = dir.join(crate::templates::SCHEMAS_DIRNAME);
        std::fs::create_dir_all(&schemas).expect("schemas dir");
        let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("templates/schemas");
        for entry in std::fs::read_dir(&src_dir).expect("read templates/schemas") {
            let entry = entry.expect("dir entry");
            let name = entry.file_name();
            std::fs::copy(entry.path(), schemas.join(&name))
                .unwrap_or_else(|_| panic!("copy {}", name.to_string_lossy()));
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
                sync_with_upstream: true
                description: A rule with properties in schema order.
                acceptance_criteria:
                  - "MUST do the thing"
                checks:
                  - name: check-one
                    description: Checks the thing
                    type: regexp
                    pattern: "TODO"
                    include:
                      - "**/*.rs"
                    severity: warn
            "#},
        )
        .expect("ordered rule");

        let findings = validate_property_order(dir.path()).expect("validate");

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

        let findings = validate_property_order(dir.path()).expect("validate");

        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].source.baseline_key(),
            Some((RULE_ID, PROPERTY_ORDER_CHECK))
        );
        assert!(findings[0].message.contains("sync_with_upstream"));
    }

    #[test]
    fn property_order_passes_when_a_checks_item_matches_the_schema_order() {
        let dir = tempdir().expect("temp dir");
        write_schemas(dir.path());
        let rules = dir.path().join(RULES_DIR);
        std::fs::create_dir_all(&rules).expect("rules dir");
        std::fs::write(
            rules.join("ordered-check.yaml"),
            indoc::indoc! {r#"
                id: ordered-check
                title: Ordered check
                sync_with_upstream: true
                description: The check's own fields are in the schema-declared order.
                acceptance_criteria:
                  - "MUST do the thing"
                checks:
                  - name: check-one
                    description: Checks the thing
                    type: regexp
                    pattern: "TODO"
                    include:
                      - "**/*.rs"
                    severity: warn
            "#},
        )
        .expect("ordered check");

        let findings = validate_property_order(dir.path()).expect("validate");

        assert!(findings.is_empty());
    }

    #[test]
    fn property_order_flags_a_nested_checks_item_field_out_of_order() {
        let dir = tempdir().expect("temp dir");
        write_schemas(dir.path());
        let rules = dir.path().join(RULES_DIR);
        std::fs::create_dir_all(&rules).expect("rules dir");
        std::fs::write(
            rules.join("misordered-check.yaml"),
            indoc::indoc! {r#"
                id: misordered-check
                title: Misordered check
                sync_with_upstream: true
                description: A check's own field order is now enforced too, not only the rule's top-level order.
                acceptance_criteria:
                  - "MUST do the thing"
                checks:
                  - name: check-one
                    description: Checks the thing
                    type: regexp
                    severity: warn
                    pattern: "TODO"
            "#},
        )
        .expect("misordered check");

        let findings = validate_property_order(dir.path()).expect("validate");

        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].source.baseline_key(),
            Some((RULE_ID, PROPERTY_ORDER_CHECK))
        );
        assert!(findings[0].message.contains("checks[0]"));
        assert!(findings[0].message.contains("severity"));
    }

    #[test]
    fn property_order_flags_a_nested_repository_field_out_of_order_in_spex_yaml() {
        let dir = tempdir().expect("temp dir");
        write_schemas(dir.path());
        std::fs::write(
            dir.path().join(crate::config::SPEX_CONFIG_FILE),
            indoc::indoc! {r#"
                project_name: pizza
                cli_version: 0.1.0
                repositories:
                  - repository_short_name: api
                    repository_name: pizza-api
                    path: ../pizza-api
            "#},
        )
        .expect("spex.yaml");

        let findings = validate_property_order(dir.path()).expect("validate");

        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].source.baseline_key(),
            Some((RULE_ID, PROPERTY_ORDER_CHECK))
        );
        assert!(findings[0].message.contains("repositories[0]"));
        assert!(findings[0].message.contains("repository_name"));
    }
}
