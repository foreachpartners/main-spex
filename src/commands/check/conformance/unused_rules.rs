use crate::config::Package;
use crate::rule::Rule;

use super::super::finding::{Finding, Location};
use super::conformance_warn;

#[cfg(test)]
#[path = "unused_rules_test.rs"]
mod unused_rules_test;

const UNUSED_RULE_CHECK: &str = "unused-rule";

pub(super) fn validate_unused_rules(packages: &[Package], rules: &[Rule]) -> Vec<Finding> {
    rules
        .iter()
        .filter(|rule| {
            !packages
                .iter()
                .any(|package| package.rules.contains(&rule.id))
        })
        .map(|rule| {
            conformance_warn(
                UNUSED_RULE_CHECK,
                Some(Location::in_file(&rule.path)),
                format!(
                    "no package lists rule `{}` under `rules` — delete its file and script folder, and add its id to `rules_unused` in spex.yaml",
                    rule.id
                ),
            )
        })
        .collect()
}
