use std::fs;
use std::path::Path;

use error_stack::{Report, ResultExt};
use serde::Deserialize;
use thiserror::Error;

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
    let mut specs = Vec::new();
    if !specs_dir.is_dir() {
        return Ok(specs);
    }

    let entries = fs::read_dir(specs_dir)
        .change_context(BizSpecError::ReadDir)
        .attach_with(|| format!("could not read `{}`", specs_dir.display()))?;

    for entry in entries {
        let entry = entry.change_context(BizSpecError::ReadDir)?;
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
            .change_context_lazy(|| BizSpecError::ReadFile(file_name.clone()))?;
        let spec: BizSpec = serde_yaml_ng::from_str(&raw)
            .change_context_lazy(|| BizSpecError::Parse(file_name.clone()))?;
        specs.push(spec);
    }

    specs.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(specs)
}
