use std::fs;
use std::path::{Path, PathBuf};

use error_stack::{Report, ResultExt};
use serde::Deserialize;
use thiserror::Error;

use crate::tech_spec::{CheckOverride, RepoCheck};

#[cfg(test)]
#[path = "config_test.rs"]
mod config_test;

pub const SPEX_CONFIG_FILE: &str = "spex.yaml";
pub const SPEX_DIR_NAME: &str = "spex";
pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

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
    pub cli_version: String,
    #[serde(default)]
    pub repositories: Vec<Repository>,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    #[serde(rename = "repository_name")]
    pub name: String,
    pub path: String,
    #[serde(default)]
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

// Warns when the version that wrote `spex.yaml` differs from the running CLI so
// the user knows to run `spex update` and resync the bundled templates.
pub fn version_warning(config: &Config) -> Option<String> {
    if config.cli_version == CLI_VERSION {
        return None;
    }
    Some(format!(
        "`{SPEX_CONFIG_FILE}` was written by spex {}, but the installed CLI is {CLI_VERSION} — run `spex update` to sync",
        config.cli_version
    ))
}

// Rewrites the `cli_version` line in raw `spex.yaml` text, preserving comments
// and field order. Inserts it after `project_name` when absent.
pub fn set_cli_version(raw: &str, version: &str) -> String {
    let new_line = format!("cli_version: {version}");

    if raw.lines().any(|line| line.starts_with("cli_version:")) {
        let mut out = String::with_capacity(raw.len());
        for line in raw.lines() {
            if line.starts_with("cli_version:") {
                out.push_str(&new_line);
            } else {
                out.push_str(line);
            }
            out.push('\n');
        }
        return out;
    }

    let mut out = String::with_capacity(raw.len() + new_line.len() + 1);
    let mut inserted = false;
    for line in raw.lines() {
        out.push_str(line);
        out.push('\n');
        if !inserted && line.starts_with("project_name:") {
            out.push_str(&new_line);
            out.push('\n');
            inserted = true;
        }
    }
    if !inserted {
        out.insert_str(0, &format!("{new_line}\n"));
    }
    out
}
