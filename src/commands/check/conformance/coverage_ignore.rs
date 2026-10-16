use crate::spec::{ContractLevel, CoverageLayer, Spec};

use super::super::finding::{Finding, Location, Source};

#[cfg(test)]
#[path = "coverage_ignore_test.rs"]
mod coverage_ignore_test;

pub(super) fn validate_coverage_ignore(spec: &Spec) -> Vec<Finding> {
    let mut findings = validate_contract_waivers(spec);
    findings.extend(validate_waiving_packages_are_listed(spec));
    findings
}

fn validate_contract_waivers(spec: &Spec) -> Vec<Finding> {
    spec.coverage_ignore
        .iter()
        .chain(spec.package_coverage_ignore.values().flatten())
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
            message.map(|message| coverage_ignore_error(spec, message))
        })
        .collect()
}

fn validate_waiving_packages_are_listed(spec: &Spec) -> Vec<Finding> {
    spec.package_coverage_ignore
        .keys()
        .filter(|short_name| !spec.packages.contains(short_name))
        .map(|short_name| {
            coverage_ignore_error(
                spec,
                format!(
                    "`coverage_ignore` of package `{short_name}` names this spec, but its `packages` does not list `{short_name}` — list the package in `packages` or remove the entry"
                ),
            )
        })
        .collect()
}

fn coverage_ignore_error(spec: &Spec, message: String) -> Finding {
    Finding::error(
        Source::spec(spec.id.as_str(), "coverage-ignore"),
        Some(Location::in_file(&spec.path)),
        message,
    )
}
