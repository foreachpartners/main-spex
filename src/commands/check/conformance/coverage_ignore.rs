use std::path::Path;

use crate::spec::{ContractLevel, CoverageLayer, Spec};

use super::{Finding, rel};

#[cfg(test)]
#[path = "coverage_ignore_test.rs"]
mod coverage_ignore_test;

// A `coverage_ignore` entry for the contract layer must match the spec's
// `contract` setting: with `none` there is no contract coverage to waive at
// all; with `single` the contract is spec-level, so a per-criterion entry
// can't match anything; only `for-each-ac` has per-criterion contracts to
// waive.
pub(super) fn validate_coverage_ignore(spex_dir: &Path, spec: &Spec) -> Vec<Finding> {
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
                    spec.id.as_str(),
                    "coverage-ignore",
                    Some(rel(spex_dir, &spec.path)),
                    message,
                )
            })
        })
        .collect()
}
