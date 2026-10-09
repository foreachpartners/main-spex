use std::path::{Path, PathBuf};

use super::spex_config::{PreexistingError, RepositoryConfig};
use crate::rule::RuleId;
use crate::support::fs::canonical;

#[cfg(test)]
#[path = "repository_test.rs"]
mod repository_test;

#[derive(Debug)]
pub struct Repository {
    pub repository_name: String,
    pub repository_short_name: String,
    pub path: PathBuf,
    pub contracts: Vec<PathBuf>,
    pub rules: Vec<RuleId>,
    pub ignored_errors: Vec<PreexistingError>,
    pub is_spex_repo: bool,
}

impl Repository {
    pub fn from_config(config: RepositoryConfig, spex_dir: &Path) -> Self {
        let path = spex_dir.join(&config.path);
        Self {
            repository_name: config.repository_name,
            repository_short_name: config.repository_short_name,
            is_spex_repo: canonical(&path) == canonical(spex_dir),
            path,
            contracts: config
                .contracts
                .into_iter()
                .map(|contract| spex_dir.join(contract.path))
                .collect(),
            rules: config.rules,
            ignored_errors: config.ignored_errors,
        }
    }

    pub fn matches_any_repo_name(&self, name: &str) -> bool {
        self.repository_name == name || self.repository_short_name == name
    }
}
