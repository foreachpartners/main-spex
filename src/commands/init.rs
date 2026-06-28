use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use error_stack::{Report, ResultExt, bail};
use thiserror::Error;

use crate::biz_spec::BIZ_SPECS_DIR;
use crate::config::{CLI_VERSION, SPEX_CONFIG_FILE, SPEX_DIR_NAME};
use crate::tech_spec::TechSpec;
use crate::templates;

#[cfg(test)]
#[path = "init_test.rs"]
mod init_test;

const PLANS_DIR_NAME: &str = "plans";
const CONTRACTS_DIR_NAME: &str = "contracts";

type Result<T> = core::result::Result<T, Report<InitError>>;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("`{SPEX_CONFIG_FILE}` already exists in this directory")]
    AlreadyInitialized,
    #[error("the spex directory has no parent workspace root")]
    NoWorkspaceRoot,
    #[error("initialization cancelled")]
    Cancelled,
    #[error("could not parse the bundled tech spec `{0}`")]
    Template(String),
    #[error("an I/O operation failed")]
    Io,
}

const SPEX_REPO_PATH: &str = ".";
const SPEX_WORKSPACE_ICON: &str = "🧰";
const CODE_WORKSPACE_EXTENSION: &str = "code-workspace";

const SPEX_REPO_DESCRIPTION: &str = "    description: >-
      Spex workspace repository. Holds the specs and config that govern the
      project: schemas/ (JSON Schemas for spex.yaml and the spec files),
      tech-specs/ (cross-cutting technical conventions) and biz-specs/ (product
      capabilities and behaviors).
";

struct DiscoveredRepo {
    name: String,
    short_name: String,
    path: String,
}

struct DefaultTechSpecs {
    all_repos: Vec<String>,
    spex: Vec<String>,
}

pub fn run() -> Result<ExitCode> {
    let current_dir = std::env::current_dir()
        .change_context(InitError::Io)
        .attach("could not determine the current directory")?;

    let workspace_root = workspace_root(&current_dir)?;

    let default_name = dir_name(&workspace_root).ok_or_else(|| {
        Report::new(InitError::NoWorkspaceRoot).attach("could not read the workspace root name")
    })?;

    let project_name = confirm_project_name(default_name)?;

    // The spex repository directory follows the naming convention
    // (`<project-name>-spex`), so it can only be resolved once the project name
    // is known.
    let spex_dir = ensure_spex_dir(&current_dir, &workspace_root, &project_name)?;
    verify_not_initialized(&spex_dir)?;

    let spex_dir_name = dir_name(&spex_dir).unwrap_or_default().to_string();
    let repositories = discover_repositories(&workspace_root, &spex_dir_name, &project_name)?;
    let repositories = confirm_repositories(repositories)?;

    let tech_specs = default_tech_specs()?;
    write_config(&spex_dir, &project_name, &repositories, &tech_specs)?;
    write_code_workspace(&spex_dir, &project_name, &repositories)?;
    copy_templates(&spex_dir)?;

    println!(
        "spex repository initialized, open project in workspace and run agent skill `/spex-setup` to complete setup"
    );
    Ok(ExitCode::SUCCESS)
}

fn copy_templates(spex_dir: &Path) -> Result<()> {
    templates::scaffold(spex_dir).change_context(InitError::Io)?;
    templates::ensure_agents_md(spex_dir).change_context(InitError::Io)?;

    let biz_specs = spex_dir.join(BIZ_SPECS_DIR);
    fs::create_dir_all(&biz_specs)
        .change_context(InitError::Io)
        .attach_with(|| format!("could not create `{}`", biz_specs.display()))?;

    let plans = spex_dir.join(PLANS_DIR_NAME);
    fs::create_dir_all(&plans)
        .change_context(InitError::Io)
        .attach_with(|| format!("could not create `{}`", plans.display()))?;

    let contracts = spex_dir.join(CONTRACTS_DIR_NAME);
    fs::create_dir_all(&contracts)
        .change_context(InitError::Io)
        .attach_with(|| format!("could not create `{}`", contracts.display()))?;

    Ok(())
}

fn write_config(
    spex_dir: &Path,
    project_name: &str,
    repositories: &[DiscoveredRepo],
    tech_specs: &DefaultTechSpecs,
) -> Result<()> {
    let config_path = spex_dir.join(SPEX_CONFIG_FILE);
    fs::write(
        &config_path,
        config_to_yaml(project_name, repositories, tech_specs),
    )
    .change_context(InitError::Io)
    .attach_with(|| format!("could not write `{}`", config_path.display()))?;
    Ok(())
}

fn write_code_workspace(
    spex_dir: &Path,
    project_name: &str,
    repositories: &[DiscoveredRepo],
) -> Result<()> {
    let file_name = format!("{project_name}.{CODE_WORKSPACE_EXTENSION}");
    let path = spex_dir.join(&file_name);
    fs::write(&path, code_workspace_json(repositories))
        .change_context(InitError::Io)
        .attach_with(|| format!("could not write `{}`", path.display()))?;
    Ok(())
}

fn code_workspace_json(repositories: &[DiscoveredRepo]) -> String {
    let mut json = String::from("{\n  \"folders\": [\n");
    for repo in repositories {
        let name = workspace_folder_name(repo);
        json.push_str(&format!(
            "    {{ \"name\": \"{}\", \"path\": \"{}\" }},\n",
            escape_json(&name),
            escape_json(&repo.path),
        ));
    }
    json.push_str("  ]\n}\n");
    json
}

