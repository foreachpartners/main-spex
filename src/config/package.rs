use std::path::{Path, PathBuf};

use super::spex_config::{PackageEntry, PreexistingError};
use eros::Context;

use crate::error::Result;
use crate::rule::RuleId;
use crate::spec::SpecCoverageIgnore;
use crate::support::fs::{canonical, normalize};
use crate::support::package_files::Exclude;

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
    pub exclude: Vec<String>,
    pub check_ignore: Vec<PreexistingError>,
    pub coverage_ignore: Vec<SpecCoverageIgnore>,
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
            exclude: config.exclude,
            check_ignore: config.check_ignore,
            coverage_ignore: config.coverage_ignore,
        }
    }

    pub fn compile_exclude(&self) -> Result<Exclude> {
        Exclude::new(&self.path, &self.exclude)
            .with_context(|| format!("invalid `exclude` of package `{}`", self.package_name))
    }

    pub fn matches_any_package_name(&self, name: &str) -> bool {
        self.package_name == name || self.package_short_name == name
    }
}
