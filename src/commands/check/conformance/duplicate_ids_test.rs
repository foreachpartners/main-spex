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
        std::fs::write(rules_dir.join("frontend/one.yaml"), "id: one\n").expect("one");
        std::fs::write(rules_dir.join("two.yaml"), "id: two\n").expect("two");

        let specs = Spec::load_all(&dir.path().join(spec::SPECS_DIR)).expect("specs");
        let rules = Rule::load_all(&rules_dir).expect("rules");
        let findings = validate_no_duplicate_ids(dir.path(), &specs, &rules);

        assert!(findings.is_empty());
    }

    #[test]
    fn no_duplicate_ids_flags_the_same_id_across_subfolders() {
        let dir = tempdir().expect("temp dir");
        let rules_dir = dir.path().join(rule::RULES_DIR);
        std::fs::create_dir_all(rules_dir.join("frontend")).expect("rules/frontend");
        std::fs::create_dir_all(rules_dir.join("backend")).expect("rules/backend");
        std::fs::write(rules_dir.join("frontend/shared.yaml"), "id: shared\n")
            .expect("frontend rule");
        std::fs::write(rules_dir.join("backend/shared.yaml"), "id: shared\n")
            .expect("backend rule");

        let specs = Spec::load_all(&dir.path().join(spec::SPECS_DIR)).expect("specs");
        let rules = Rule::load_all(&rules_dir).expect("rules");
        let findings = validate_no_duplicate_ids(dir.path(), &specs, &rules);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].check, DUPLICATE_ID_CHECK);
        assert!(findings[0].message.contains("shared"));
    }

    #[test]
    fn no_duplicate_ids_flags_a_rule_id_colliding_with_a_spec_id() {
        let dir = tempdir().expect("temp dir");
        let specs_dir = dir.path().join(spec::SPECS_DIR);
        let rules_dir = dir.path().join(rule::RULES_DIR);
        std::fs::create_dir_all(&specs_dir).expect("specs dir");
        std::fs::create_dir_all(&rules_dir).expect("rules dir");
        std::fs::write(
            specs_dir.join("shared.yaml"),
            "id: shared\ncontract: single\n",
        )
        .expect("spec");
        std::fs::write(rules_dir.join("shared.yaml"), "id: shared\n").expect("rule");

        let specs = Spec::load_all(&specs_dir).expect("specs");
        let rules = Rule::load_all(&rules_dir).expect("rules");
        let findings = validate_no_duplicate_ids(dir.path(), &specs, &rules);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].check, DUPLICATE_ID_CHECK);
        assert!(findings[0].message.contains("shared"));
    }
}
