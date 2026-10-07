use std::collections::HashSet;

use crate::spec::{AcId, Spec};

use super::{Finding, RULE_ID};

const DUPLICATE_AC_ID_CHECK: &str = "no-duplicate-ac-ids";

// An acceptance criterion id must be unique within its own spec: annotations
// reference `<spec-id>#<ac-id>`, so a duplicate id within one spec would make
// that reference ambiguous.
pub(super) fn validate_no_duplicate_ac_ids(specs: &[Spec]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for spec in specs {
        let mut seen: HashSet<&AcId> = HashSet::new();
        for ac in &spec.acceptance_criteria {
            if !seen.insert(&ac.id) {
                findings.push(Finding::error(
                    RULE_ID,
                    DUPLICATE_AC_ID_CHECK,
                    Some(format!("{}:{}", spec.location(), ac.line)),
                    format!(
                        "duplicate acceptance criterion id `{}` within this spec",
                        ac.id
                    ),
                ));
            }
        }
    }
    findings
}
