use std::path::Path;

use crate::rule::{CheckKind, RULES_DIR, Rule, SCRIPTS_DIR};

use super::super::runners::resolve_script;
use super::{Finding, conform_finding};

// Every script a rule check references must exist at
// `scripts/<spec-id-lowercase>/<script>`.
pub(super) fn validate_script_references(spex_dir: &Path, rules: &[Rule]) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut rules: Vec<&Rule> = rules.iter().collect();
    rules.sort_by(|a, b| a.id.cmp(&b.id));

    for rule in rules {
        for check in &rule.checks {
            let CheckKind::Script { script, .. } = &check.kind else {
                continue;
            };
            if resolve_script(script, spex_dir, rule.id.as_str()).is_none() {
                let dir = spex_dir.join(SCRIPTS_DIR).join(rule.id.as_str());
                findings.push(conform_finding(
                    Some(format!("{RULES_DIR}/{}", rule.path)),
                    format!("missing script {}/{script}", dir.display()),
                ));
            }
        }
    }
    findings
}
