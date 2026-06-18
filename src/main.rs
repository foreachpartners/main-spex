mod commands;
mod config;
mod tech_spec;
mod templates;

use std::fmt::Debug;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use error_stack::Report;

use commands::check::CheckArgs;
use commands::check_schema::CheckSchemaArgs;

#[derive(Parser)]
#[command(
    name = "spex",
    version,
    about = "AI-first specification-driven development framework"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Initialize the spex workspace repository")]
    Init,
    #[command(
        about = "Rewrite bundled skills, schemas, scripts and tech-specs with the latest versions"
    )]
    Update,
    #[command(about = "Run technical spec checks against the workspace")]
    Check(CheckArgs),
    #[command(about = "Validate JSON or YAML files against a JSON Schema")]
    CheckSchema(CheckSchemaArgs),
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => finish(commands::init::run()),
        Commands::Update => finish(commands::update::run()),
        Commands::Check(args) => finish(commands::check::run(args)),
        Commands::CheckSchema(args) => finish(commands::check_schema::run(args)),
    }
}

fn finish<E: Debug>(result: Result<ExitCode, Report<E>>) -> ExitCode {
    match result {
        Ok(code) => code,
        Err(report) => {
            eprintln!("error: {report:?}");
            ExitCode::FAILURE
        }
    }
}
