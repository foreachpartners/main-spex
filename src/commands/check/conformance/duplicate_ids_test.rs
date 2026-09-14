#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::rule;
    use crate::spec;
    use tempfile::tempdir;

    #[test]
    fn no_duplicate_ids_passes_when_all_ids_are_unique() {
        let dir = tempdir().expect("temp dir");
        let rules_dir = dir.path().join(rule::RULES_DIR);
        std::fs::create_dir_all(rules_dir.join("frontend")).expect("rules/frontend");
        std::fs::write(rules_dir.join("frontend/one.yaml"), "id: ONE\n").expect("one");
        std::fs::write(rules_dir.join("two.yaml"), "id: TWO\n").expect("two");

        let specs = spec::load_all(&dir.path().join(spec::SPECS_DIR)).expect("specs");
        let rules = rule::load_all_list(&rules_dir).expect("rules");
        let mut findings = Vec::new();
        validate_no_duplicate_ids(&specs, &rules, &mut findings);

        assert!(findings.is_empty());
    }

    #[test]
    fn no_duplicate_ids_flags_the_same_id_across_subfolders() {
        let dir = tempdir().expect("temp dir");
        let rules_dir = dir.path().join(rule::RULES_DIR);
        std::fs::create_dir_all(rules_dir.join("frontend")).expect("rules/frontend");
        std::fs::create_dir_all(rules_dir.join("backend")).expect("rules/backend");
        std::fs::write(rules_dir.join("frontend/shared.yaml"), "id: SHARED\n")
            .expect("frontend rule");
        std::fs::write(rules_dir.join("backend/shared.yaml"), "id: SHARED\n")
            .expect("backend rule");

        let specs = spec::load_all(&dir.path().join(spec::SPECS_DIR)).expect("specs");
        let rules = rule::load_all_list(&rules_dir).expect("rules");
        let mut findings = Vec::new();
        validate_no_duplicate_ids(&specs, &rules, &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].check, DUPLICATE_ID_CHECK);
        assert!(findings[0].message.contains("SHARED"));
    }

    #[test]
    fn no_duplicate_ids_flags_a_rule_id_colliding_with_a_spec_id() {
        let dir = tempdir().expect("temp dir");
        let specs_dir = dir.path().join(spec::SPECS_DIR);
        let rules_dir = dir.path().join(rule::RULES_DIR);
        std::fs::create_dir_all(&specs_dir).expect("specs dir");
        std::fs::create_dir_all(&rules_dir).expect("rules dir");
        std::fs::write(specs_dir.join("shared.yaml"), "id: SHARED\n").expect("spec");
        std::fs::write(rules_dir.join("shared.yaml"), "id: SHARED\n").expect("rule");

        let specs = spec::load_all(&specs_dir).expect("specs");
        let rules = rule::load_all_list(&rules_dir).expect("rules");
        let mut findings = Vec::new();
        validate_no_duplicate_ids(&specs, &rules, &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].check, DUPLICATE_ID_CHECK);
        assert!(findings[0].message.contains("SHARED"));
    }
}
