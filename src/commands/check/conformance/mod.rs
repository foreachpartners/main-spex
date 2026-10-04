use std::path::Path;

use crate::error::Result;
use crate::spec::Spec;

use super::Finding;

mod ac_order;
mod duplicate_ac_ids;
mod duplicate_ids;
mod property_order;
mod schema;
mod script_references;

use ac_order::validate_ac_order;
use duplicate_ac_ids::validate_no_duplicate_ac_ids;
use duplicate_ids::validate_no_duplicate_ids;
use property_order::validate_property_order;
use schema::validate_schema;
use script_references::validate_script_references;

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
    specs: &[Spec],
    rules: &[crate::rule::Rule],
    findings: &mut Vec<Finding>,
) -> Result<()> {
    validate_schema(spex_dir, specs, rules, findings)?;
    validate_script_references(spex_dir, rules, findings);
    validate_ac_order(specs, rules, findings);
    validate_no_duplicate_ids(specs, rules, findings);
    validate_no_duplicate_ac_ids(specs, findings);
    validate_property_order(spex_dir, findings)
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
