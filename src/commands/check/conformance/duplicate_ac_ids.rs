use std::collections::HashSet;

use crate::spec::{AcId, Spec};

use super::super::finding::{Finding, Location};
use super::conformance_error;

const DUPLICATE_AC_ID_CHECK: &str = "no-duplicate-ac-ids";

pub(super) fn validate_no_duplicate_ac_ids(specs: &[Spec]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for spec in specs {
        let mut seen: HashSet<&AcId> = HashSet::new();
        for ac in &spec.acceptance_criteria {
            if !seen.insert(&ac.id) {
                findings.push(conformance_error(
                    DUPLICATE_AC_ID_CHECK,
                    Some(Location::at_line(&spec.path, ac.line)),
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
