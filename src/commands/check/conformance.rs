use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::commands::check_schema;
use crate::rule::{CheckKind, RULES_DIR, Rule, SCRIPTS_DIR};
use crate::spec::SPECS_DIR;
use crate::templates::SCHEMAS_DIR;
use crate::util::yaml_files;

use super::runners::resolve_script;
use super::{Finding, Severity};

#[cfg(test)]
#[path = "conformance_test.rs"]
mod conformance_test;

// Findings from this module are attributed to a synthetic spec id rather than
// a real rule, since spex's own conformance checks are built into `spex
// check` itself rather than declared in a rule (they must run even before
// any rules can be trusted to parse).
const SPEC_ID: &str = "spex-internal-conformance";
const CONFORM_CHECK: &str = "spec-files-conform";
const AC_ORDER_CHECK: &str = "ac-ordered";
const DUPLICATE_ID_CHECK: &str = "no-duplicate-ids";
const PROPERTY_ORDER_CHECK: &str = "property-order";

#[derive(Deserialize)]
struct IdOnly {
    id: String,
}

// Validates the spex repository's own spec files: `spex.yaml` and every file
// in `specs/` and `rules/` must satisfy its JSON Schema, each spec file
// must be named `<id>.yaml`, every script a rule check
// references must exist under `scripts/<spec-id>/`, and every
// spec's acceptance criteria must be ordered by RFC 2119 commitment level.
pub(super) fn validate_conformance(
    spex_dir: &Path,
    rules: &std::collections::HashMap<String, Rule>,
    findings: &mut Vec<Finding>,
) {
    validate_schema(spex_dir, findings);
    validate_script_references(spex_dir, rules, findings);
    validate_ac_order(spex_dir, findings);
    validate_no_duplicate_ids(spex_dir, findings);
    validate_property_order(spex_dir, findings);
}

fn rel(spex_dir: &Path, file: &Path) -> String {
    file.strip_prefix(spex_dir)
        .unwrap_or(file)
        .to_string_lossy()
        .into_owned()
}

fn conform_finding(location: Option<String>, message: String) -> Finding {
    Finding {
        severity: Severity::Error,
        spec_id: SPEC_ID.to_string(),
        check: CONFORM_CHECK.to_string(),
        location,
        message,
    }
}

fn validate_schema(spex_dir: &Path, findings: &mut Vec<Finding>) {
    let spex_yaml = spex_dir.join(crate::config::SPEX_CONFIG_FILE);
    if spex_yaml.is_file() {
        validate_schema_group(spex_dir, &[spex_yaml], "spex.schema.json", findings);
    } else {
        findings.push(conform_finding(
            None,
            format!("{} not found", crate::config::SPEX_CONFIG_FILE),
        ));
    }

    validate_schema_group(
        spex_dir,
        &yaml_files(&spex_dir.join(SPECS_DIR)),
        "spec.schema.json",
        findings,
    );
    validate_schema_group(
        spex_dir,
        &yaml_files(&spex_dir.join(RULES_DIR)),
        "rule.schema.json",
        findings,
    );

    for file in yaml_files(&spex_dir.join(SPECS_DIR)) {
        check_filename_matches_id(spex_dir, &file, findings);
    }
    for file in yaml_files(&spex_dir.join(RULES_DIR)) {
        check_filename_matches_id(spex_dir, &file, findings);
    }
}

fn validate_schema_group(
    spex_dir: &Path,
    files: &[PathBuf],
    schema_name: &str,
    findings: &mut Vec<Finding>,
) {
    if files.is_empty() {
        return;
    }
    let schema_path = spex_dir.join(SCHEMAS_DIR).join(schema_name);
    let validator = match check_schema::build_validator(&schema_path) {
        Ok(validator) => validator,
        Err(err) => {
            findings.push(conform_finding(
                None,
                format!("could not load schema `{}`: {err:?}", schema_path.display()),
            ));
            return;
        }
    };

    for file in files {
        match check_schema::validate_file(&validator, file) {
            Ok(messages) => {
                for message in messages {
                    findings.push(conform_finding(Some(rel(spex_dir, file)), message));
                }
            }
            Err(err) => {
                findings.push(conform_finding(
                    Some(rel(spex_dir, file)),
                    format!("{err:?}"),
                ));
            }
        }
    }
}

// A spec file must be named after its own id (naming-convention: id and
// filename are identical).
fn check_filename_matches_id(spex_dir: &Path, file: &Path, findings: &mut Vec<Finding>) {
    let Ok(text) = std::fs::read_to_string(file) else {
        return;
    };
    let Ok(parsed) = serde_yaml_ng::from_str::<IdOnly>(&text) else {
        return;
    };
    let stem = file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    if parsed.id != stem {
        findings.push(conform_finding(
            Some(rel(spex_dir, file)),
            format!("filename stem `{stem}` does not match id `{}`", parsed.id),
        ));
    }
}

