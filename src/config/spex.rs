use std::path::PathBuf;

use anstream::eprintln;
use owo_colors::OwoColorize;

use crate::config::{Repository, SpexConfig};
use crate::error::Result;
use crate::rule::{self, Rule};
use crate::spec::{self, Spec};
use crate::support::fs::current_dir;

pub struct Spex {
    pub dir: PathBuf,
    pub project_name: String,
    pub repositories: Vec<Repository>,
}

impl Spex {
    pub fn locate_spex_config() -> Result<PathBuf> {
        SpexConfig::find_file(&current_dir()?)
    }

    pub fn discover() -> Result<Self> {
        let (dir, config) = Self::discover_config()?;
        Ok(Self::from_config(dir, config))
    }

    pub fn discover_config() -> Result<(PathBuf, SpexConfig)> {
        let config_path = Self::locate_spex_config()?;
        let config = SpexConfig::load(&config_path)?;
        warn_if_cli_outdated(&config);
        Ok((SpexConfig::dir_of(&config_path).to_path_buf(), config))
    }

    pub fn from_config(dir: PathBuf, config: SpexConfig) -> Self {
        let repositories = config
            .repositories
            .into_iter()
            .map(|repo| Repository::from_config(repo, &dir))
            .collect();
        Self {
            dir,
            project_name: config.project_name,
            repositories,
        }
    }

    pub fn rules(&self) -> Result<Vec<Rule>> {
        Rule::load_all(&self.dir.join(rule::RULES_DIR))
    }

    pub fn specs(&self) -> Result<Vec<Spec>> {
        Spec::load_all(&self.dir.join(spec::SPECS_DIR))
    }
}

fn warn_if_cli_outdated(config: &SpexConfig) {
    if let Some(warning) = config.version_warning() {
        eprintln!("{} {warning}", "warning:".yellow());
    }
}
