use std::process::ExitCode;

use clap::Args;
use error_stack::{Report, ResultExt};

use crate::error::{CliError, Result};
use crate::spex::Spex;

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
    let workspace = Spex::discover().change_context(CliError::Workspace)?;
    workspace.warn_if_cli_outdated();
    let value = serde_json::to_value(&workspace.config).change_context(CliError::Serialize)?;

    if let Some(field) = &args.print {
        return print_field(&value, field);
    }

    if args.repo.is_empty() {
        let json = serde_json::to_string_pretty(&value).change_context(CliError::Serialize)?;
        println!("{json}");
        return Ok(ExitCode::SUCCESS);
    }

    let repos = args
        .repo
        .iter()
        .map(|repo_filter| {
            workspace
                .config
                .repositories
                .iter()
                .find(|r| r.matches_any_repo_name(repo_filter))
                .ok_or_else(|| Report::new(CliError::UnknownRepo(repo_filter.clone())))
        })
        .collect::<Result<Vec<_>>>()?;

    let filtered = serde_json::json!({
        "project_name": workspace.config.project_name,
        "cli_version": workspace.config.cli_version,
        "repositories": repos,
    });
    let json = serde_json::to_string_pretty(&filtered).change_context(CliError::Serialize)?;
    println!("{json}");

    Ok(ExitCode::SUCCESS)
}

fn print_field(value: &serde_json::Value, field: &str) -> Result<ExitCode> {
    let Some(found) = value.get(field) else {
        return Err(Report::new(CliError::UnknownField(field.to_string())));
    };

    match found {
        serde_json::Value::String(s) => println!("{s}"),
        serde_json::Value::Number(n) => println!("{n}"),
        serde_json::Value::Bool(b) => println!("{b}"),
        _ => return Err(Report::new(CliError::NotScalar(field.to_string()))),
    }

    Ok(ExitCode::SUCCESS)
}
