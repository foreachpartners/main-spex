use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::commands::check_schema;
use crate::functional_spec::FUNCTIONAL_SPECS_DIR;
use crate::rule_spec::{RULE_SPECS_DIR, RuleSpec, SCRIPTS_DIR};
use crate::templates::SCHEMAS_DIR;

use super::runners::resolve_script;
use super::{Finding, Severity};

// Findings from this module are attributed to a synthetic spec id rather than
// a real rule spec, since spex's own conformance checks are built into `spex
// check` itself rather than declared in a rs-spec (they must run even before
// any rs-specs can be trusted to parse).
const SPEC_ID: &str = "RS-SPEX-CONFORMANCE";
const CONFORM_CHECK: &str = "spec-files-conform";
const AC_ORDER_CHECK: &str = "ac-ordered";

#[derive(Deserialize)]
struct IdOnly {
    id: String,
}

// Validates the spex repository's own spec files: `spex.yaml` and every file
// in `fs-specs/` and `rs-specs/` must satisfy its JSON Schema, each spec file
// must be named `<id-lowercase>.yaml`, every script a rs-spec check
// references must exist under `scripts/<spec-id-lowercase>/`, and every
// spec's acceptance criteria must be ordered by RFC 2119 commitment level.
pub(super) fn validate_conformance(
    spex_dir: &Path,
    rs_specs: &std::collections::HashMap<String, RuleSpec>,
    findings: &mut Vec<Finding>,
) {
    validate_schema(spex_dir, findings);
    validate_script_references(spex_dir, rs_specs, findings);
    validate_ac_order(spex_dir, findings);
}

fn yaml_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("yaml"))
        .collect();
    files.sort();
    files
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
        &yaml_files(&spex_dir.join(FUNCTIONAL_SPECS_DIR)),
        "fs-spec.schema.json",
        findings,
    );
    validate_schema_group(
        spex_dir,
        &yaml_files(&spex_dir.join(RULE_SPECS_DIR)),
        "rs-spec.schema.json",
        findings,
    );

    for file in yaml_files(&spex_dir.join(FUNCTIONAL_SPECS_DIR)) {
        check_filename_matches_id(spex_dir, &file, findings);
    }
    for file in yaml_files(&spex_dir.join(RULE_SPECS_DIR)) {
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

// A spec file must be named after its own id, lowercased
// (RS-NAMING-CONVENTION: uppercase id, lowercase filename).
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
    let id_lower = parsed.id.to_lowercase();
    if id_lower != stem {
        findings.push(conform_finding(
            Some(rel(spex_dir, file)),
            format!("filename stem `{stem}` does not match lowercased id `{id_lower}`"),
        ));
    }
}

// Every script a rs-spec check references must exist at
// `scripts/<spec-id-lowercase>/<script>`.
fn validate_script_references(
    spex_dir: &Path,
    rs_specs: &std::collections::HashMap<String, RuleSpec>,
    findings: &mut Vec<Finding>,
) {
    let mut specs: Vec<&RuleSpec> = rs_specs.values().collect();
    specs.sort_by(|a, b| a.id.cmp(&b.id));

    for spec in specs {
        for check in &spec.checks {
            let Some(script) = &check.script else {
                continue;
            };
            if resolve_script(script, spex_dir, &spec.id).is_none() {
                let dir = spex_dir.join(SCRIPTS_DIR).join(spec.id.to_lowercase());
                findings.push(conform_finding(
                    Some(format!("{RULE_SPECS_DIR}/{}.yaml", spec.id.to_lowercase())),
                    format!("missing script {}/{script}", dir.display()),
                ));
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
    for dir_name in [FUNCTIONAL_SPECS_DIR, RULE_SPECS_DIR] {
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
