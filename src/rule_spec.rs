use std::collections::HashMap;
use std::path::Path;

use error_stack::Report;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::util;

pub const RULE_SPECS_DIR: &str = "rs-specs";
pub const SCRIPTS_DIR: &str = "scripts";

type Result<T> = core::result::Result<T, Report<SpecError>>;

#[derive(Debug, Error)]
pub enum SpecError {
    #[error("could not read the `{RULE_SPECS_DIR}` directory")]
    ReadDir,
    #[error("could not read rule spec `{0}`")]
    ReadFile(String),
    #[error("could not parse rule spec `{0}`")]
    Parse(String),
}

#[derive(Debug, Deserialize)]
pub struct RuleSpec {
    pub id: String,
    #[serde(default)]
    pub checks: Vec<Check>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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
    pub script: Option<String>,
    #[serde(default)]
    pub run: RunScope,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckKind {
    Regexp,
    Glob,
    Script,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RunScope {
    #[default]
    PerFile,
    PerRepo,
    Workspace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Warn,
    #[default]
    Error,
}

pub fn load_all(specs_dir: &Path) -> Result<HashMap<String, RuleSpec>> {
    let specs: Vec<RuleSpec> = util::load_yaml_dir(
        specs_dir,
        || SpecError::ReadDir,
        SpecError::ReadFile,
        SpecError::Parse,
    )?;

    Ok(specs
        .into_iter()
        .map(|spec| (spec.id.clone(), spec))
        .collect())
}
