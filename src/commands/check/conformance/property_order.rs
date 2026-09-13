use std::path::Path;

use crate::rule::RULES_DIR;
use crate::spec::SPECS_DIR;
use crate::templates::SCHEMAS_DIR;
use crate::util::yaml_files;

use super::{Finding, SPEC_ID, Severity, rel};

#[cfg(test)]
#[path = "property_order_test.rs"]
mod property_order_test;

pub(super) const PROPERTY_ORDER_CHECK: &str = "property-order";

// The canonical property order for a schema (`spec.schema.json`,
// `rule.schema.json`, `spex.schema.json`) is simply its top-level
// `properties` key order.
fn schema_top_level_order(schema_path: &Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(schema_path) else {
        return Vec::new();
    };
    let Ok(schema) = serde_json::from_str::<serde_json::Value>(&text) else {
        return Vec::new();
    };
    schema
        .get("properties")
        .and_then(|properties| properties.as_object())
        .map(|properties| properties.keys().cloned().collect())
        .unwrap_or_default()
}

fn yaml_mapping_keys(value: &serde_yaml_ng::Value) -> Option<Vec<String>> {
    let mapping = value.as_mapping()?;
    Some(
        mapping
            .keys()
            .filter_map(|k| k.as_str().map(str::to_string))
            .collect(),
    )
}

// Finds the first key whose position in `actual` disagrees with `canonical`,
// ignoring keys absent from either side (an unknown key or one the schema
// doesn't declare is somebody else's problem, e.g. `spec-files-conform`).
fn find_order_violation(canonical: &[String], actual: &[String]) -> Option<(String, String)> {
    let filtered_actual: Vec<&String> = actual.iter().filter(|k| canonical.contains(k)).collect();
    let filtered_canonical: Vec<&String> =
        canonical.iter().filter(|k| actual.contains(k)).collect();
    filtered_actual
        .iter()
        .zip(filtered_canonical.iter())
        .find(|(found, expected)| found != expected)
        .map(|(found, expected)| ((*found).clone(), (*expected).clone()))
}

fn property_order_message(found: &str, expected: &str, canonical: &[String]) -> String {
    format!(
        "property `{found}` is out of order: expected `{expected}` at this position (canonical order: {})",
        canonical.join(", ")
    )
}

// Verifies every spec file, rule file, and `spex.yaml` declares its
// top-level properties in the order their JSON Schema declares them.
pub(super) fn validate_property_order(spex_dir: &Path, findings: &mut Vec<Finding>) {
    let schemas_dir = spex_dir.join(SCHEMAS_DIR);

    let spec_order = schema_top_level_order(&schemas_dir.join("spec.schema.json"));
    for file in yaml_files(&spex_dir.join(SPECS_DIR)) {
        check_top_level_order(spex_dir, &file, &spec_order, findings);
    }

    let rule_order = schema_top_level_order(&schemas_dir.join("rule.schema.json"));
    for file in yaml_files(&spex_dir.join(RULES_DIR)) {
        check_top_level_order(spex_dir, &file, &rule_order, findings);
    }

    let spex_yaml = spex_dir.join(crate::config::SPEX_CONFIG_FILE);
    if spex_yaml.is_file() {
        let spex_order = schema_top_level_order(&schemas_dir.join("spex.schema.json"));
        check_top_level_order(spex_dir, &spex_yaml, &spex_order, findings);
    }
}

fn check_top_level_order(
    spex_dir: &Path,
    file: &Path,
    canonical: &[String],
    findings: &mut Vec<Finding>,
) {
    if canonical.is_empty() {
        return;
    }
    let Ok(text) = std::fs::read_to_string(file) else {
        return;
    };
    let Ok(value) = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&text) else {
        return;
    };
    let Some(actual) = yaml_mapping_keys(&value) else {
        return;
    };

    if let Some((found, expected)) = find_order_violation(canonical, &actual) {
        findings.push(Finding {
            severity: Severity::Error,
            spec_id: SPEC_ID.to_string(),
            check: PROPERTY_ORDER_CHECK.to_string(),
            location: Some(rel(spex_dir, file)),
            message: property_order_message(&found, &expected, canonical),
        });
    }
}
