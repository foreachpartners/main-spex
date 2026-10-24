use std::path::Path;

use crate::config::Package;
use crate::error::Result;
use crate::rule::Rule;
use crate::spec::Spec;

use super::super::finding::{Finding, Location, Source};
use super::ac_order::validate_ac_order;
use super::blocked_by::{status_by_id, validate_blocked_by};
use super::coverage_ignore::validate_coverage_ignore;
use super::declared_files::validate_declared_files;
use super::duplicate_ac_ids::validate_no_duplicate_ac_ids;
use super::duplicate_ids::validate_no_duplicate_ids;
use super::packages::validate_packages;
use super::property_order::validate_property_order;
use super::schema::validate_schema;
use super::script_references::validate_script_references;
use super::status::{active_incomplete_finding, status_finding};
use super::unused_rules::validate_unused_rules;

#[cfg(test)]
#[path = "conformance_test.rs"]
mod conformance_test;

pub(super) const RULE_ID: &str = "spex-internal-conformance";
const CONFORM_CHECK: &str = "spec-files-conform";

pub(in crate::commands::check) fn validate_conformance(
    spex_dir: &Path,
    prd: &Path,
    packages: &[Package],
    specs: &[Spec],
    rules: &[Rule],
) -> Result<Vec<Finding>> {
    let status_by_id = status_by_id(specs);
    let mut findings = Vec::new();

    for spec in specs {
        findings.extend(status_finding(spec));
        findings.extend(active_incomplete_finding(spec));
    }

    for spec in specs {
        findings.extend(validate_blocked_by(spec, &status_by_id));
        findings.extend(validate_packages(spec, packages));
        findings.extend(validate_coverage_ignore(spec));
    }

    findings.extend(validate_schema(spex_dir, packages, specs, rules)?);
    findings.extend(validate_declared_files(spex_dir, prd, packages));
    findings.extend(validate_script_references(rules));
    findings.extend(validate_unused_rules(packages, rules));
    findings.extend(validate_ac_order(specs, rules));
    findings.extend(validate_no_duplicate_ids(spex_dir, specs, rules));
    findings.extend(validate_no_duplicate_ac_ids(specs));
    findings.extend(validate_property_order(spex_dir)?);
    Ok(findings
        .into_iter()
        .map(|finding| finding.relative_to(spex_dir))
        .collect())
}

pub(super) fn conformance_error(
    check: &str,
    location: Option<Location>,
    message: String,
) -> Finding {
    Finding::error(Source::internal(RULE_ID, check), location, message)
}

pub(super) fn conformance_warn(
    check: &str,
    location: Option<Location>,
    message: String,
) -> Finding {
    Finding::warn(Source::internal(RULE_ID, check), location, message)
}

pub(super) fn conform_finding(location: Option<Location>, message: String) -> Finding {
    conformance_error(CONFORM_CHECK, location, message)
}
