use crate::rule::Rule;
use crate::spec::Spec;

use super::{Finding, RULE_ID};

const AC_ORDER_CHECK: &str = "ac-ordered";

// Verifies each spec and rule lists its `acceptance_criteria` in RFC 2119
// commitment order: MUST, MUST NOT, SHOULD, SHOULD NOT, MAY. A spec's
// criteria carry the commitment level as a key (`must`, `must_not`, ...); a
// rule's are still plain, MUST/MUST NOT-prefixed strings.
pub(super) fn validate_ac_order(specs: &[Spec], rules: &[Rule], findings: &mut Vec<Finding>) {
    for spec in specs {
        let mut prev: Option<(u8, &'static str)> = None;
        for ac in &spec.acceptance_criteria {
            let rank = ac.commitment.rank();
            let label = ac.commitment.label();
            check_order(prev, (rank, label), &spec.location(), ac.line, findings);
            prev = Some((rank, label));
        }
    }

    for rule in rules {
        let mut prev: Option<(u8, &'static str)> = None;
        for ac in &rule.acceptance_criteria {
            let Some((rank, label)) = ac.rank() else {
                continue;
            };
            check_order(prev, (rank, label), &rule.location(), ac.line, findings);
            prev = Some((rank, label));
        }
    }
}

fn check_order(
    prev: Option<(u8, &'static str)>,
    current: (u8, &'static str),
    location: &str,
    line: usize,
    findings: &mut Vec<Finding>,
) {
    let (rank, label) = current;
    let Some((prev_rank, prev_label)) = prev else {
        return;
    };
    if rank < prev_rank {
        findings.push(Finding::error(
            RULE_ID,
            AC_ORDER_CHECK,
            Some(format!("{location}:{line}")),
            format!(
                "acceptance criteria out of order: {label} must not follow {prev_label} (order: MUST, MUST NOT, SHOULD, SHOULD NOT, MAY)"
            ),
        ));
    }
}
