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
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => finish(commands::init::run()),
        Commands::Update => finish(commands::update::run()),
        Commands::Check(args) => finish(commands::check::run(args)),
        Commands::CheckSchema(args) => finish(commands::check_schema::run(args)),
        Commands::Coverage(args) => finish(commands::coverage::run(args)),
        Commands::Info(args) => finish(commands::info::run(args)),
        Commands::Rules(args) => finish(commands::rules::run(args)),
        Commands::Specs(args) => finish(commands::specs::run(args)),
    }
}

fn finish(result: Result<ExitCode>) -> ExitCode {
    match result {
        Ok(code) => code,
        Err(report) => {
            eprintln!("error: {report:?}");
            ExitCode::FAILURE
        }
    }
}
