use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use error_stack::{Report, ResultExt, bail};
use include_dir::{Dir, include_dir};
use thiserror::Error;

use crate::config::{SPEX_CONFIG_FILE, SPEX_DIR_NAME};

#[cfg(test)]
#[path = "init_test.rs"]
mod init_test;

const SCRIPTS_DIR_NAME: &str = "scripts";

static TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

type Result<T> = core::result::Result<T, Report<InitError>>;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("`spex init` must be run from inside a directory named `{SPEX_DIR_NAME}`")]
    NotSpexDir,
    #[error("`{SPEX_CONFIG_FILE}` already exists in this directory")]
    AlreadyInitialized,
    #[error("the spex directory has no parent workspace folder")]
    NoWorkspaceRoot,
    #[error("initialization cancelled")]
    Cancelled,
    #[error("an I/O operation failed")]
    Io,
}

struct Repository {
    name: String,
    path: String,
}

pub fn run() -> Result<ExitCode> {
    let current_dir = std::env::current_dir()
        .change_context(InitError::Io)
        .attach("could not determine the current directory")?;

    verify_spex_dir(&current_dir)?;
    verify_not_initialized(&current_dir)?;

    let workspace_root = current_dir.parent().ok_or_else(|| {
        Report::new(InitError::NoWorkspaceRoot)
            .attach("expected the spex directory to live inside a workspace root folder")
    })?;

    let default_name = dir_name(workspace_root).ok_or_else(|| {
        Report::new(InitError::NoWorkspaceRoot).attach("could not read the workspace folder name")
    })?;

    let project_name = confirm_project_name(default_name)?;

    let repositories = discover_repositories(workspace_root)?;
    let repositories = confirm_repositories(repositories)?;

    let config_path = write_config(&current_dir, &project_name, &repositories)?;
    copy_templates(&current_dir)?;

    println!("Created {}", config_path.display());
    Ok(ExitCode::SUCCESS)
}

fn copy_templates(spex_dir: &Path) -> Result<()> {
    TEMPLATES
        .extract(spex_dir)
        .change_context(InitError::Io)
        .attach_with(|| format!("could not write templates into `{}`", spex_dir.display()))?;

    mark_scripts_executable(&spex_dir.join(SCRIPTS_DIR_NAME))
}

#[cfg(unix)]
fn mark_scripts_executable(scripts_dir: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    if !scripts_dir.is_dir() {
        return Ok(());
    }

    let entries = fs::read_dir(scripts_dir)
        .change_context(InitError::Io)
        .attach_with(|| format!("could not read `{}`", scripts_dir.display()))?;

    for entry in entries {
        let path = entry.change_context(InitError::Io)?.path();
        let mut permissions = fs::metadata(&path)
            .change_context(InitError::Io)?
            .permissions();
        permissions.set_mode(permissions.mode() | 0o111);
        fs::set_permissions(&path, permissions)
            .change_context(InitError::Io)
            .attach_with(|| format!("could not set permissions on `{}`", path.display()))?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn mark_scripts_executable(_scripts_dir: &Path) -> Result<()> {
    Ok(())
}

fn write_config(
    spex_dir: &Path,
    project_name: &str,
    repositories: &[Repository],
) -> Result<PathBuf> {
    let config_path = spex_dir.join(SPEX_CONFIG_FILE);
    fs::write(&config_path, config_to_yaml(project_name, repositories))
        .change_context(InitError::Io)
        .attach_with(|| format!("could not write `{}`", config_path.display()))?;
    Ok(config_path)
}

fn verify_spex_dir(current_dir: &Path) -> Result<()> {
    match dir_name(current_dir) {
        Some(name) if name == SPEX_DIR_NAME => Ok(()),
        Some(name) => Err(Report::new(InitError::NotSpexDir)
            .attach(format!("current directory is named `{name}`"))),
        None => Err(Report::new(InitError::NotSpexDir)),
    }
}

fn verify_not_initialized(current_dir: &Path) -> Result<()> {
    if current_dir.join(SPEX_CONFIG_FILE).exists() {
        bail!(InitError::AlreadyInitialized);
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

fn discover_repositories(workspace_root: &Path) -> Result<Vec<Repository>> {
    let entries = fs::read_dir(workspace_root)
        .change_context(InitError::Io)
        .attach_with(|| format!("could not read `{}`", workspace_root.display()))?;

    let mut repositories = Vec::new();
    for entry in entries {
        let entry = entry.change_context(InitError::Io)?;
        let file_type = entry.file_type().change_context(InitError::Io)?;
        if !file_type.is_dir() {
            continue;
        }

        let dir_name = entry.file_name().to_string_lossy().into_owned();
        if dir_name == SPEX_DIR_NAME || dir_name.starts_with('.') {
            continue;
        }

        repositories.push(Repository {
            name: to_kebab_case(&dir_name),
            path: format!("../{dir_name}"),
        });
    }

    repositories.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(repositories)
}

fn confirm_repositories(repositories: Vec<Repository>) -> Result<Vec<Repository>> {
    if repositories.is_empty() {
        let answer = prompt("No sibling repositories found. Continue anyway? [Y/n]: ")?;
        if is_yes(&answer) {
            return Ok(repositories);
        }
        bail!(InitError::Cancelled);
    }

    let names = repositories
        .iter()
        .map(|repo| repo.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    let answer = prompt(&format!(
        "Repositories: {names}\nUse these repositories? [Y/n]: "
    ))?;
    if is_yes(&answer) {
        Ok(repositories)
    } else {
        bail!(InitError::Cancelled);
    }
}

fn config_to_yaml(project_name: &str, repositories: &[Repository]) -> String {
    let mut yaml = format!("name: {project_name}\n");

    if repositories.is_empty() {
        yaml.push_str("repositories: []\n");
    } else {
        yaml.push_str("repositories:\n");
        for repo in repositories {
            yaml.push_str(&format!("  - name: {}\n", repo.name));
            yaml.push_str(&format!("    path: {}\n", repo.path));
            yaml.push_str("    arch-specs: []\n");
        }
    }

    yaml
}

fn dir_name(path: &Path) -> Option<&str> {
    path.file_name().and_then(|name| name.to_str())
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
        .change_context(InitError::Io)
        .attach("could not flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .change_context(InitError::Io)
        .attach("could not read from stdin")?;
    Ok(input)
}

fn is_yes(answer: &str) -> bool {
    matches!(
        answer.trim().to_ascii_lowercase().as_str(),
        "" | "y" | "yes"
    )
}
