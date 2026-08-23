use std::fs;
use std::process::ExitCode;

use error_stack::{Report, ResultExt};
use thiserror::Error;

use crate::config::{self, CLI_VERSION, SPEX_CONFIG_FILE};
use crate::templates;
use serde::Deserialize;

#[derive(Deserialize, Default)]
struct RuleSpecIdsRemoved {
    #[serde(default)]
    rules_unused: Vec<String>,
}

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
    #[error("could not update `{SPEX_CONFIG_FILE}`")]
    Config,
}

pub fn run() -> Result<ExitCode> {
    let current_dir = std::env::current_dir()
        .change_context(UpdateError::CurrentDir)
        .attach("could not determine the current directory")?;

    let spex_dir = config::find_dir(&current_dir).change_context(UpdateError::NotInitialized)?;

    let config_path = spex_dir.join(SPEX_CONFIG_FILE);
    let raw = fs::read_to_string(&config_path)
        .change_context(UpdateError::Config)
        .attach_with(|| format!("could not read `{}`", config_path.display()))?;

    let removed_rules = serde_yaml_ng::from_str::<RuleSpecIdsRemoved>(&raw)
        .unwrap_or_default()
        .rules_unused;

    templates::scaffold(&spex_dir, &removed_rules)
        .change_context(UpdateError::Templates)?;

    fs::write(&config_path, config::set_cli_version(&raw, CLI_VERSION))
        .change_context(UpdateError::Config)
        .attach_with(|| format!("could not write `{}`", config_path.display()))?;

    println!("spex skills, schemas, scripts and rules updated, cli_version set to {CLI_VERSION}");
    Ok(ExitCode::SUCCESS)
}
