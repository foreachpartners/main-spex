use std::collections::HashMap;

use crate::rule::Rule;
use crate::spec::Spec;

use super::{Finding, RULE_ID};

#[cfg(test)]
#[path = "duplicate_ids_test.rs"]
mod duplicate_ids_test;

pub(super) const DUPLICATE_ID_CHECK: &str = "no-duplicate-ids";

// Every id must be unique across both `rules/`
// and `specs/`, since rule ids and spec ids share the same naming
// convention and could otherwise collide.
pub(super) fn validate_no_duplicate_ids(
    specs: &[Spec],
    rules: &[Rule],
    findings: &mut Vec<Finding>,
) {
    let mut seen: HashMap<&str, String> = HashMap::new();
    let locations: Vec<(&str, String)> = specs
        .iter()
        .map(|spec| (spec.id.as_str(), spec.location()))
        .chain(rules.iter().map(|rule| (rule.id.as_str(), rule.location())))
        .collect();

    for (id, location) in &locations {
        if let Some(first_seen) = seen.get(id) {
            findings.push(Finding::error(
                RULE_ID,
                DUPLICATE_ID_CHECK,
                Some(location.clone()),
                format!("duplicate id `{id}`: already defined in `{first_seen}`"),
            ));
        } else {
            seen.insert(id, location.clone());
        }
    }
}
