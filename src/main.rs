mod commands;
mod config;
mod error;
mod rule;
mod spec;
mod support;
mod templates;
mod tracing;

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
    about = "AI-first specification-driven development framework cli"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Initialize the spex workspace repository")]
    Init,
    #[command(about = "Update spex to the latest version (skills, rules, docs, etc.)")]
    Update,
    #[command(about = "Run spex and rule checks")]
    Check(CheckArgs),
    #[command(about = "Report coverage")]
    Coverage(CoverageArgs),
    #[command(about = "Validate JSON or YAML files against a JSON Schema")]
    CheckSchema(CheckSchemaArgs),
    #[command(about = "Print project configuration")]
    Info(InfoArgs),
    #[command(about = "Print the project rules")]
    Rules(RulesArgs),
    #[command(about = "Print the project specs")]
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
