use std::path::PathBuf;

use error_stack::Report;
use thiserror::Error;

use crate::config::SPEX_CONFIG_FILE;
use crate::rule::RULES_DIR;
use crate::spec::SPECS_DIR;

pub type Result<T> = core::result::Result<T, Report<CliError>>;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("could not determine the current directory")]
    CurrentDir,
    #[error(
        "`{SPEX_CONFIG_FILE}` not found — run `spex` from the workspace root or any workspace repository"
    )]
    ConfigNotFound,
    #[error(
        "`{SPEX_CONFIG_FILE}` not found — run `spex update` inside an initialized spex workspace"
    )]
    NotInitialized,
    #[error("`{SPEX_CONFIG_FILE}` already exists in this directory")]
    AlreadyInitialized,
    #[error("the spex directory has no parent workspace root")]
    NoWorkspaceRoot,
    #[error("could not read `{SPEX_CONFIG_FILE}`")]
    ReadConfig,
    #[error("could not parse `{SPEX_CONFIG_FILE}`")]
    ParseConfig,
    #[error("could not update `{SPEX_CONFIG_FILE}`")]
    UpdateConfig,
    #[error("could not load spex configuration")]
    LoadConfig,
    #[error("could not load rules")]
    LoadRules,
    #[error("could not load specs")]
    LoadSpecs,
    #[error("could not load the spex workspace")]
    Workspace,
    #[error("could not rewrite bundled templates")]
    Templates,
    #[error("could not walk workspace repository files")]
    Walk,
    #[error("an I/O operation failed")]
    Io,
    #[error("could not read directory `{}`", .0.display())]
    ReadDir(PathBuf),
    #[error("could not read `{}`", .0.display())]
    ReadFile(PathBuf),
    #[error("could not parse `{}`", .0.display())]
    ParseFile(PathBuf),
    #[error("could not write `{}`", .0.display())]
    WriteFile(PathBuf),
    #[error("invalid schema `{}`", .0.display())]
    InvalidSchema(PathBuf),
    #[error("could not serialize output")]
    Serialize,
    #[error("no field `{0}` in the spex configuration")]
    UnknownField(String),
    #[error("field `{0}` is not a single value")]
    NotScalar(String),
    #[error("unknown workspace repository `{0}` — not listed in {SPEX_CONFIG_FILE}")]
    UnknownRepo(String),
    #[error("unknown repository short name `{0}` — not listed in {SPEX_CONFIG_FILE}")]
    UnknownScope(String),
    #[error("unknown spec id `{0}` — not found in {SPECS_DIR}")]
    UnknownSpecId(String),
    #[error("unknown rule id `{0}` — not found in {RULES_DIR}")]
    UnknownRuleId(String),
    #[error("unknown file `{0}` — not found in any workspace repository")]
    UnknownFile(String),
}
