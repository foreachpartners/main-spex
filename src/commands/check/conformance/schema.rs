use std::path::{Path, PathBuf};

use crate::commands::check_schema;
use crate::error::Result;
use crate::rule::{RULES_DIR, Rule};
use crate::spec::{SPECS_DIR, Spec};
use crate::support::util::yaml_files;
use crate::templates::SCHEMAS_DIRNAME;

use super::super::finding::{Finding, Location};
use super::conform_finding;

pub(super) fn validate_schema(
    spex_dir: &Path,
    specs: &[Spec],
    rules: &[Rule],
) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();

    let spex_yaml = spex_dir.join(crate::config::SPEX_CONFIG_FILE);
    if spex_yaml.is_file() {
        findings.extend(validate_schema_group(
            spex_dir,
            &[spex_yaml],
            "spex.schema.json",
        ));
    } else {
        findings.push(conform_finding(
            None,
            format!("{} not found", crate::config::SPEX_CONFIG_FILE),
        ));
    }

    findings.extend(validate_schema_group(
        spex_dir,
        &yaml_files(&spex_dir.join(SPECS_DIR))?,
        "spec.schema.json",
    ));
    findings.extend(validate_schema_group(
        spex_dir,
        &yaml_files(&spex_dir.join(RULES_DIR))?,
        "rule.schema.json",
    ));

    for spec in specs {
        findings.extend(check_filename_matches_id(&spec.path, spec.id.as_str()));
    }
    for rule in rules {
        findings.extend(check_filename_matches_id(&rule.path, rule.id.as_str()));
    }
    Ok(findings)
}

fn validate_schema_group(spex_dir: &Path, files: &[PathBuf], schema_name: &str) -> Vec<Finding> {
    if files.is_empty() {
        return Vec::new();
    }
    let schema_path = spex_dir.join(SCHEMAS_DIRNAME).join(schema_name);
    let validator = match check_schema::build_validator(&schema_path) {
        Ok(validator) => validator,
        Err(err) => {
            return vec![conform_finding(
                None,
                format!("could not load schema `{}`: {err:?}", schema_path.display()),
            )];
        }
    };

    let mut findings = Vec::new();
    for file in files {
        match check_schema::validate_file(&validator, file) {
            Ok(messages) => {
                for message in messages {
                    findings.push(conform_finding(Some(Location::in_file(file)), message));
                }
            }
            Err(err) => {
                findings.push(conform_finding(
                    Some(Location::in_file(file)),
                    format!("{err:?}"),
                ));
            }
        }
    }
    findings
}

// A spec or rule file must be named after its own id (naming-convention: id
// and filename are identical).
fn check_filename_matches_id(path: &Path, id: &str) -> Option<Finding> {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    (id != stem).then(|| {
        conform_finding(
            Some(Location::in_file(path)),
            format!("filename stem `{stem}` does not match id `{id}`"),
        )
    })
}
