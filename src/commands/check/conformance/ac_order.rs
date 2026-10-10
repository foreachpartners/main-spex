use std::path::Path;

use crate::rule::Rule;
use crate::spec::Spec;
use crate::support::commitment::Commitment;

use super::super::finding::{Finding, Location};
use super::conformance_error;

#[cfg(test)]
#[path = "ac_order_test.rs"]
mod ac_order_test;

const AC_ORDER_CHECK: &str = "ac-ordered";

pub(super) fn validate_ac_order(specs: &[Spec], rules: &[Rule]) -> Vec<Finding> {
    let mut findings = Vec::new();

    for spec in specs {
        let criteria = spec
            .acceptance_criteria
            .iter()
            .map(|ac| (ac.line, ac.statement.commitment()));
        findings.extend(check_order(&spec.path, criteria));
    }

    for rule in rules {
        let criteria = rule
            .acceptance_criteria
            .iter()
            .filter_map(|ac| Some((ac.line, ac.commitment()?)));
        findings.extend(check_order(&rule.path, criteria));
    }

    findings
}

fn check_order(file: &Path, criteria: impl Iterator<Item = (usize, Commitment)>) -> Vec<Finding> {
    let criteria: Vec<_> = criteria.collect();
    criteria
        .windows(2)
        .filter_map(|pair| {
            let ((_, previous), (line, commitment)) = (pair[0], pair[1]);
            (commitment < previous).then(|| {
                conformance_error(
                    AC_ORDER_CHECK,
                    Some(Location::at_line(file, line)),
                    format!(
                        "acceptance criteria out of order: {} must not follow {} (order: {})",
                        commitment.label(),
                        previous.label(),
                        Commitment::order()
                    ),
                )
            })
        })
        .collect()
}
