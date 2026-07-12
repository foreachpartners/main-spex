use std::path::Path;

use error_stack::Report;
use serde::Deserialize;
use thiserror::Error;

use crate::spec;
use crate::util;

pub use crate::spec::Status;

#[cfg(test)]
#[path = "functional_spec_test.rs"]
mod functional_spec_test;

pub const FUNCTIONAL_SPECS_DIR: &str = "fs-specs";

type Result<T> = core::result::Result<T, Report<FunctionalSpecError>>;

#[derive(Debug, Error)]
pub enum FunctionalSpecError {
    #[error("could not read the `{FUNCTIONAL_SPECS_DIR}` directory")]
    ReadDir,
    #[error("could not read functional spec `{0}`")]
    ReadFile(String),
    #[error("could not parse functional spec `{0}`")]
    Parse(String),
}

#[derive(Debug, Deserialize)]
pub struct FunctionalSpec {
    pub id: String,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    #[serde(default)]
    pub blocked_by: Vec<String>,
    #[serde(default)]
    pub status: Status,
    #[serde(default)]
    pub coverage_ignore: Vec<CoverageLayer>,
}

impl FunctionalSpec {
    pub fn ac_counts(&self) -> (usize, usize) {
        spec::ac_counts(&self.acceptance_criteria)
    }
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

pub fn load_all(specs_dir: &Path) -> Result<Vec<FunctionalSpec>> {
    let mut specs: Vec<FunctionalSpec> = util::load_yaml_dir(
        specs_dir,
        || FunctionalSpecError::ReadDir,
        FunctionalSpecError::ReadFile,
        FunctionalSpecError::Parse,
    )?;

    specs.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(specs)
}
