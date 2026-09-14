use std::collections::HashSet;

use crate::spec::Spec;

use super::{Finding, SPEC_ID, Severity};

const DUPLICATE_AC_ID_CHECK: &str = "no-duplicate-ac-ids";

// An acceptance criterion id must be unique within its own spec: annotations
// reference `<spec-id>#<ac-id>`, so a duplicate id within one spec would make
// that reference ambiguous.
pub(super) fn validate_no_duplicate_ac_ids(specs: &[Spec], findings: &mut Vec<Finding>) {
    for spec in specs {
        let mut seen: HashSet<&str> = HashSet::new();
        for ac in &spec.acceptance_criteria {
            if !seen.insert(&ac.id) {
                findings.push(Finding {
                    severity: Severity::Error,
                    spec_id: SPEC_ID.to_string(),
                    check: DUPLICATE_AC_ID_CHECK.to_string(),
                    location: Some(format!("{}:{}", spec.location(), ac.line)),
                    message: format!(
                        "duplicate acceptance criterion id `{}` within this spec",
                        ac.id
                    ),
                });
            }
        }
    }
}
