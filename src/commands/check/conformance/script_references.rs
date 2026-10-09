use std::path::Path;

use crate::rule::{CheckKind, Rule};

use super::super::runners::resolve_script;
use super::{Finding, conform_finding, rel};

#[cfg(test)]
#[path = "script_references_test.rs"]
mod script_references_test;

// Every script a rule check references must exist in the rule's script folder,
// beside the rule file.
pub(super) fn validate_script_references(spex_dir: &Path, rules: &[Rule]) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut rules: Vec<&Rule> = rules.iter().collect();
    rules.sort_by(|a, b| a.id.cmp(&b.id));

    for rule in rules {
        for check in &rule.checks {
            let CheckKind::Script { script, .. } = &check.kind else {
                continue;
            };
            let script_dir = rule.script_dir();
            if resolve_script(&script_dir, script).is_none() {
                findings.push(conform_finding(
                    Some(rel(spex_dir, &rule.path)),
                    format!("missing script {}/{script}", script_dir.display()),
                ));
            }
        }
    }
    findings
}
