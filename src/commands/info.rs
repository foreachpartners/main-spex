use std::path::Path;
use std::process::ExitCode;

use clap::Args;
use error_stack::{Report, ResultExt};
use serde::Serialize;
use serde_json::Value;

use crate::config::spex::Spex;
use crate::config::{RepositoryConfig, SpexConfig};
use crate::error::{CliError, Result};
use crate::support::fs::canonical;

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
struct Info {
    spex_dir: String,
    #[serde(flatten)]
    config: Value,
}

pub fn run(args: InfoArgs) -> Result<ExitCode> {
    let (dir, config) = Spex::discover_config()?;
    let repositories = named_repositories(&config, &args.repo)?
        .into_iter()
        .map(|repository| repository_with_absolute_path(repository, &dir))
        .collect::<Result<Vec<_>>>()?;

    let mut config_json = to_json(&config)?;
    config_json["repositories"] = Value::Array(repositories);
    let info = Info {
        spex_dir: dir.to_string_lossy().into_owned(),
        config: config_json,
    };
    let json = serde_json::to_string_pretty(&info).change_context_lazy(|| {
        CliError::Internal("could not serialize the spex configuration".into())
    })?;
    println!("{json}");

    Ok(ExitCode::SUCCESS)
}

fn to_json(value: &impl Serialize) -> Result<Value> {
    serde_json::to_value(value).change_context_lazy(|| {
        CliError::Internal("could not serialize the spex configuration".into())
    })
}

fn repository_with_absolute_path(repository: &RepositoryConfig, spex_dir: &Path) -> Result<Value> {
    let absolute_path = canonical(&spex_dir.join(&repository.path));
    let mut json = to_json(repository)?;
    json["path"] = Value::String(absolute_path.to_string_lossy().into_owned());
    Ok(json)
}

fn named_repositories<'a>(
    config: &'a SpexConfig,
    names: &[String],
) -> Result<Vec<&'a RepositoryConfig>> {
    if let Some(unknown) = names.iter().find(|name| {
        !config
            .repositories
            .iter()
            .any(|repo| repo.matches_any_repo_name(name))
    }) {
        return Err(Report::new(CliError::UnknownRepo(unknown.clone())));
    }
    Ok(config
        .repositories
        .iter()
        .filter(|repo| {
            names.is_empty() || names.iter().any(|name| repo.matches_any_repo_name(name))
        })
        .collect())
}
