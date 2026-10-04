use crate::commitment::Commitment;
use crate::rule::Rule;
use crate::spec::Spec;

use super::{Finding, RULE_ID};

#[cfg(test)]
#[path = "ac_order_test.rs"]
mod ac_order_test;

const AC_ORDER_CHECK: &str = "ac-ordered";

// Verifies each spec and rule lists its `acceptance_criteria` in RFC 2119
// commitment order: MUST, MUST NOT, SHOULD, SHOULD NOT, MAY. A spec's
// criteria carry the commitment as a key (`must`, `must_not`, ...); a rule's
// are plain strings starting with it, and those without one are skipped.
pub(super) fn validate_ac_order(specs: &[Spec], rules: &[Rule], findings: &mut Vec<Finding>) {
    for spec in specs {
        let criteria = spec
            .acceptance_criteria
            .iter()
            .map(|ac| (ac.line, ac.statement.commitment()));
        check_order(&spec.location(), criteria, findings);
    }

    for rule in rules {
        let criteria = rule
            .acceptance_criteria
            .iter()
            .filter_map(|ac| Some((ac.line, ac.commitment()?)));
        check_order(&rule.location(), criteria, findings);
    }
}

fn check_order(
    location: &str,
    criteria: impl Iterator<Item = (usize, Commitment)>,
    findings: &mut Vec<Finding>,
) {
    let criteria: Vec<_> = criteria.collect();
    for pair in criteria.windows(2) {
        let ((_, previous), (line, commitment)) = (pair[0], pair[1]);
        if commitment < previous {
            findings.push(Finding::error(
                RULE_ID,
                AC_ORDER_CHECK,
                Some(format!("{location}:{line}")),
                format!(
                    "acceptance criteria out of order: {} must not follow {} (order: {})",
                    commitment.label(),
                    previous.label(),
                    Commitment::order()
                ),
            ));
        }
    }
}
