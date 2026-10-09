use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anstream::println;
use error_stack::{Report, ResultExt, bail};
use owo_colors::OwoColorize;

use crate::config::{CLI_VERSION, SPEX_CONFIG_FILE, SPEX_DIR_NAME, SpexConfig};
use crate::error::{CliError, Result};
use crate::spec::SPECS_DIR;
use crate::support::fs::{create_dir_all, current_dir, dir_name, write_file};

use super::scaffold::scaffold;

#[cfg(test)]
#[path = "init_test.rs"]
mod init_test;

const PLANS_DIR_NAME: &str = "plans";
const CONTRACTS_DIR_NAME: &str = "contracts";

const CODE_WORKSPACE_EXTENSION: &str = "code-workspace";
const CONFIG_TEMPLATE: &str = include_str!("../../../templates/spex.yaml");
const CODE_WORKSPACE_TEMPLATE: &str =
    include_str!("../../../templates/project-name.code-workspace");

pub fn run() -> Result<ExitCode> {
    let current_dir = current_dir()?;

    let workspace_root = workspace_root(&current_dir)?;

    let default_name = dir_name(&workspace_root).ok_or_else(|| {
        Report::new(CliError::Internal(
            "could not read the workspace root name".into(),
        ))
    })?;

    let project_name = confirm_project_name(default_name)?;

    // The spex repository directory follows the naming convention <project-name>-spex
    let spex_dir = ensure_spex_dir(&current_dir, &workspace_root, &project_name)?;
    verify_empty(&spex_dir)?;

    write_config(&spex_dir, &project_name)?;
    write_code_workspace(&spex_dir, &project_name)?;
    copy_templates(&spex_dir)?;

    println!(
        "{}",
        "spex repository initialized, open project in workspace and run agent skill `/spex-setup` to add workspace repositories and complete setup"
            .green()
    );
    Ok(ExitCode::SUCCESS)
}

fn copy_templates(spex_dir: &Path) -> Result<()> {
    scaffold(spex_dir)?;

    let specs = spex_dir.join(SPECS_DIR);
    create_dir_all(&specs)?;

    let plans = spex_dir.join(PLANS_DIR_NAME);
    create_dir_all(&plans)?;

    let contracts = spex_dir.join(CONTRACTS_DIR_NAME);
    create_dir_all(&contracts)?;

    Ok(())
}

fn write_config(spex_dir: &Path, project_name: &str) -> Result<()> {
    let config_path = spex_dir.join(SPEX_CONFIG_FILE);
    write_file(&config_path, config_from_template(project_name))?;
    Ok(())
}

fn config_from_template(project_name: &str) -> String {
    let named = CONFIG_TEMPLATE
        .replace(
            "project_name: unknown",
            &format!("project_name: {project_name}"),
        )
        .replace(
            "repository_name: unknown",
            &format!("repository_name: {project_name}-{SPEX_DIR_NAME}"),
        );
    SpexConfig::set_cli_version(&named, CLI_VERSION)
}

fn write_code_workspace(spex_dir: &Path, project_name: &str) -> Result<()> {
    let file_name = format!("{project_name}.{CODE_WORKSPACE_EXTENSION}");
    let path = spex_dir.join(&file_name);
    write_file(&path, CODE_WORKSPACE_TEMPLATE)?;
    Ok(())
}

fn is_inside_spex_dir(current_dir: &Path) -> bool {
    dir_name(current_dir).is_some_and(SpexConfig::is_spex_dir_name)
}

// The workspace root is the current directory, unless `spex init` runs from
// inside the spex repository directory, in which case it is the parent.
fn workspace_root(current_dir: &Path) -> Result<PathBuf> {
    if is_inside_spex_dir(current_dir) {
        let parent = current_dir.parent().ok_or_else(|| {
            Report::new(CliError::Internal(
                "expected the spex directory to live inside a workspace root".into(),
            ))
        })?;
        return Ok(parent.to_path_buf());
    }
    Ok(current_dir.to_path_buf())
}

// Resolves the spex repository directory (`<project-name>-spex`), creating it
// when it does not yet exist. Reuses the current directory when `spex init` runs
// from inside the spex repository.
fn ensure_spex_dir(
    current_dir: &Path,
    workspace_root: &Path,
    project_name: &str,
) -> Result<PathBuf> {
    if is_inside_spex_dir(current_dir) {
        return Ok(current_dir.to_path_buf());
    }

    let spex_dir = workspace_root.join(format!("{project_name}-{SPEX_DIR_NAME}"));
    create_dir_all(&spex_dir)?;
    Ok(spex_dir)
}

fn verify_empty(spex_dir: &Path) -> Result<()> {
    let mut entries =
        fs::read_dir(spex_dir).change_context_lazy(|| CliError::ReadDir(spex_dir.to_path_buf()))?;
    if entries.next().is_some() {
        bail!(CliError::Custom(format!(
            "`{}` is not empty — `spex init` sets up a fresh project in an empty directory, use `spex update` for an initialized one",
            spex_dir.display()
        )));
    }
    Ok(())
}

fn confirm_project_name(default_name: &str) -> Result<String> {
    let default_name = to_kebab_case(default_name);
    let input = prompt(&format!("Project name [{default_name}]: "))?;
    let name = input.trim();
    Ok(if name.is_empty() {
        default_name
    } else {
        to_kebab_case(name)
    })
}

fn to_kebab_case(name: &str) -> String {
    let mut result = String::with_capacity(name.len());
    let mut pending_separator = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_separator && !result.is_empty() {
                result.push('-');
            }
            pending_separator = false;
            result.push(ch.to_ascii_lowercase());
        } else if matches!(ch, ' ' | '_' | '-') {
            pending_separator = true;
        }
    }
    result
}

fn prompt(message: &str) -> Result<String> {
    print!("{message}");
    io::stdout()
        .flush()
        .change_context_lazy(|| CliError::Internal("could not flush stdout".into()))?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .change_context_lazy(|| CliError::Internal("could not read from stdin".into()))?;
    Ok(input)
}
