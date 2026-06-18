use std::fs;
use std::path::{Path, PathBuf};

use error_stack::{Report, ResultExt};
use serde::Deserialize;
use thiserror::Error;

use crate::tech_spec::{CheckOverride, RepoCheck};

#[cfg(test)]
#[path = "config_test.rs"]
mod config_test;

pub const SPEX_CONFIG_FILE: &str = "spex.yml";
pub const SPEX_DIR_NAME: &str = "spex";

type Result<T> = core::result::Result<T, Report<ConfigError>>;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(
        "`{SPEX_CONFIG_FILE}` not found — run `spex` from the workspace root or any workspace repository"
    )]
    NotFound,
    #[error("could not read `{SPEX_CONFIG_FILE}`")]
    Read,
    #[error("could not parse `{SPEX_CONFIG_FILE}`")]
    Parse,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub project_name: String,
    #[serde(default)]
    pub repositories: Vec<Repository>,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    #[serde(rename = "repository_name")]
    pub name: String,
    pub path: String,
    #[serde(rename = "tech-specs", default)]
    pub tech_specs: Vec<String>,
    #[serde(default)]
    pub checks_additional: Vec<RepoCheck>,
    #[serde(default)]
    pub checks_override: Vec<CheckOverride>,
}

pub fn find_dir(start: &Path) -> Result<PathBuf> {
    for dir in start.ancestors() {
        if dir.join(SPEX_CONFIG_FILE).is_file() {
            return Ok(dir.to_path_buf());
        }
        let nested = dir.join(SPEX_DIR_NAME);
        if nested.join(SPEX_CONFIG_FILE).is_file() {
            return Ok(nested);
        }
    }
    Err(Report::new(ConfigError::NotFound))
}

pub fn load(spex_dir: &Path) -> Result<Config> {
    let config_path = spex_dir.join(SPEX_CONFIG_FILE);
    if !config_path.exists() {
        return Err(Report::new(ConfigError::NotFound));
    }

    let raw = fs::read_to_string(&config_path)
        .change_context(ConfigError::Read)
        .attach_with(|| format!("could not read `{}`", config_path.display()))?;

    serde_yaml_ng::from_str(&raw).change_context(ConfigError::Parse)
}
