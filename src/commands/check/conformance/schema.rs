use std::path::{Path, PathBuf};

use crate::commands::check_schema;
use crate::rule::RULES_DIR;
use crate::spec::SPECS_DIR;
use crate::templates::SCHEMAS_DIR;
use crate::util::yaml_files;

use super::{Finding, IdOnly, conform_finding, rel};

pub(super) fn validate_schema(spex_dir: &Path, findings: &mut Vec<Finding>) {
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
