#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::super::*;
    use crate::rule::{Check, CheckKind, RunScope, Severity};
    use crate::support::testing;
    use tempfile::tempdir;

    fn script_rule(rule_path: PathBuf) -> Rule {
        Rule {
            path: rule_path,
            checks: vec![Check {
                name: "lint".to_string(),
                description: None,
                kind: CheckKind::Script {
                    script: "lint.sh".to_string(),
                    run: RunScope::PerRepo,
                },
                include: Vec::new(),
                exclude: Vec::new(),
                severity: Severity::Error,
            }],
            ..testing::rule("x")
        }
    }

    #[test]
    fn script_beside_the_rule_file_is_found() {
        let spex = tempdir().expect("spex dir");
        let rule_dir = spex.path().join("rules/x");
        std::fs::create_dir_all(&rule_dir).expect("rule script dir");
        std::fs::write(rule_dir.join("lint.sh"), "").expect("script");
        let rule = script_rule(spex.path().join("rules/x.yaml"));

        assert!(validate_script_references(spex.path(), &[rule]).is_empty());
    }

    #[test]
    fn script_beside_a_nested_rule_file_is_found() {
        let spex = tempdir().expect("spex dir");
        let rule_dir = spex.path().join("rules/backend/x");
        std::fs::create_dir_all(&rule_dir).expect("rule script dir");
        std::fs::write(rule_dir.join("lint.sh"), "").expect("script");
        let rule = script_rule(spex.path().join("rules/backend/x.yaml"));

        assert!(validate_script_references(spex.path(), &[rule]).is_empty());
    }

    #[test]
    fn missing_script_is_reported_with_the_rule_script_folder() {
        let spex = tempdir().expect("spex dir");
        let rule = script_rule(spex.path().join("rules/x.yaml"));

        let findings = validate_script_references(spex.path(), &[rule]);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].location.as_deref(), Some("rules/x.yaml"));
        assert_eq!(
            findings[0].message,
            format!(
                "missing script {}/lint.sh",
                spex.path().join("rules/x").display()
            )
        );
    }
}
