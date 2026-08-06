use std::process::ExitCode;

use clap::Args;
use error_stack::{Report, ResultExt};
use thiserror::Error;

use crate::workspace::Workspace;

type Result<T> = core::result::Result<T, Report<InfoError>>;

#[derive(Debug, Error)]
pub enum InfoError {
    #[error("could not load the spex workspace")]
    Workspace,
    #[error("could not serialize the spex configuration")]
    Serialize,
    #[error("no field `{0}` in the spex configuration")]
    UnknownField(String),
    #[error("field `{0}` is not a single value (use --json to inspect it)")]
    NotScalar(String),
}

#[derive(Args)]
pub struct InfoArgs {
    #[arg(long, help = "Print the full resolved configuration as JSON")]
    json: bool,
    #[arg(
        long,
        help = "Print a single top-level scalar field (e.g. `project_name`)",
        conflicts_with = "json"
    )]
    print: Option<String>,
}

pub fn run(args: InfoArgs) -> Result<ExitCode> {
    let workspace = Workspace::discover().change_context(InfoError::Workspace)?;
    let value = serde_json::to_value(&workspace.config).change_context(InfoError::Serialize)?;

    if let Some(field) = &args.print {
        return print_field(&value, field);
    }

    if args.json {
        let json = serde_json::to_string_pretty(&value).change_context(InfoError::Serialize)?;
        println!("{json}");
    } else {
        print_human(&workspace.config);
    }

    Ok(ExitCode::SUCCESS)
}

fn print_field(value: &serde_json::Value, field: &str) -> Result<ExitCode> {
    let Some(found) = value.get(field) else {
        return Err(Report::new(InfoError::UnknownField(field.to_string())));
    };

    match found {
        serde_json::Value::String(s) => println!("{s}"),
        serde_json::Value::Number(n) => println!("{n}"),
        serde_json::Value::Bool(b) => println!("{b}"),
        _ => return Err(Report::new(InfoError::NotScalar(field.to_string()))),
    }

    Ok(ExitCode::SUCCESS)
}

fn print_human(config: &crate::config::Config) {
    println!("project_name: {}", config.project_name);
    println!("cli_version: {}", config.cli_version);
    println!("repositories:");
    for repo in &config.repositories {
        println!("  {} ({}) -> {}", repo.name, repo.short_name, repo.path);
    }
}
