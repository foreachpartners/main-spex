use std::fs;
use std::path::{Path, PathBuf};

use eros::{Context, IntoAnyUnion};
use serde::{Deserialize, Serialize};

use super::Layout;
use crate::error::{Result, config_not_found, could_not_parse, could_not_read};
use crate::rule::RuleId;
use crate::spec::SpecCoverageIgnore;
use crate::support::fs::dir_name;

#[cfg(test)]
#[path = "spex_config_test.rs"]
mod spex_config_test;

pub const SPEX_CONFIG_FILE: &str = "spex.yaml";
pub const SPEX_DIR_NAME: &str = "spex";
pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize, Serialize)]
pub struct SpexConfig {
    pub project_name: String,
    pub cli_version: String,
    pub layout: Layout,
    pub agent_root: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub rules_unused: Vec<RuleId>,
    #[serde(default)]
    pub packages: Vec<PackageEntry>,
    #[serde(default)]
    pub coverage_ignore: Vec<SpecCoverageIgnore>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PackageEntry {
    pub package_name: String,
    pub package_short_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub path: String,
    #[serde(default)]
    pub contracts: Vec<ContractConfig>,
    #[serde(default)]
    pub rules: Vec<RuleId>,
    #[serde(default)]
    pub check_ignore: Vec<PreexistingError>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ContractConfig {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PreexistingError {
    pub rule_id: String,
    pub check: String,
    #[serde(default)]
    pub files: Vec<String>,
}

impl SpexConfig {
    pub fn find_file(start: &Path) -> Result<PathBuf> {
        for dir in start.ancestors() {
            let config_path = dir.join(SPEX_CONFIG_FILE);
            if config_path.is_file() {
                return Ok(config_path);
            }
            if let Some(child_config_path) = Self::spex_child_config_path(dir) {
                return Ok(child_config_path);
            }
        }
        Err(config_not_found())
    }

    pub fn dir_of(config_path: &Path) -> &Path {
        config_path.parent().unwrap_or(Path::new(""))
    }

    pub fn is_spex_dir_name(name: &str) -> bool {
        name == SPEX_DIR_NAME || name.ends_with(&format!("-{SPEX_DIR_NAME}"))
    }

    fn spex_child_config_path(dir: &Path) -> Option<PathBuf> {
        let mut config_paths: Vec<PathBuf> = fs::read_dir(dir)
            .ok()?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| dir_name(path).is_some_and(Self::is_spex_dir_name))
            .map(|path| path.join(SPEX_CONFIG_FILE))
            .filter(|config_path| config_path.is_file())
            .collect();
        config_paths.sort();
        config_paths.into_iter().next()
    }

    pub fn load(config_path: &Path) -> Result<Self> {
        if !config_path.exists() {
            return Err(config_not_found());
        }

        let raw = fs::read_to_string(config_path).with_context(|| could_not_read(config_path))?;

        serde_yaml_ng::from_str(&raw)
            .with_context(|| could_not_parse(config_path))
            .any_union()
    }

    pub fn version_warning(&self) -> Option<String> {
        if self.cli_version == CLI_VERSION {
            return None;
        }
        Some(format!(
            "`{SPEX_CONFIG_FILE}` was written by spex {}, but the installed CLI is {CLI_VERSION} — run `spex update` to sync",
            self.cli_version
        ))
    }
}
