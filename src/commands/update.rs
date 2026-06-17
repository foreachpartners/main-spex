use std::process::ExitCode;

use error_stack::{Report, ResultExt};
use thiserror::Error;

use crate::config::{self, SPEX_CONFIG_FILE};
use crate::templates;

type Result<T> = core::result::Result<T, Report<UpdateError>>;

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("could not determine the current directory")]
    CurrentDir,
    #[error(
        "`{SPEX_CONFIG_FILE}` not found — run `spex update` inside an initialized spex workspace"
    )]
    NotInitialized,
    #[error("could not rewrite bundled templates")]
    Templates,
}

pub fn run() -> Result<ExitCode> {
    let current_dir = std::env::current_dir()
        .change_context(UpdateError::CurrentDir)
        .attach("could not determine the current directory")?;

    let spex_dir = config::find_dir(&current_dir).change_context(UpdateError::NotInitialized)?;

    templates::scaffold(&spex_dir).change_context(UpdateError::Templates)?;

    println!("spex skills, schemas, scripts and tech-specs updated");
    Ok(ExitCode::SUCCESS)
}
