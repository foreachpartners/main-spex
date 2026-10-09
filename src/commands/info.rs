use std::process::ExitCode;

use clap::Args;
use error_stack::{Report, ResultExt};
use serde::Serialize;

use crate::config::SpexConfig;
use crate::config::spex::Spex;
use crate::error::{CliError, Result};

#[derive(Args)]
pub struct InfoArgs {
    #[arg(
        long,
        num_args = 1..,
        help = "Limit output to the general fields plus the named repositories (by name or short_name)"
    )]
    repo: Vec<String>,
}

#[derive(Serialize)]
struct Info<'a> {
    spex_dir: String,
    #[serde(flatten)]
    config: &'a SpexConfig,
}

pub fn run(args: InfoArgs) -> Result<ExitCode> {
    let (dir, config) = Spex::discover_config()?;
    let spex_dir = dir.to_string_lossy().into_owned();

    if args.repo.is_empty() {
        let json = serde_json::to_string_pretty(&Info {
            spex_dir,
            config: &config,
        })
        .change_context_lazy(serialize_error)?;
        println!("{json}");
        return Ok(ExitCode::SUCCESS);
    }

    let repos = args
        .repo
        .iter()
        .map(|repo_filter| {
            config
                .repositories
                .iter()
                .find(|r| r.matches_any_repo_name(repo_filter))
                .ok_or_else(|| Report::new(CliError::UnknownRepo(repo_filter.clone())))
        })
        .collect::<Result<Vec<_>>>()?;

    let filtered = serde_json::json!({
        "spex_dir": spex_dir,
        "project_name": config.project_name,
        "cli_version": config.cli_version,
        "repositories": repos,
    });
    let json = serde_json::to_string_pretty(&filtered).change_context_lazy(serialize_error)?;
    println!("{json}");

    Ok(ExitCode::SUCCESS)
}

fn serialize_error() -> CliError {
    CliError::Internal("could not serialize the spex configuration".into())
}