// Every script a rule check references must exist at
// `scripts/<spec-id-lowercase>/<script>`.
fn validate_script_references(
    spex_dir: &Path,
    rules: &std::collections::HashMap<String, Rule>,
    findings: &mut Vec<Finding>,
) {
    let mut rules: Vec<&Rule> = rules.values().collect();
    rules.sort_by(|a, b| a.id.cmp(&b.id));

    for rule in rules {
        for check in &rule.checks {
            let CheckKind::Script { script, .. } = &check.kind else {
                continue;
            };
            if resolve_script(script, spex_dir, &rule.id).is_none() {
                let dir = spex_dir.join(SCRIPTS_DIR).join(&rule.id);
                findings.push(conform_finding(
                    Some(format!("{RULES_DIR}/{}", rule.path)),
                    format!("missing script {}/{script}", dir.display()),
                ));
            }
        }
    }
}

// Every id must be unique across both `rules/`
// and `specs/`, since rule ids and spec ids share the same naming
// convention and could otherwise collide.
fn validate_no_duplicate_ids(spex_dir: &Path, findings: &mut Vec<Finding>) {
    let mut seen: HashMap<String, PathBuf> = HashMap::new();
    for dir_name in [SPECS_DIR, RULES_DIR] {
        for file in yaml_files(&spex_dir.join(dir_name)) {
            let Ok(text) = std::fs::read_to_string(&file) else {
                continue;
            };
            let Ok(parsed) = serde_yaml_ng::from_str::<IdOnly>(&text) else {
                continue;
            };

            if let Some(first_seen) = seen.get(&parsed.id) {
                findings.push(Finding {
                    severity: Severity::Error,
                    spec_id: SPEC_ID.to_string(),
                    check: DUPLICATE_ID_CHECK.to_string(),
                    location: Some(rel(spex_dir, &file)),
                    message: format!(
                        "duplicate id `{}`: already defined in `{}`",
                        parsed.id,
                        rel(spex_dir, first_seen)
                    ),
                });
            } else {
                seen.insert(parsed.id, file);
            }
        }
    }
}

fn rfc2119_rank(text: &str) -> Option<(u8, &'static str)> {
    if text.starts_with("MUST NOT") {
        Some((2, "MUST NOT"))
    } else if text.starts_with("MUST") {
        Some((1, "MUST"))
    } else if text.starts_with("SHOULD NOT") {
        Some((4, "SHOULD NOT"))
    } else if text.starts_with("SHOULD") {
        Some((3, "SHOULD"))
    } else if text.starts_with("MAY") {
        Some((5, "MAY"))
    } else {
        None
    }
}

// Verifies each spec file lists its `acceptance_criteria` in RFC 2119
// commitment order: MUST, MUST NOT, SHOULD, SHOULD NOT, MAY.
fn validate_ac_order(spex_dir: &Path, findings: &mut Vec<Finding>) {
    for dir_name in [SPECS_DIR, RULES_DIR] {
        for file in yaml_files(&spex_dir.join(dir_name)) {
            check_ac_order(spex_dir, &file, findings);
        }
    }
}

fn check_ac_order(spex_dir: &Path, file: &Path, findings: &mut Vec<Finding>) {
    let Ok(text) = std::fs::read_to_string(file) else {
        return;
    };

    let mut in_ac = false;
    let mut prev_rank: u8 = 0;
    let mut prev_label = "";

    for (index, line) in text.lines().enumerate() {
        if line.starts_with("acceptance_criteria:") {
            in_ac = true;
            prev_rank = 0;
            prev_label = "";
            continue;
        }
        if !in_ac {
            continue;
        }
        if !line.starts_with(' ') && !line.starts_with('\t') {
            in_ac = false;
            continue;
        }

        let Some(mut item) = line.trim_start().strip_prefix("- ") else {
            continue;
        };
        item = item.trim();
        if item.len() >= 2 && item.starts_with('"') && item.ends_with('"') {
            item = &item[1..item.len() - 1];
        }
        let item = item
            .strip_prefix("[ ]")
            .or_else(|| item.strip_prefix("[x]"))
            .unwrap_or(item)
            .trim_start();

        let Some((rank, label)) = rfc2119_rank(item) else {
            continue;
        };
        if prev_rank != 0 && rank < prev_rank {
            findings.push(Finding {
                severity: Severity::Error,
                spec_id: SPEC_ID.to_string(),
                check: AC_ORDER_CHECK.to_string(),
                location: Some(format!("{}:{}", rel(spex_dir, file), index + 1)),
                message: format!(
                    "acceptance criteria out of order: {label} must not follow {prev_label} (order: MUST, MUST NOT, SHOULD, SHOULD NOT, MAY)"
                ),
            });
        }
        prev_rank = rank;
        prev_label = label;
    }
}

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
fn validate_property_order(spex_dir: &Path, findings: &mut Vec<Finding>) {
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
