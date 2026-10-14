use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use clap::ValueEnum;

use anstream::println;
use eros::{Context, bail};
use owo_colors::OwoColorize;

use crate::config::{CLI_VERSION, Layout, SPEX_CONFIG_FILE, spex_yaml_edit};
use crate::error::{Result, could_not_read_dir};
use crate::spec::SPECS_DIR;
use crate::support::fs::{create_dir_all, current_dir, dir_name, normalize, write_file};

use super::agent_files::verify_claude_skills;
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
    let project_root = current_dir()?;

    let default_name = dir_name(&project_root)
        .ok_or_else(|| eros::error!("could not read the project root name"))?;

    let layout = confirm_layout()?;
    let project_name = confirm_project_name(default_name)?;

    let spex_dir = project_root.join(layout.spex_dir_name(&project_name));
    verify_empty(&spex_dir)?;
    let agent_root = normalize(&spex_dir.join(layout.agent_root()));
    verify_claude_skills(&agent_root)?;

    create_dir_all(&spex_dir)?;
    write_config(&spex_dir, layout, &project_name)?;
    if layout.is_multi_repo() {
        write_code_workspace(&spex_dir, &project_name)?;
    }
    copy_templates(&spex_dir, &agent_root)?;

    println!(
        "{}",
        "spex initialized, open project in your editor and run agent skill `/spex-setup` to complete setup"
            .green()
    );
    Ok(ExitCode::SUCCESS)
}

fn copy_templates(spex_dir: &Path, agent_root: &Path) -> Result<()> {
    scaffold(spex_dir, agent_root)?;

    let specs = spex_dir.join(SPECS_DIR);
    create_dir_all(&specs)?;

    let plans = spex_dir.join(PLANS_DIR_NAME);
    create_dir_all(&plans)?;

    let contracts = spex_dir.join(CONTRACTS_DIR_NAME);
    create_dir_all(&contracts)?;

    Ok(())
}

fn write_config(spex_dir: &Path, layout: Layout, project_name: &str) -> Result<()> {
    let config_path = spex_dir.join(SPEX_CONFIG_FILE);
    write_file(&config_path, config_from_template(layout, project_name))?;
    Ok(())
}

fn spex_package_name(project_name: &str) -> String {
    Layout::MultiRepo.spex_dir_name(project_name)
}

fn config_from_template(layout: Layout, project_name: &str) -> String {
    let named = CONFIG_TEMPLATE
        .replace(
            "project_name: unknown",
            &format!("project_name: {project_name}"),
        )
        .replace("layout: unknown", &format!("layout: {layout}"))
        .replace(
            "agent_root: unknown",
            &format!("agent_root: {}", layout.agent_root()),
        )
        .replace(
            "package_name: unknown",
            &format!("package_name: {}", spex_package_name(project_name)),
        );
    spex_yaml_edit::set_cli_version(&named, CLI_VERSION)
}

fn write_code_workspace(spex_dir: &Path, project_name: &str) -> Result<()> {
    let file_name = format!("{project_name}.{CODE_WORKSPACE_EXTENSION}");
    let path = spex_dir.join(&file_name);
    write_file(&path, CODE_WORKSPACE_TEMPLATE)?;
    Ok(())
}

fn is_empty_dir(dir: &Path) -> Result<bool> {
    let mut entries = fs::read_dir(dir).with_context(|| could_not_read_dir(dir))?;
    Ok(entries.next().is_none())
}

fn verify_empty(spex_dir: &Path) -> Result<()> {
    if !spex_dir.exists() || is_empty_dir(spex_dir)? {
        return Ok(());
    }

    bail!(
        "`{}` is not empty — `spex init` sets up a fresh project in an empty directory, use `spex update` for an initialized one",
        spex_dir.display()
    );
}

fn confirm_layout() -> Result<Layout> {
    let layouts = Layout::value_variants();
    println!("Layout:");
    for (index, layout) in layouts.iter().enumerate() {
        println!("  {}) {layout}", index + 1);
    }
    let default_number = layouts
        .iter()
        .position(|layout| *layout == Layout::default())
        .map_or(1, |index| index + 1);
    let input = prompt(&format!("Choose [{default_number}]: "))?;
    parse_layout(input.trim())
}

fn parse_layout(answer: &str) -> Result<Layout> {
    if answer.is_empty() {
        return Ok(Layout::default());
    }
    let layouts = Layout::value_variants();
    let by_number = answer
        .parse::<usize>()
        .ok()
        .and_then(|number| number.checked_sub(1))
        .and_then(|index| layouts.get(index));
    by_number
        .copied()
        .or_else(|| Layout::from_str(answer, true).ok())
        .ok_or_else(|| {
            eros::error!(
                "unknown layout `{answer}` — enter a number from 1 to {} or one of: {}",
                layouts.len(),
                Layout::choices()
            )
        })
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
    io::stdout().flush().context("could not flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("could not read from stdin")?;
    Ok(input)
}
