mod commands;
mod config;
mod rule;
mod spec;
mod style;
mod templates;
mod util;
mod workspace;

use std::fmt::Debug;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use error_stack::Report;

use commands::check::CheckArgs;
use commands::check_schema::CheckSchemaArgs;
use commands::info::InfoArgs;
use commands::rules::RulesArgs;
use commands::specs::SpecsArgs;

#[derive(Parser)]
#[command(
    name = "spex",
    version,
    about = "AI-first specification-driven development framework"
)]
struct Cli {
    #[arg(
        long,
        short,
        global = true,
        help = "Print progress to stderr as work executes"
    )]
    verbose: bool,
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
    Coverage,
    #[command(about = "Print the resolved spex configuration, for scripting")]
    Info(InfoArgs),
    #[command(about = "Print the active rules per repository, as JSON")]
    Rules(RulesArgs),
    #[command(about = "Print the workspace specs, as YAML")]
    Specs(SpecsArgs),
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let verbose = cli.verbose;

    match cli.command {
        Commands::Init => finish(commands::init::run()),
        Commands::Update => finish(commands::update::run()),
        Commands::Check(args) => finish(commands::check::run(args, verbose)),
        Commands::CheckSchema(args) => finish(commands::check_schema::run(args)),
        Commands::Coverage => finish(commands::coverage::run()),
        Commands::Info(args) => finish(commands::info::run(args)),
        Commands::Rules(args) => finish(commands::rules::run(args)),
        Commands::Specs(args) => finish(commands::specs::run(args)),
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
