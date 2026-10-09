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
        help = "Filter the repositories list (by name or short_name)"
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
    let (dir, mut config) = Spex::discover_config()?;
    keep_named_repositories(&mut config, &args.repo)?;

    let info = Info {
        spex_dir: dir.to_string_lossy().into_owned(),
        config: &config,
    };
    let json = serde_json::to_string_pretty(&info).change_context_lazy(|| {
        CliError::Internal("could not serialize the spex configuration".into())
    })?;
    println!("{json}");

    Ok(ExitCode::SUCCESS)
}

fn keep_named_repositories(config: &mut SpexConfig, names: &[String]) -> Result<()> {
    if names.is_empty() {
        return Ok(());
    }
    if let Some(unknown) = names.iter().find(|name| {
        !config
            .repositories
            .iter()
            .any(|repo| repo.matches_any_repo_name(name))
    }) {
        return Err(Report::new(CliError::UnknownRepo(unknown.clone())));
    }
    config
        .repositories
        .retain(|repo| names.iter().any(|name| repo.matches_any_repo_name(name)));
    Ok(())
}
