use std::fs;
use std::process::ExitCode;

use anstream::println;
use error_stack::ResultExt;
use owo_colors::OwoColorize;

use crate::config::spex::Spex;
use crate::config::{CLI_VERSION, SpexConfig};
use crate::error::{CliError, Result};
use crate::support::fs::write_file;
use serde::Deserialize;

use super::sync_templates::sync_templates;

#[cfg(test)]
#[path = "update_test.rs"]
mod update_test;

const CHANGELOG: &str = include_str!("../../../CHANGELOG.md");

#[derive(Deserialize, Default)]
struct UpdateConfig {
    #[serde(default)]
    cli_version: String,
    #[serde(default)]
    rules_unused: Vec<String>,
}

pub fn run() -> Result<ExitCode> {
    let config_path = Spex::locate_spex_config().change_context(CliError::NotInitialized)?;

    let raw = fs::read_to_string(&config_path)
        .change_context_lazy(|| CliError::ReadFile(config_path.clone()))?;

    let previous = serde_yaml_ng::from_str::<UpdateConfig>(&raw).unwrap_or_default();

    sync_templates(SpexConfig::dir_of(&config_path), &previous.rules_unused)
        .change_context_lazy(|| CliError::Internal("could not rewrite bundled templates".into()))?;

    write_file(&config_path, SpexConfig::set_cli_version(&raw, CLI_VERSION))?;

    println!(
        "{}",
        format!(
            "spex skills, schemas, scripts and rules updated, cli_version set to {CLI_VERSION}"
        )
        .green()
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
    let mut sections: Vec<String> = Vec::new();
    let mut current: Option<String> = None;

    for line in changelog.lines() {
        if line.starts_with("## ") {
            if let Some(section) = current.take() {
                sections.push(section);
            }
            current = Some(String::new());
        }
        if let Some(section) = current.as_mut() {
            section.push_str(line);
            section.push('\n');
        }
    }
    if let Some(section) = current {
        sections.push(section);
    }

    let mut result = Vec::new();
    for section in sections {
        if is_version_header(&section, from_version) {
            break;
        }
        result.push(section.trim_end().to_string());
    }
    result
}

fn is_version_header(section: &str, version: &str) -> bool {
    section
        .lines()
        .next()
        .is_some_and(|header| header.starts_with(&format!("## [{version}]")))
}
