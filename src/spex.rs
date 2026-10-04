use std::path::PathBuf;

use error_stack::{Report, ResultExt};
use thiserror::Error;

use crate::config::Config;
use crate::rule::{self, Rule};
use crate::spec::{self, Spec};
use crate::style::Style;

type Result<T> = core::result::Result<T, Report<WorkspaceError>>;

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("could not determine the current directory")]
    CurrentDir,
    #[error("could not load spex configuration")]
    Config,
    #[error("could not load rules")]
    Rules,
    #[error("could not load specs")]
    Specs,
}

pub struct Spex {
    pub dir: PathBuf,
    pub config: Config,
}

impl Spex {
    pub fn locate_spex_config() -> Result<PathBuf> {
        let current_dir = std::env::current_dir().change_context(WorkspaceError::CurrentDir)?;
        Config::find_file(&current_dir).change_context(WorkspaceError::Config)
    }

    pub fn discover() -> Result<Self> {
        let config_path = Self::locate_spex_config()?;
        let config = Config::load(&config_path).change_context(WorkspaceError::Config)?;
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
        Rule::load_all(&self.dir.join(rule::RULES_DIR)).change_context(WorkspaceError::Rules)
    }

    pub fn specs(&self) -> Result<Vec<Spec>> {
        Spec::load_all(&self.dir.join(spec::SPECS_DIR)).change_context(WorkspaceError::Specs)
    }
}
