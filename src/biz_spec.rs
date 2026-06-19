use std::path::Path;

use error_stack::Report;
use serde::Deserialize;
use thiserror::Error;

use crate::util;

pub const BIZ_SPECS_DIR: &str = "biz-specs";

type Result<T> = core::result::Result<T, Report<BizSpecError>>;

#[derive(Debug, Error)]
pub enum BizSpecError {
    #[error("could not read the `{BIZ_SPECS_DIR}` directory")]
    ReadDir,
    #[error("could not read biz spec `{0}`")]
    ReadFile(String),
    #[error("could not parse biz spec `{0}`")]
    Parse(String),
}

#[derive(Debug, Deserialize)]
pub struct BizSpec {
    pub id: String,
    #[serde(default)]
    pub status: Status,
    #[serde(default)]
    pub coverage_ignore: Vec<CoverageLayer>,
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
}

pub fn load_all(specs_dir: &Path) -> Result<Vec<BizSpec>> {
    let mut specs: Vec<BizSpec> = util::load_yaml_dir(
        specs_dir,
        || BizSpecError::ReadDir,
        BizSpecError::ReadFile,
        BizSpecError::Parse,
    )?;

    specs.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(specs)
}
