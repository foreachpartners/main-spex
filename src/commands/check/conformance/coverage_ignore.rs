use crate::spec::{ContractLevel, CoverageLayer, Spec};

use super::super::finding::{Finding, Location, Source};

#[cfg(test)]
#[path = "coverage_ignore_test.rs"]
mod coverage_ignore_test;

pub(super) fn validate_coverage_ignore(spec: &Spec) -> Vec<Finding> {
    spec.coverage_ignore
        .iter()
        .filter(|ignore| ignore.layer == CoverageLayer::Contract)
        .filter_map(|ignore| {
            let message = match (spec.contract, ignore.ac.as_ref()) {
                (ContractLevel::None, _) => Some(
                    "`coverage_ignore` waives the contract layer but `contract: none` already disables contract coverage — remove the entry or set `contract` to `single` or `for-each-ac`".to_string(),
                ),
                (ContractLevel::Single, Some(_)) => Some(
                    "`coverage_ignore` waives one acceptance criterion's contract but `contract: single` requires a single spec-level contract — use `contract: for-each-ac` for per-criterion contracts".to_string(),
                ),
                _ => None,
            };
            message.map(|message| {
                Finding::error(
                    Source::spec(spec.id.as_str(), "coverage-ignore"),
                    Some(Location::in_file(&spec.path)),
                    message,
                )
            })
        })
        .collect()
}
