use std::process::ExitCode;

use clap::Args;
use error_stack::ResultExt;
use serde::Serialize;
use serde_json::Value;

use crate::config::Repository;
use crate::config::spex::Spex;
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
    let mut config_json = to_json(&config)?;
    let raw_repositories = config
        .repositories
        .iter()
        .map(to_json)
        .collect::<Result<Vec<_>>>()?;
    let spex = Spex::from_config(dir, config);
    let selected = spex.select_repositories(&args.repo)?;

    config_json["repositories"] = Value::Array(
        raw_repositories
            .into_iter()
            .zip(&spex.repositories)
            .filter(|(_, repository)| {
                selected
                    .iter()
                    .any(|picked| std::ptr::eq(*picked, *repository))
            })
            .map(|(json, repository)| repository_with_absolute_path(json, repository))
            .collect(),
    );
    let info = Info {
        spex_dir: spex.dir.to_string_lossy().into_owned(),
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

fn repository_with_absolute_path(mut json: Value, repository: &Repository) -> Value {
    json["path"] = Value::String(canonical(&repository.path).to_string_lossy().into_owned());
    json
}
