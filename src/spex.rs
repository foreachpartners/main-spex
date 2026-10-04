use std::collections::HashMap;
use std::path::PathBuf;

use error_stack::{Report, ResultExt};
use thiserror::Error;

use crate::config::{self, Config};
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
        config::find_dir(&current_dir).change_context(WorkspaceError::Config)
    }

    pub fn discover() -> Result<Self> {
        let dir = Self::locate_spex_config()?;
        let config = config::load(&dir).change_context(WorkspaceError::Config)?;
        Ok(Self { dir, config })
    }

    pub fn warn_if_cli_outdated(&self) {
        if let Some(warning) = config::version_warning(&self.config) {
            let style = Style::detect();
            eprintln!("{} {warning}", style.yellow("warning:"));
        }
    }

    pub fn rules(&self) -> Result<HashMap<String, Rule>> {
        rule::load_all(&self.dir.join(rule::RULES_DIR)).change_context(WorkspaceError::Rules)
    }

    // Every rule file, keeping files with colliding ids separate — see
    // `rule::load_all_list`.
    pub fn rules_list(&self) -> Result<Vec<Rule>> {
        rule::load_all_list(&self.dir.join(rule::RULES_DIR)).change_context(WorkspaceError::Rules)
    }

    pub fn specs(&self) -> Result<Vec<Spec>> {
        spec::load_all(&self.dir.join(spec::SPECS_DIR)).change_context(WorkspaceError::Specs)
    }
}
