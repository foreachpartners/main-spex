use std::collections::HashMap;
use std::path::Path;

use crate::rule::Rule;
use crate::spec::Spec;

use crate::support::fs::relative_to;

use super::super::finding::Location;
use super::{Finding, conformance_error};

#[cfg(test)]
#[path = "duplicate_ids_test.rs"]
mod duplicate_ids_test;

pub(super) const DUPLICATE_ID_CHECK: &str = "no-duplicate-ids";

// Every id must be unique across both `rules/`
// and `specs/`, since rule ids and spec ids share the same naming
// convention and could otherwise collide.
pub(super) fn validate_no_duplicate_ids(
    spex_dir: &Path,
    specs: &[Spec],
    rules: &[Rule],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut seen: HashMap<&str, &Path> = HashMap::new();
    let definitions = specs
        .iter()
        .map(|spec| (spec.id.as_str(), spec.path.as_path()))
        .chain(
            rules
                .iter()
                .map(|rule| (rule.id.as_str(), rule.path.as_path())),
        );

    for (id, path) in definitions {
        if let Some(first_seen) = seen.get(id) {
            findings.push(conformance_error(
                DUPLICATE_ID_CHECK,
                Some(Location::in_file(path)),
                format!(
                    "duplicate id `{id}`: already defined in `{}`",
                    relative_to(first_seen, spex_dir).display()
                ),
            ));
        } else {
            seen.insert(id, path);
        }
    }
    findings
}
