use std::fs;
use std::process::ExitCode;

use error_stack::{Report, ResultExt};
use thiserror::Error;

use crate::config::{self, CLI_VERSION, SPEX_CONFIG_FILE};
use crate::style::Style;
use crate::templates;
use crate::spex::Spex;
use serde::Deserialize;

#[cfg(test)]
#[path = "update_test.rs"]
mod update_test;

const CHANGELOG: &str = include_str!("../../CHANGELOG.md");

#[derive(Deserialize, Default)]
struct UpdateConfig {
    #[serde(default)]
    cli_version: String,
    #[serde(default)]
    rules_unused: Vec<String>,
}

type Result<T> = core::result::Result<T, Report<UpdateError>>;

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error(
        "`{SPEX_CONFIG_FILE}` not found — run `spex update` inside an initialized spex workspace"
    )]
    NotInitialized,
    #[error("could not rewrite bundled templates")]
    Templates,
    #[error("could not update `{SPEX_CONFIG_FILE}`")]
    Config,
}

pub fn run() -> Result<ExitCode> {
    let spex_dir = Spex::locate_spex_repository().change_context(UpdateError::NotInitialized)?;

    let config_path = spex_dir.join(SPEX_CONFIG_FILE);
    let raw = fs::read_to_string(&config_path)
        .change_context(UpdateError::Config)
        .attach_with(|| format!("could not read `{}`", config_path.display()))?;

    let previous = serde_yaml_ng::from_str::<UpdateConfig>(&raw).unwrap_or_default();

    templates::scaffold(&spex_dir, &previous.rules_unused)
        .change_context(UpdateError::Templates)?;

    fs::write(&config_path, config::set_cli_version(&raw, CLI_VERSION))
        .change_context(UpdateError::Config)
        .attach_with(|| format!("could not write `{}`", config_path.display()))?;

    let style = Style::detect();
    println!(
        "{}",
        style.green(&format!(
            "spex skills, schemas, scripts and rules updated, cli_version set to {CLI_VERSION}"
        ))
    );

    let entries = changelog_since(CHANGELOG, &previous.cli_version);
    if !entries.is_empty() {
        println!(
            "\n{}\n",
            style.bold(&format!("Changes since {}:", previous.cli_version))
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
