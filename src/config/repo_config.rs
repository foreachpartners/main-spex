use std::fs;
use std::path::Path;

use eros::{Context, IntoAnyUnion};
use serde::Deserialize;

use super::spex_config::PreexistingError;
use crate::error::{Result, could_not_parse, could_not_read};
use crate::spec::SpecCoverageIgnore;

#[cfg(test)]
#[path = "repo_config_test.rs"]
mod repo_config_test;

pub const REPO_CONFIG_FILE: &str = "spex.repo.yaml";

#[derive(Debug, Default, Deserialize)]
pub struct RepoConfig {
    #[serde(default)]
    pub coverage_ignore: Vec<SpecCoverageIgnore>,
    #[serde(default)]
    pub ignored_errors: Vec<PreexistingError>,
}

impl RepoConfig {
    pub fn load(repo_path: &Path) -> Result<Self> {
        let config_path = repo_path.join(REPO_CONFIG_FILE);
        if !config_path.is_file() {
            return Ok(Self::default());
        }

        let raw = fs::read_to_string(&config_path).with_context(|| could_not_read(&config_path))?;

        serde_yaml_ng::from_str(&raw)
            .with_context(|| could_not_parse(&config_path))
            .any_union()
    }
}
