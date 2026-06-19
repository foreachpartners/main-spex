use std::collections::HashMap;
use std::path::PathBuf;

use error_stack::{Report, ResultExt};
use thiserror::Error;

use crate::biz_spec::{self, BizSpec};
use crate::config::{self, Config};
use crate::style::Style;
use crate::tech_spec::{self, TechSpec};

type Result<T> = core::result::Result<T, Report<WorkspaceError>>;

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("could not determine the current directory")]
    CurrentDir,
    #[error("could not load spex configuration")]
    Config,
    #[error("could not load tech specs")]
    TechSpecs,
    #[error("could not load business specs")]
    BizSpecs,
}

// A resolved spex workspace: the spex directory plus its parsed configuration.
// Commands start from here so directory discovery and config loading live in
// one place.
pub struct Workspace {
    pub dir: PathBuf,
    pub config: Config,
}

impl Workspace {
    pub fn discover() -> Result<Self> {
        let current_dir = std::env::current_dir().change_context(WorkspaceError::CurrentDir)?;
        let dir = config::find_dir(&current_dir).change_context(WorkspaceError::Config)?;
        let config = config::load(&dir).change_context(WorkspaceError::Config)?;
        if let Some(warning) = config::version_warning(&config) {
            let style = Style::detect();
            eprintln!("{} {warning}", style.yellow("warning:"));
        }
        Ok(Self { dir, config })
    }

    pub fn tech_specs(&self) -> Result<HashMap<String, TechSpec>> {
        tech_spec::load_all(&self.dir.join(tech_spec::TECH_SPECS_DIR))
            .change_context(WorkspaceError::TechSpecs)
    }

    pub fn biz_specs(&self) -> Result<Vec<BizSpec>> {
        biz_spec::load_all(&self.dir.join(biz_spec::BIZ_SPECS_DIR))
            .change_context(WorkspaceError::BizSpecs)
    }
}
