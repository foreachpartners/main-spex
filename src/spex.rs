use std::path::PathBuf;

use error_stack::ResultExt;

use crate::config::Config;
use crate::error::{CliError, Result};
use crate::rule::{self, Rule};
use crate::spec::{self, Spec};
use crate::style::Style;

pub struct Spex {
    pub dir: PathBuf,
    pub config: Config,
}

impl Spex {
    pub fn locate_spex_config() -> Result<PathBuf> {
        let current_dir = std::env::current_dir().change_context(CliError::CurrentDir)?;
        Config::find_file(&current_dir).change_context(CliError::LoadConfig)
    }

    pub fn discover() -> Result<Self> {
        let config_path = Self::locate_spex_config()?;
        let config = Config::load(&config_path).change_context(CliError::LoadConfig)?;
        let dir = Config::dir_of(&config_path).to_path_buf();
        Ok(Self { dir, config })
    }

    pub fn warn_if_cli_outdated(&self) {
        if let Some(warning) = self.config.version_warning() {
            let style = Style::detect();
            eprintln!("{} {warning}", style.yellow("warning:"));
        }
    }

    pub fn rules(&self) -> Result<Vec<Rule>> {
        Rule::load_all(&self.dir.join(rule::RULES_DIR)).change_context(CliError::LoadRules)
    }

    pub fn specs(&self) -> Result<Vec<Spec>> {
        Spec::load_all(&self.dir.join(spec::SPECS_DIR)).change_context(CliError::LoadSpecs)
    }
}
