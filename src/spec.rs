use std::path::Path;

use error_stack::Report;
use serde::Deserialize;
use thiserror::Error;

use crate::util;

#[cfg(test)]
#[path = "spec_test.rs"]
mod spec_test;

pub const SPECS_DIR: &str = "specs";

type Result<T> = core::result::Result<T, Report<SpecError>>;

#[derive(Debug, Error)]
pub enum SpecError {
    #[error("could not read the `{SPECS_DIR}` directory")]
    ReadDir,
    #[error("could not read spec `{0}`")]
    ReadFile(String),
    #[error("could not parse spec `{0}`")]
    Parse(String),
}

#[derive(Debug, Deserialize)]
pub struct Spec {
    pub id: String,
    #[serde(skip, default)]
    pub path: String,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    #[serde(default)]
    pub blocked_by: Vec<String>,
    #[serde(default)]
    pub scope: Vec<String>,
    #[serde(default)]
    pub status: Status,
    #[serde(default)]
    pub coverage_ignore: Vec<CoverageLayer>,
}

impl Spec {
    pub fn ac_counts(&self) -> (usize, usize) {
        ac_counts(&self.acceptance_criteria)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Active,
    #[default]
    Planned,
    Blocked,
    Deprecated,
}

impl Status {
    pub const ALL: [Status; 4] = [
        Status::Active,
        Status::Planned,
        Status::Blocked,
        Status::Deprecated,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Status::Active => "active",
            Status::Planned => "planned",
            Status::Blocked => "blocked",
            Status::Deprecated => "deprecated",
        }
    }

    pub fn is_implemented(self) -> bool {
        matches!(self, Status::Active | Status::Deprecated)
    }
}

fn ac_implemented(criterion: &str) -> bool {
    criterion.trim_start().starts_with("[x]")
}

pub fn ac_counts(criteria: &[String]) -> (usize, usize) {
    let done = criteria
        .iter()
        .filter(|criterion| ac_implemented(criterion))
        .count();
    (done, criteria.len())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CoverageLayer {
    Contract,
    Impl,
    Test,
}

impl CoverageLayer {
    pub const ALL: [CoverageLayer; 3] = [
        CoverageLayer::Impl,
        CoverageLayer::Test,
        CoverageLayer::Contract,
    ];

    pub fn label(self) -> &'static str {
        match self {
            CoverageLayer::Contract => "contract",
            CoverageLayer::Impl => "impl",
            CoverageLayer::Test => "test",
        }
    }
}

pub fn load_all(specs_dir: &Path) -> Result<Vec<Spec>> {
    let loaded = util::load_yaml_dir(
        specs_dir,
        || SpecError::ReadDir,
        SpecError::ReadFile,
        SpecError::Parse,
    )?;

    let mut specs: Vec<Spec> = loaded
        .into_iter()
        .map(|(mut spec, path): (Spec, String)| {
            spec.path = path;
            spec
        })
        .collect();
    specs.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(specs)
}
