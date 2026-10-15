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
use commands::jq::JqArgs;
use commands::rules::RulesArgs;
use commands::specs::SpecsArgs;
use commands::update::UpdateArgs;
use error::{Result, failure_report};

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
    #[command(about = "Initialize a spex project")]
    Init,
    #[command(about = "Update spex to the latest version (skills, rules, docs, etc.)")]
    Update(UpdateArgs),
    #[command(about = "Run spex and rule checks")]
    Check(CheckArgs),
    #[command(about = "Report coverage")]
    Coverage(CoverageArgs),
    #[command(about = "Validate JSON or YAML files against a JSON Schema")]
    CheckSchema(CheckSchemaArgs),
    #[command(about = "Print project configuration")]
    Info(InfoArgs),
    #[command(about = "Query JSON or YAML with a jq filter")]
    Jq(JqArgs),
    #[command(about = "Print the project rules")]
    Rules(RulesArgs),
    #[command(about = "Print the project specs")]
    Specs(SpecsArgs),
}

const TOOL_FAILURE_EXIT_CODE: u8 = 2;

fn main() -> ExitCode {
    run(Cli::parse().command).unwrap_or_else(|error| {
        eprintln!("{}", failure_report(&error));
        ExitCode::from(TOOL_FAILURE_EXIT_CODE)
    })
}

fn run(command: Commands) -> Result<ExitCode> {
    match command {
        Commands::Init => commands::init::run(),
        Commands::Update(args) => commands::update::run(args),
        Commands::Check(args) => commands::check::run(args),
        Commands::CheckSchema(args) => commands::check_schema::run(args),
        Commands::Coverage(args) => commands::coverage::run(args),
        Commands::Info(args) => commands::info::run(args),
        Commands::Jq(args) => commands::jq::run(args),
        Commands::Rules(args) => commands::rules::run(args),
        Commands::Specs(args) => commands::specs::run(args),
    }
}
