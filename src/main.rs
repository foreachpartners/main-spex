mod commands;
mod config;
mod error;
mod rule;
mod spec;
mod support;
mod templates;
mod tracing;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use eros::bail;

use commands::check::CheckArgs;
use commands::check_schema::CheckSchemaArgs;
use commands::context::ContextArgs;
use commands::coverage::CoverageArgs;
use commands::doctor::DoctorArgs;
use commands::info::InfoArgs;
use commands::jq::JqArgs;
use commands::library::LibraryArgs;
use commands::receipt::ReceiptArgs;
use commands::rules::RulesArgs;
use commands::specs::SpecsArgs;
use commands::update::UpdateArgs;
use config::spex::Spex;
use error::{Result, failure_report};

#[derive(Parser)]
#[command(
    name = "spex",
    version,
    about = "AI-first specification-driven development framework cli"
)]
struct Cli {
    #[arg(
        long,
        global = true,
        help = "Directory containing spex.yaml or path to the file"
    )]
    project: Option<PathBuf>,
    #[arg(
        long,
        global = true,
        help = "Require an exact workspace revision manifest for check or coverage"
    )]
    require_revisions: Option<PathBuf>,
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
    #[command(about = "Report trace coverage")]
    Coverage(CoverageArgs),
    #[command(about = "Diagnose workspace versions and package revisions")]
    Doctor(DoctorArgs),
    #[command(about = "Assemble deterministic task context from explicit specs")]
    Context(ContextArgs),
    #[command(about = "Record and retrieve decision and verification receipts")]
    Receipt(ReceiptArgs),
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
    #[command(about = "Print template specs from the bundled library, not the project's own specs")]
    Lib(LibraryArgs),
}

const TOOL_FAILURE_EXIT_CODE: u8 = 2;

fn main() -> ExitCode {
    run(Cli::parse()).unwrap_or_else(|error| {
        eprintln!("{}", failure_report(&error));
        ExitCode::from(TOOL_FAILURE_EXIT_CODE)
    })
}

fn run(cli: Cli) -> Result<ExitCode> {
    use Commands::*;

    let project = cli.project.as_deref();
    let required = cli.require_revisions.as_deref();
    if required.is_some() && !matches!(&cli.command, Check(_) | Coverage(_)) {
        bail!("--require-revisions applies to check and coverage");
    }
    if project.is_some()
        && matches!(
            &cli.command,
            Init | CheckSchema(_) | Jq(_) | Lib(_) | Receipt(_)
        )
    {
        bail!("this command does not use --project");
    }

    match cli.command {
        Init => commands::init::run(),
        Update(args) => commands::update::run(args, project),
        Check(args) => commands::check::run(args, verified_project(project, required)?),
        CheckSchema(args) => commands::check_schema::run(args),
        Coverage(args) => commands::coverage::run(args, verified_project(project, required)?),
        Doctor(args) => commands::doctor::run(args, project),
        Context(args) => commands::context::run(args, project),
        Receipt(args) => commands::receipt::run(args),
        Info(args) => commands::info::run(args, project),
        Jq(args) => commands::jq::run(args),
        Rules(args) => commands::rules::run(args, project),
        Specs(args) => commands::specs::run(args, project),
        Lib(args) => commands::library::run(args),
    }
}

fn verified_project(project: Option<&Path>, required: Option<&Path>) -> Result<Spex> {
    let spex = Spex::load_project(project)?;
    if let Some(manifest) = required {
        spex.verify_revisions(manifest)?;
    }
    Ok(spex)
}
