use std::path::PathBuf;

use anstream::eprintln;
use eros::bail;
use owo_colors::OwoColorize;

use crate::config::{Layout, Package, PackageConfig, SPEX_CONFIG_FILE, SpexConfig};
use crate::error::Result;
use crate::rule::{self, Rule};
use crate::spec::{self, Spec, SpecCoverageIgnore};
use crate::support::fs::{current_dir, normalize};

#[cfg(test)]
#[path = "spex_test.rs"]
mod spex_test;

pub struct Spex {
    pub dir: PathBuf,
    pub project_name: String,
    pub layout: Layout,
    pub agent_root: PathBuf,
    pub packages: Vec<Package>,
    pub coverage_ignore: Vec<SpecCoverageIgnore>,
}

impl Spex {
    pub fn locate_spex_config() -> Result<PathBuf> {
        SpexConfig::find_file(&current_dir()?)
    }

    pub fn discover() -> Result<Self> {
        let (dir, config) = Self::discover_config()?;
        Self::from_config(dir, config).with_package_configs()
    }

    pub fn discover_config() -> Result<(PathBuf, SpexConfig)> {
        let config_path = Self::locate_spex_config()?;
        let config = SpexConfig::load(&config_path)?;
        warn_if_cli_outdated(&config);
        Ok((SpexConfig::dir_of(&config_path).to_path_buf(), config))
    }

    pub fn from_config(dir: PathBuf, config: SpexConfig) -> Self {
        let packages = config
            .packages
            .into_iter()
            .map(|package| Package::from_config(package, &dir))
            .collect();
        Self {
            agent_root: normalize(&dir.join(&config.agent_root)),
            dir,
            project_name: config.project_name,
            layout: config.layout,
            packages,
            coverage_ignore: config.coverage_ignore,
        }
    }

    pub fn with_package_configs(mut self) -> Result<Self> {
        for package in &mut self.packages {
            let package_config = PackageConfig::load(&package.path)?;
            self.coverage_ignore.extend(package_config.coverage_ignore);
            package.ignored_errors.extend(package_config.ignored_errors);
        }
        Ok(self)
    }

    pub fn rules(&self) -> Result<Vec<Rule>> {
        Rule::load_all(&self.dir.join(rule::RULES_DIR))
    }

    pub fn specs(&self) -> Result<Vec<Spec>> {
        let mut specs = Spec::load_all(&self.dir.join(spec::SPECS_DIR))?;
        for spec in &mut specs {
            let waivers = self
                .coverage_ignore
                .iter()
                .filter(|waiver| waiver.spec_id == spec.id)
                .map(|waiver| waiver.ignore.clone());
            spec.coverage_ignore.extend(waivers);
        }
        Ok(specs)
    }

    pub fn validate_package_names(&self, names: &[String]) -> Result<()> {
        for name in names {
            if !self
                .packages
                .iter()
                .any(|package| package.matches_any_package_name(name))
            {
                bail!("unknown package `{name}` — not listed in {SPEX_CONFIG_FILE}");
            }
        }
        Ok(())
    }

    pub fn select_packages(&self, names: &[String]) -> Result<Vec<&Package>> {
        self.validate_package_names(names)?;
        Ok(self
            .packages
            .iter()
            .filter(|package| {
                names.is_empty()
                    || names
                        .iter()
                        .any(|name| package.matches_any_package_name(name))
            })
            .collect())
    }

    pub fn package_short_names(&self, names: &[String]) -> Result<Vec<&str>> {
        if names.is_empty() {
            return Ok(Vec::new());
        }
        Ok(self
            .select_packages(names)?
            .into_iter()
            .map(|package| package.package_short_name.as_str())
            .collect())
    }
}

fn warn_if_cli_outdated(config: &SpexConfig) {
    if let Some(warning) = config.version_warning() {
        eprintln!("{} {warning}", "warning:".yellow());
    }
}
