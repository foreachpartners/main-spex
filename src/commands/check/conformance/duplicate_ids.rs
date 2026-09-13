use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::rule::RULES_DIR;
use crate::spec::SPECS_DIR;
use crate::util::yaml_files;

use super::{Finding, IdOnly, SPEC_ID, Severity, rel};

#[cfg(test)]
#[path = "duplicate_ids_test.rs"]
mod duplicate_ids_test;

pub(super) const DUPLICATE_ID_CHECK: &str = "no-duplicate-ids";

// Every id must be unique across both `rules/`
// and `specs/`, since rule ids and spec ids share the same naming
// convention and could otherwise collide.
pub(super) fn validate_no_duplicate_ids(spex_dir: &Path, findings: &mut Vec<Finding>) {
    let mut seen: HashMap<String, PathBuf> = HashMap::new();
    for dir_name in [SPECS_DIR, RULES_DIR] {
        for file in yaml_files(&spex_dir.join(dir_name)) {
            let Ok(text) = std::fs::read_to_string(&file) else {
                continue;
            };
            let Ok(parsed) = serde_yaml_ng::from_str::<IdOnly>(&text) else {
                continue;
            };

            if let Some(first_seen) = seen.get(&parsed.id) {
                findings.push(Finding {
                    severity: Severity::Error,
                    spec_id: SPEC_ID.to_string(),
                    check: DUPLICATE_ID_CHECK.to_string(),
                    location: Some(rel(spex_dir, &file)),
                    message: format!(
                        "duplicate id `{}`: already defined in `{}`",
                        parsed.id,
                        rel(spex_dir, first_seen)
                    ),
                });
            } else {
                seen.insert(parsed.id, file);
            }
        }
    }
}
