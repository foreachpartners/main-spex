use std::process::ExitCode;

use clap::Args;
use error_stack::{Report, ResultExt};

use crate::config::spex::Spex;
use crate::error::{CliError, Result};

#[derive(Args)]
pub struct InfoArgs {
    #[arg(
        long,
        help = "Print a single top-level scalar field (e.g. `project_name`)"
    )]
    print: Option<String>,
    #[arg(
        long,
        num_args = 1..,
        help = "Limit output to the general fields plus the named repositories (by name or short_name)"
    )]
    repo: Vec<String>,
}

pub fn run(args: InfoArgs) -> Result<ExitCode> {
    let spex = Spex::discover_and_warn()?;
    let value = serde_json::to_value(&spex.config).change_context_lazy(serialize_error)?;

    if let Some(field) = &args.print {
        return print_field(&value, field);
    }

    if args.repo.is_empty() {
        let json = serde_json::to_string_pretty(&value).change_context_lazy(serialize_error)?;
        println!("{json}");
        return Ok(ExitCode::SUCCESS);
    }

    let repos = args
        .repo
        .iter()
        .map(|repo_filter| {
            spex.config
                .repositories
                .iter()
                .find(|r| r.matches_any_repo_name(repo_filter))
                .ok_or_else(|| Report::new(CliError::UnknownRepo(repo_filter.clone())))
        })
        .collect::<Result<Vec<_>>>()?;

    let filtered = serde_json::json!({
        "project_name": spex.config.project_name,
        "cli_version": spex.config.cli_version,
        "repositories": repos,
    });
    let json = serde_json::to_string_pretty(&filtered).change_context_lazy(serialize_error)?;
    println!("{json}");

    Ok(ExitCode::SUCCESS)
}

fn serialize_error() -> CliError {
    CliError::Internal("could not serialize the spex configuration".into())
}

fn print_field(value: &serde_json::Value, field: &str) -> Result<ExitCode> {
    let Some(found) = value.get(field) else {
        return Err(Report::new(CliError::Custom(format!(
            "no field `{field}` in the spex configuration"
        ))));
    };

    match found {
        serde_json::Value::String(s) => println!("{s}"),
        serde_json::Value::Number(n) => println!("{n}"),
        serde_json::Value::Bool(b) => println!("{b}"),
        _ => {
            return Err(Report::new(CliError::Custom(format!(
                "field `{field}` is not a single value"
            ))));
        }
    }

    Ok(ExitCode::SUCCESS)
}