// Folders display the repository short name; spex owns its icon, and spex-setup
// fills the icons for the other folders.
fn workspace_folder_name(repo: &DiscoveredRepo) -> String {
    if repo.path == SPEX_REPO_PATH {
        return format!("{SPEX_WORKSPACE_ICON} {}", repo.short_name);
    }
    repo.short_name.clone()
}

fn escape_json(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn default_tech_specs() -> Result<DefaultTechSpecs> {
    let Some(dir) = templates::tech_specs_dir() else {
        return Ok(DefaultTechSpecs {
            all_repos: Vec::new(),
            spex: Vec::new(),
        });
    };

    let mut all_repos = Vec::new();
    let mut spex = Vec::new();
    for file in dir.files() {
        if file.path().extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }

        let name = file.path().to_string_lossy().into_owned();
        let raw = file
            .contents_utf8()
            .ok_or_else(|| Report::new(InitError::Template(name.clone())))?;
        let spec: TechSpec = serde_yaml_ng::from_str(raw)
            .change_context_lazy(|| InitError::Template(name.clone()))?;
        if spec.include_for_all_repos {
            all_repos.push(spec.id.clone());
        }
        if spec.include_for_spex.unwrap_or(spec.include_for_all_repos) {
            spex.push(spec.id);
        }
    }

    all_repos.sort();
    spex.sort();
    Ok(DefaultTechSpecs { all_repos, spex })
}

// The workspace root is the current directory, unless `spex init` runs from
// inside the spex repository directory, in which case it is the parent.
fn workspace_root(current_dir: &Path) -> Result<PathBuf> {
    if dir_name(current_dir).is_some_and(is_spex_dir_name) {
        let parent = current_dir.parent().ok_or_else(|| {
            Report::new(InitError::NoWorkspaceRoot)
                .attach("expected the spex directory to live inside a workspace root")
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
    if dir_name(current_dir).is_some_and(is_spex_dir_name) {
        return Ok(current_dir.to_path_buf());
    }

    let spex_dir = workspace_root.join(format!("{project_name}-{SPEX_DIR_NAME}"));
    if !spex_dir.is_dir() {
        fs::create_dir(&spex_dir)
            .change_context(InitError::Io)
            .attach_with(|| format!("could not create `{}` directory", spex_dir.display()))?;
    }
    Ok(spex_dir)
}

fn is_spex_dir_name(name: &str) -> bool {
    name == SPEX_DIR_NAME || name.ends_with(&format!("-{SPEX_DIR_NAME}"))
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

fn discover_repositories(
    workspace_root: &Path,
    spex_dir_name: &str,
    project_name: &str,
) -> Result<Vec<DiscoveredRepo>> {
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
        if dir_name.starts_with('.') {
            continue;
        }

        let path = if dir_name == spex_dir_name {
            SPEX_REPO_PATH.to_string()
        } else {
            format!("../{dir_name}")
        };

        let name = to_kebab_case(&dir_name);
        let short_name = repository_short_name(&name, project_name);
        repositories.push(DiscoveredRepo {
            name,
            short_name,
            path,
        });
    }

    repositories.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(repositories)
}

// The repository short name is the full repository name minus the
// `<project-name>-` prefix, per TECH-NAMING-CONVENTION.
fn repository_short_name(name: &str, project_name: &str) -> String {
    name.strip_prefix(&format!("{project_name}-"))
        .unwrap_or(name)
        .to_string()
}

fn confirm_repositories(repositories: Vec<DiscoveredRepo>) -> Result<Vec<DiscoveredRepo>> {
    if repositories.is_empty() {
        let answer = prompt("No workspace repositories found. Continue anyway? [Y/n]: ")?;
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
        "Workspace repositories: {names}\nUse these repositories? [Y/n]: "
    ))?;
    if is_yes(&answer) {
        Ok(repositories)
    } else {
        bail!(InitError::Cancelled);
    }
}

fn config_to_yaml(
    project_name: &str,
    repositories: &[DiscoveredRepo],
    tech_specs: &DefaultTechSpecs,
) -> String {
    let mut yaml = format!("project_name: {project_name}\ncli_version: {CLI_VERSION}\n");

    if repositories.is_empty() {
        yaml.push_str("repositories: []\n");
    } else {
        yaml.push_str("repositories:\n");
        for repo in repositories {
            let ids = if repo.path == SPEX_REPO_PATH {
                &tech_specs.spex
            } else {
                &tech_specs.all_repos
            };
            yaml.push_str(&format!("  - repository_name: {}\n", repo.name));
            yaml.push_str(&format!("    repository_short_name: {}\n", repo.short_name));
            if repo.path == SPEX_REPO_PATH {
                yaml.push_str(SPEX_REPO_DESCRIPTION);
            }
            yaml.push_str(&format!("    path: {}\n", repo.path));
            if ids.is_empty() {
                yaml.push_str("    tech_specs: []\n");
            } else {
                yaml.push_str("    tech_specs:\n");
                for id in ids {
                    yaml.push_str(&format!("      - {id}\n"));
                }
            }
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
