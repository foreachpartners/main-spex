use std::fs;
use std::path::Path;

use error_stack::{Report, ResultExt};
use serde::Deserialize;
use thiserror::Error;

pub const SPEX_CONFIG_FILE: &str = "spex.yml";

type Result<T> = core::result::Result<T, Report<ConfigError>>;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("`{SPEX_CONFIG_FILE}` not found — run `spex check` from inside the `spex` directory")]
    NotFound,
    #[error("could not read `{SPEX_CONFIG_FILE}`")]
    Read,
    #[error("could not parse `{SPEX_CONFIG_FILE}`")]
    Parse,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub name: String,
    #[serde(default)]
    pub repositories: Vec<Repository>,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    pub name: String,
    pub path: String,
    #[serde(rename = "arch-specs", default)]
    pub arch_specs: Vec<String>,
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
