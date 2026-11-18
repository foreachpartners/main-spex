use std::fs;
use std::path::Path;
use std::process::ExitCode;

use anstream::println;
use clap::Args;
use eros::{Context, bail};
use owo_colors::OwoColorize;

use crate::config::spex::Spex;
use crate::config::workspace::{WorkspaceConfig, configuration_issues};
use crate::config::{CLI_VERSION, SPEX_CONFIG_FILE, SpexConfig, spex_yaml_edit};
use crate::error::{Result, could_not_parse, could_not_read};
use crate::support::fs::{normalize, relative_to};
use crate::support::text::level_two_sections;

use super::migration_guide;
use super::sync_templates::template_updates;

#[cfg(test)]
#[path = "update_test.rs"]
mod update_test;

const CHANGELOG: &str = include_str!("../../../CHANGELOG.md");

#[derive(Args)]
pub struct UpdateArgs {
    #[arg(
        long,
        value_name = "FROM_VERSION",
        help = "Print the migration steps after FROM_VERSION, newest first, instead of updating"
    )]
    migration_guide: Option<String>,
    #[arg(
        long,
        conflicts_with = "migration_guide",
        help = "Preview managed file updates without writing"
    )]
    dry_run: bool,
}

pub fn run(args: UpdateArgs, project: Option<&Path>) -> Result<ExitCode> {
    match args.migration_guide {
        Some(from_version) => migration_guide::print_since(&from_version),
        None => update(project, args.dry_run),
    }
}

fn update(project: Option<&Path>, dry_run: bool) -> Result<ExitCode> {
    let config_path = Spex::project_config_path(project).with_context(|| {
        format!(
            "`{SPEX_CONFIG_FILE}` not found — run `spex update` inside an initialized spex project"
        )
    })?;

    let raw = fs::read_to_string(&config_path).with_context(|| could_not_read(&config_path))?;

    let previous = WorkspaceConfig::parse(&raw, &config_path)?;
    let previous_version = previous.cli_version.as_deref().unwrap_or("0");
    let updated_config = if previous_version == CLI_VERSION {
        raw
    } else {
        spex_yaml_edit::set_cli_version(&raw, CLI_VERSION)
    };
    let issues = configuration_issues(&updated_config, &config_path)?;
    if !issues.is_empty() {
        bail!(
            "{} is incompatible with spex {CLI_VERSION}. Run spex update --migration-guide {}\n{}",
            config_path.display(),
            previous.migration_start(),
            issues.join("\n")
        );
    }
    let config: SpexConfig =
        serde_yaml_ng::from_str(&updated_config).with_context(|| could_not_parse(&config_path))?;
    let spex_dir = SpexConfig::dir_of(&config_path);
    let agent_root = normalize(&spex_dir.join(&config.agent_root));
    let mut plan = template_updates(spex_dir, &agent_root)
        .context("could not plan bundled template updates")?;
    plan.add(config_path.clone(), updated_config.as_bytes(), false)?;

    if dry_run {
        println!("spex update preview {previous_version} → {CLI_VERSION}");
        if plan.list().is_empty() {
            println!("No managed file changes");
        }
        for edit in plan.list() {
            println!(
                "{} {}",
                edit.action(),
                relative_to(&edit.path, spex_dir).display()
            );
        }
        return Ok(ExitCode::SUCCESS);
    }
    plan.apply()
        .context("could not apply bundled template updates")?;

    println!(
        "{}",
        format!("spex skills, schemas and rules updated, cli_version set to {CLI_VERSION}").green()
    );

    let entries = changelog_since(CHANGELOG, previous_version);
    if !entries.is_empty() {
        println!(
            "\n{}\n",
            format!("Changes since {previous_version}:").bold()
        );
        println!("{}", entries.join("\n\n"));
    }

    Ok(ExitCode::SUCCESS)
}

fn changelog_since(changelog: &str, from_version: &str) -> Vec<String> {
    level_two_sections(changelog)
        .into_iter()
        .take_while(|section| !is_version_header(section, from_version))
        .map(|section| section.trim_end().to_string())
        .collect()
}

fn is_version_header(section: &str, version: &str) -> bool {
    section
        .lines()
        .next()
        .is_some_and(|header| header.starts_with(&format!("## [{version}]")))
}
