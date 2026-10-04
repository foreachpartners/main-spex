use std::path::Path;

use crate::config::Repository;
use crate::error::Result;
use crate::rule::Rule;
use crate::spec::Spec;

use super::Finding;

mod ac_order;
mod blocked_by;
mod coverage_ignore;
mod duplicate_ac_ids;
mod duplicate_ids;
mod property_order;
mod schema;
mod scope;
mod script_references;
mod status;

use ac_order::validate_ac_order;
use blocked_by::{status_by_id, validate_blocked_by};
use coverage_ignore::validate_coverage_ignore;
use duplicate_ac_ids::validate_no_duplicate_ac_ids;
use duplicate_ids::validate_no_duplicate_ids;
use property_order::validate_property_order;
use schema::validate_schema;
use scope::validate_scope;
use script_references::validate_script_references;
use status::{active_incomplete_finding, status_finding};

// Findings from this module are attributed to a synthetic spec id rather than
// a real rule, since spex's own conformance checks are built into `spex
// check` itself rather than declared in a rule (they must run even before
// any rules can be trusted to parse).
const RULE_ID: &str = "spex-internal-conformance";
const CONFORM_CHECK: &str = "spec-files-conform";

// Validates the spex repository's own spec files: `spex.yaml` and every file
// in `specs/` and `rules/` must satisfy its JSON Schema, each spec file
// must be named `<id>.yaml`, every script a rule check
// references must exist under `scripts/<spec-id>/`, every
// spec's acceptance criteria must be ordered by RFC 2119 commitment level,
// and every acceptance criterion id must be unique within its spec.
pub(super) fn validate_conformance(
    spex_dir: &Path,
    repositories: &[Repository],
    specs: &[Spec],
    rules: &[Rule],
) -> Result<Vec<Finding>> {
    let status_by_id = status_by_id(specs, rules);
    let mut findings = Vec::new();

    for spec in specs {
        findings.extend(status_finding(spec));
        findings.extend(active_incomplete_finding(spec));
    }

    for spec in specs {
        findings.extend(validate_blocked_by(spec, &status_by_id));
        findings.extend(validate_scope(spec, repositories));
        findings.extend(validate_coverage_ignore(spec));
    }

    findings.extend(validate_schema(spex_dir, specs, rules)?);
    findings.extend(validate_script_references(spex_dir, rules));
    findings.extend(validate_ac_order(specs, rules));
    findings.extend(validate_no_duplicate_ids(specs, rules));
    findings.extend(validate_no_duplicate_ac_ids(specs));
    findings.extend(validate_property_order(spex_dir)?);
    Ok(findings)
}

fn rel(spex_dir: &Path, file: &Path) -> String {
    file.strip_prefix(spex_dir)
        .unwrap_or(file)
        .to_string_lossy()
        .into_owned()
}

fn conform_finding(location: Option<String>, message: String) -> Finding {
    Finding::error(RULE_ID, CONFORM_CHECK, location, message)
}
