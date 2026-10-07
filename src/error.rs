use std::path::PathBuf;

use error_stack::Report;
use thiserror::Error;

use crate::config::SPEX_CONFIG_FILE;
use crate::rule::RULES_DIR;
use crate::spec::SPECS_DIR;

pub type Result<T> = core::result::Result<T, Report<CliError>>;

#[derive(Debug, Error)]
pub enum CliError {
    #[error(
        "`{SPEX_CONFIG_FILE}` not found — run `spex` from the workspace root or any workspace repository"
    )]
    ConfigNotFound,
    #[error(
        "`{SPEX_CONFIG_FILE}` not found — run `spex update` inside an initialized spex workspace"
    )]
    NotInitialized,
    #[error("could not read directory `{}`", .0.display())]
    ReadDir(PathBuf),
    #[error("could not read `{}`", .0.display())]
    ReadFile(PathBuf),
    #[error("could not parse `{}`", .0.display())]
    ParseFile(PathBuf),
    #[error("could not write `{}`", .0.display())]
    WriteFile(PathBuf),
    #[error("could not create directory `{}`", .0.display())]
    CreateDir(PathBuf),
    #[error("unknown workspace repository `{0}` — not listed in {SPEX_CONFIG_FILE}")]
    UnknownRepo(String),
    #[error("unknown repository short name `{0}` — not listed in {SPEX_CONFIG_FILE}")]
    UnknownScope(String),
    #[error("unknown spec id `{0}` — not found in {SPECS_DIR}")]
    UnknownSpecId(String),
    #[error("unknown rule id `{0}` — not found in {RULES_DIR}")]
    UnknownRuleId(String),
    #[error("{0}")]
    Custom(String),
    #[error("{0}")]
    Internal(String),
}
