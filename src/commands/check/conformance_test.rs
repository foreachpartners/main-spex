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
}
