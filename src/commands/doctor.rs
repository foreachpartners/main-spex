use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anstream::println;
use clap::Args;
use eros::{Context, bail};
use serde::Serialize;

use crate::config::spex::Spex;
use crate::config::workspace::{
    RevisionManifest, RevisionStatus, WorkspaceConfig, configuration_issues,
};
use crate::config::{CLI_VERSION, SpexConfig};
use crate::error::{Result, could_not_read, describe};
use crate::support::output::{OutputFormat, print_json};

#[derive(Args)]
pub struct DoctorArgs {
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,
    #[arg(
        long,
        conflicts_with = "verify_revisions",
        help = "Print a JSON manifest of the clean Git revisions"
    )]
    revisions: bool,
    #[arg(
        long,
        value_name = "PATH",
        help = "Verify all current Git inputs against this manifest"
    )]
    verify_revisions: Option<PathBuf>,
}

#[derive(Serialize)]
struct DoctorReport {
    schema_version: u8,
    cli_version: &'static str,
    executable: PathBuf,
    config_path: PathBuf,
    project: String,
    configured_version: Option<String>,
    version_matches: bool,
    config_issues: Vec<String>,
    migration_guide: String,
    project_git: RevisionStatus,
    packages: Vec<DoctorPackage>,
    revision_issues: Vec<String>,
}

#[derive(Serialize)]
struct DoctorPackage {
    name: String,
    path: PathBuf,
    present: bool,
    git: RevisionStatus,
}

pub fn run(args: DoctorArgs, project: Option<&Path>) -> Result<ExitCode> {
    let config_path = Spex::project_config_path(project)?;
    let raw = fs::read_to_string(&config_path).with_context(|| could_not_read(&config_path))?;
    let config = WorkspaceConfig::parse(&raw, &config_path).unwrap_or_default();
    let config_issues =
        configuration_issues(&raw, &config_path).unwrap_or_else(|error| vec![describe(&error)]);
    let project_name = config.project_name.as_deref().unwrap_or("unknown");
    let dir = SpexConfig::dir_of(&config_path);
    let project_git =
        RevisionStatus::inspect_owned(dir, config.expected_git_root(dir, dir).as_deref());
    let packages: Vec<_> = config
        .package_paths(dir)
        .map(|(name, path)| DoctorPackage {
            name: name.into(),
            present: path.is_dir(),
            git: RevisionStatus::inspect_owned(
                &path,
                config.expected_git_root(dir, &path).as_deref(),
            ),
            path,
        })
        .collect();
    let revisions = || {
        packages
            .iter()
            .map(|package| (package.name.as_str(), &package.git))
    };

    let migration_guide = format!("spex update --migration-guide {}", config.migration_start());
    if args.revisions {
        if !config_issues.is_empty() {
            bail!(
                "project configuration is incompatible. Run {migration_guide}\n{}",
                config_issues.join("\n")
            );
        }
        let manifest = RevisionManifest::capture(project_name, &project_git, revisions())?;
        print_json(&manifest)?;
        return Ok(ExitCode::SUCCESS);
    }

    let revision_issues = if let Some(path) = args.verify_revisions {
        RevisionManifest::read(&path)?.mismatches(project_name, &project_git, revisions())
    } else {
        Vec::new()
    };
    let report = DoctorReport {
        schema_version: 1,
        cli_version: CLI_VERSION,
        executable: env::current_exe()
            .context("could not determine the running spex executable")?,
        config_path,
        project: project_name.into(),
        version_matches: config.cli_version.as_deref() == Some(CLI_VERSION),
        configured_version: config.cli_version,
        config_issues,
        migration_guide,
        project_git,
        packages,
        revision_issues,
    };

    match args.format {
        OutputFormat::Text => print_report(&report),
        OutputFormat::Json => print_json(&report)?,
    }

    if !report.config_issues.is_empty()
        || !report.revision_issues.is_empty()
        || report.packages.iter().any(|package| !package.present)
    {
        return Ok(ExitCode::FAILURE);
    }
    Ok(ExitCode::SUCCESS)
}

fn print_report(report: &DoctorReport) {
    println!(
        "spex {} at {}",
        report.cli_version,
        report.executable.display()
    );
    println!(
        "Project {} at {}",
        report.project,
        report.config_path.display()
    );
    println!(
        "Configured CLI {}",
        report
            .configured_version
            .as_deref()
            .unwrap_or("unspecified")
    );
    println!("Project Git {}", git_description(&report.project_git));
    for package in &report.packages {
        println!(
            "{} at {} ({})",
            package.name,
            package.path.display(),
            git_description(&package.git)
        );
    }
    if !report.version_matches || !report.config_issues.is_empty() {
        println!(
            "Migration steps are available with {}",
            report.migration_guide
        );
    }
    for issue in &report.config_issues {
        println!("Config error {issue}");
    }
    for issue in &report.revision_issues {
        println!("Revision error {issue}");
    }
}

fn git_description(status: &RevisionStatus) -> String {
    match (&status.head, status.dirty) {
        (Some(head), Some(false)) => format!("{head}, clean"),
        (Some(head), Some(true)) => format!("{head}, dirty"),
        (Some(head), None) => format!(
            "{head}, {}",
            status.problem.as_deref().unwrap_or("status unavailable")
        ),
        (None, _) => status
            .problem
            .clone()
            .unwrap_or_else(|| "revision unavailable".into()),
    }
}
