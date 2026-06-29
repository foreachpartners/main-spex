use std::collections::HashMap;
use std::path::PathBuf;

use error_stack::{Report, ResultExt};
use thiserror::Error;

use crate::capability_spec::{self, CapabilitySpec};
use crate::config::{self, Config};
use crate::engineering_spec::{self, EngineeringSpec};
use crate::style::Style;

type Result<T> = core::result::Result<T, Report<WorkspaceError>>;

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("could not determine the current directory")]
    CurrentDir,
    #[error("could not load spex configuration")]
    Config,
    #[error("could not load engineering specs")]
    EngineeringSpecs,
    #[error("could not load capability specs")]
    CapabilitySpecs,
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

    pub fn es_specs(&self) -> Result<HashMap<String, EngineeringSpec>> {
        engineering_spec::load_all(&self.dir.join(engineering_spec::ENGINEERING_SPECS_DIR))
            .change_context(WorkspaceError::EngineeringSpecs)
    }

    pub fn cs_specs(&self) -> Result<Vec<CapabilitySpec>> {
        capability_spec::load_all(&self.dir.join(capability_spec::CAPABILITY_SPECS_DIR))
            .change_context(WorkspaceError::CapabilitySpecs)
    }
}
