mod commands;
mod config;
mod tech_spec;

use std::fmt::Debug;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use error_stack::Report;

use commands::check::CheckArgs;

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
    #[command(about = "Run technical spec checks against the workspace")]
    Check(CheckArgs),
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => finish(commands::init::run()),
        Commands::Check(args) => finish(commands::check::run(args)),
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
