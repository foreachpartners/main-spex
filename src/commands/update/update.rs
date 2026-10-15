use std::fs;
use std::process::ExitCode;

use anstream::println;
use clap::Args;
use eros::Context;
use owo_colors::OwoColorize;

use crate::config::spex::Spex;
use crate::config::{CLI_VERSION, Layout, SPEX_CONFIG_FILE, SpexConfig, spex_yaml_edit};
use crate::error::{Result, could_not_read};
use crate::support::fs::{normalize, write_file};
use crate::support::text::level_two_sections;
use serde::Deserialize;

use super::migration_guide;
use super::sync_templates::sync_templates;

#[cfg(test)]
#[path = "update_test.rs"]
mod update_test;

const CHANGELOG: &str = include_str!("../../../CHANGELOG.md");

#[derive(Args)]
pub struct UpdateArgs {
    #[arg(
        long,
        value_name = "FROM_VERSION",
        help = "Print the migration steps after FROM_VERSION, oldest first, instead of updating"
    )]
    migration_guide: Option<String>,
}

#[derive(Deserialize, Default)]
struct UpdateConfig {
    #[serde(default)]
    cli_version: String,
    #[serde(default)]
    rules_unused: Vec<String>,
    #[serde(default)]
    agent_root: Option<String>,
}

pub fn run(args: UpdateArgs) -> Result<ExitCode> {
    match args.migration_guide {
        Some(from_version) => migration_guide::print_since(&from_version),
        None => update(),
    }
}

fn update() -> Result<ExitCode> {
    let config_path = Spex::locate_spex_config().with_context(|| {
        format!(
            "`{SPEX_CONFIG_FILE}` not found — run `spex update` inside an initialized spex project"
        )
    })?;

    let raw = fs::read_to_string(&config_path).with_context(|| could_not_read(&config_path))?;

    let previous = serde_yaml_ng::from_str::<UpdateConfig>(&raw).unwrap_or_default();

    let spex_dir = SpexConfig::dir_of(&config_path);
    let agent_root = normalize(
        &spex_dir.join(
            previous
                .agent_root
                .as_deref()
                .unwrap_or(Layout::MultiRepo.agent_root()),
        ),
    );
    sync_templates(spex_dir, &agent_root, &previous.rules_unused)
        .context("could not rewrite bundled templates")?;

    write_file(
        &config_path,
        spex_yaml_edit::set_cli_version(&raw, CLI_VERSION),
    )?;

    println!(
        "{}",
        format!("spex skills, schemas and rules updated, cli_version set to {CLI_VERSION}").green()
    );

    let entries = changelog_since(CHANGELOG, &previous.cli_version);
    if !entries.is_empty() {
        println!(
            "\n{}\n",
            format!("Changes since {}:", previous.cli_version).bold()
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
