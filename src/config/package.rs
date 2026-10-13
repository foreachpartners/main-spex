use std::path::{Path, PathBuf};

use super::spex_config::{PackageEntry, PreexistingError};
use crate::rule::RuleId;
use crate::support::fs::{canonical, normalize};

#[cfg(test)]
#[path = "package_test.rs"]
mod package_test;

#[derive(Debug)]
pub struct Package {
    pub package_name: String,
    pub package_short_name: String,
    pub path: PathBuf,
    pub contracts: Vec<PathBuf>,
    pub rules: Vec<RuleId>,
    pub ignored_errors: Vec<PreexistingError>,
    pub is_spex_dir: bool,
}

impl Package {
    pub fn from_config(config: PackageEntry, spex_dir: &Path) -> Self {
        let path = normalize(&spex_dir.join(&config.path));
        Self {
            package_name: config.package_name,
            package_short_name: config.package_short_name,
            is_spex_dir: canonical(&path) == canonical(spex_dir),
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

    pub fn matches_any_package_name(&self, name: &str) -> bool {
        self.package_name == name || self.package_short_name == name
    }
}
