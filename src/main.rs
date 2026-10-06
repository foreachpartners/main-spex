mod annotations;
mod commands;
mod commitment;
mod config;
mod error;
mod repo_files;
mod rule;
mod spec;
mod spex;
mod style;
mod templates;
#[cfg(test)]
mod testing;
mod util;

use std::process::ExitCode;

use clap::{Parser, Subcommand};

use commands::check::CheckArgs;
use commands::check_schema::CheckSchemaArgs;
use commands::coverage::CoverageArgs;
use commands::info::InfoArgs;
use commands::rules::RulesArgs;
use commands::specs::SpecsArgs;
use error::Result;

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
        about = "Rewrite bundled skills, schemas, scripts and rules with the latest versions"
    )]
    Update,
    #[command(about = "Run rule checks against the workspace")]
    Check(CheckArgs),
    #[command(about = "Validate JSON or YAML files against a JSON Schema")]
    CheckSchema(CheckSchemaArgs),
    #[command(about = "Report annotation coverage of active specs")]
    Coverage(CoverageArgs),
    #[command(about = "Print `spex.yaml`: project and per-repository configuration")]
    Info(InfoArgs),
    #[command(about = "Print the active rules per repository, as YAML")]
    Rules(RulesArgs),
    #[command(about = "Print the workspace specs, as YAML")]
    Specs(SpecsArgs),
}

fn main() -> ExitCode {
    run(Cli::parse().command).unwrap_or_else(|report| {
        eprintln!("error: {report:?}");
        ExitCode::FAILURE
    })
}

fn run(command: Commands) -> Result<ExitCode> {
    match command {
        Commands::Init => commands::init::run(),
        Commands::Update => commands::update::run(),
        Commands::Check(args) => commands::check::run(args),
        Commands::CheckSchema(args) => commands::check_schema::run(args),
        Commands::Coverage(args) => commands::coverage::run(args),
        Commands::Info(args) => commands::info::run(args),
        Commands::Rules(args) => commands::rules::run(args),
        Commands::Specs(args) => commands::specs::run(args),
    }
}
