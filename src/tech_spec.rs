use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use error_stack::Report;
use serde::Deserialize;
use thiserror::Error;

use crate::util;

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
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    #[serde(default)]
    pub include_for_all_repos: bool,
    // Overrides spex-repo inclusion when present; otherwise spex follows `include_for_all_repos`.
    #[serde(default)]
    pub include_for_spex: Option<bool>,
    #[serde(default)]
    pub checks: Vec<Check>,
    #[serde(default)]
    pub examples: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
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

// A full check declared per repository in `spex.yaml` under `checks_additional`.
#[derive(Debug, Deserialize)]
pub struct RepoCheck {
    pub spec: String,
    #[serde(flatten)]
    pub check: Check,
}

// A partial patch declared per repository in `spex.yaml` under `checks_override`.
// `spec` + `name` identify the inherited check; every other field overrides it
// when present and is inherited when absent.
#[derive(Debug, Deserialize)]
pub struct CheckOverride {
    pub spec: String,
    pub name: String,
    #[serde(default)]
    pub disable: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "type", default)]
    pub kind: Option<CheckKind>,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub negate: Option<bool>,
    #[serde(default)]
    pub script: Option<String>,
    #[serde(default)]
    pub run: Option<RunScope>,
    #[serde(default)]
    pub include: Option<Vec<String>>,
    #[serde(default)]
    pub exclude: Option<Vec<String>>,
    #[serde(default)]
    pub severity: Option<Severity>,
}

impl CheckOverride {
    pub fn apply(&self, base: &Check) -> Check {
        Check {
            name: base.name.clone(),
            description: self
                .description
                .clone()
                .or_else(|| base.description.clone()),
            kind: self.kind.unwrap_or(base.kind),
            pattern: self.pattern.clone().or_else(|| base.pattern.clone()),
            negate: self.negate.unwrap_or(base.negate),
            script: self.script.clone().or_else(|| base.script.clone()),
            run: self.run.unwrap_or(base.run),
            include: self.include.clone().unwrap_or_else(|| base.include.clone()),
            exclude: self.exclude.clone().unwrap_or_else(|| base.exclude.clone()),
            severity: self.severity.unwrap_or(base.severity),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckKind {
    Regexp,
    Glob,
    Script,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RunScope {
    #[default]
    PerFile,
    PerRepo,
    Workspace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Warn,
    #[default]
    Error,
}

pub fn load_all(specs_dir: &Path) -> Result<HashMap<String, TechSpec>> {
    let specs: Vec<TechSpec> = util::load_yaml_dir(
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
