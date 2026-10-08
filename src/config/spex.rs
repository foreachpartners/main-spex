use std::path::PathBuf;

use anstream::eprintln;
use owo_colors::OwoColorize;

use crate::config::Config;
use crate::error::Result;
use crate::rule::{self, Rule};
use crate::spec::{self, Spec};
use crate::support::fs::current_dir;

pub struct Spex {
    pub dir: PathBuf,
    pub config: Config,
}

impl Spex {
    pub fn locate_spex_config() -> Result<PathBuf> {
        Config::find_file(&current_dir()?)
    }

    pub fn discover() -> Result<Self> {
        let config_path = Self::locate_spex_config()?;
        let config = Config::load(&config_path)?;
        let dir = Config::dir_of(&config_path).to_path_buf();
        let spex = Self { dir, config };
        spex.warn_if_cli_outdated();
        Ok(spex)
    }

    fn warn_if_cli_outdated(&self) {
        if let Some(warning) = self.config.version_warning() {
            eprintln!("{} {warning}", "warning:".yellow());
        }
    }

    pub fn rules(&self) -> Result<Vec<Rule>> {
        Rule::load_all(&self.dir.join(rule::RULES_DIR))
    }

    pub fn specs(&self) -> Result<Vec<Spec>> {
        Spec::load_all(&self.dir.join(spec::SPECS_DIR))
    }
}
