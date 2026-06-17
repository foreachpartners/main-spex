use std::collections::HashMap;
use std::fs;
use std::path::Path;

use error_stack::{Report, ResultExt};
use serde::Deserialize;
use thiserror::Error;

pub const TECH_SPECS_DIR: &str = "tech-specs";
pub const SCRIPTS_DIR: &str = "scripts";

type Result<T> = core::result::Result<T, Report<SpecError>>;

#[derive(Debug, Error)]
pub enum SpecError {
    #[error("could not read the `{TECH_SPECS_DIR}` directory")]
    ReadDir,
    #[error("could not read tech spec `{0}`")]
    ReadFile(String),
    #[error("could not parse tech spec `{0}`")]
    Parse(String),
}

#[derive(Debug, Deserialize)]
pub struct TechSpec {
    pub id: String,
    #[serde(default)]
    pub include_for_all_repos: bool,
    // Overrides spex-repo inclusion when present; otherwise spex follows `include_for_all_repos`.
    #[serde(default)]
    pub include_for_spex: Option<bool>,
    #[serde(default)]
    pub checks: Vec<Check>,
}

#[derive(Debug, Deserialize)]
pub struct Check {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub kind: CheckKind,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub negate: bool,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckKind {
    Regexp,
    Glob,
    Script,
    Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Warn,
    #[default]
    Error,
}

pub fn load_all(specs_dir: &Path) -> Result<HashMap<String, TechSpec>> {
    let mut specs = HashMap::new();
    if !specs_dir.is_dir() {
        return Ok(specs);
    }

    let entries = fs::read_dir(specs_dir)
        .change_context(SpecError::ReadDir)
        .attach_with(|| format!("could not read `{}`", specs_dir.display()))?;

    for entry in entries {
        let entry = entry.change_context(SpecError::ReadDir)?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }

        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let raw = fs::read_to_string(&path)
            .change_context_lazy(|| SpecError::ReadFile(file_name.clone()))?;
        let spec: TechSpec = serde_yaml_ng::from_str(&raw)
            .change_context_lazy(|| SpecError::Parse(file_name.clone()))?;
        specs.insert(spec.id.clone(), spec);
    }

    Ok(specs)
}
