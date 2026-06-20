use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use error_stack::{Report, ResultExt};
use serde::Deserialize;
use thiserror::Error;

use crate::tech_spec::{CheckOverride, RepoCheck};

#[cfg(test)]
#[path = "config_test.rs"]
mod config_test;

pub const SPEX_CONFIG_FILE: &str = "spex.yaml";
pub const SPEX_DIR_NAME: &str = "spex";
pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

type Result<T> = core::result::Result<T, Report<ConfigError>>;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(
        "`{SPEX_CONFIG_FILE}` not found — run `spex` from the workspace root or any workspace repository"
    )]
    NotFound,
    #[error("could not read `{SPEX_CONFIG_FILE}`")]
    Read,
    #[error("could not parse `{SPEX_CONFIG_FILE}`")]
    Parse,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub project_name: String,
    pub cli_version: String,
    #[serde(default)]
    pub repositories: Vec<Repository>,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    #[serde(rename = "repository_name")]
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub contracts: Vec<Contract>,
    #[serde(default)]
    pub tech_specs: Vec<String>,
    #[serde(default)]
    pub checks_additional: Vec<RepoCheck>,
    #[serde(default)]
    pub checks_override: Vec<CheckOverride>,
    #[serde(default)]
    pub checks_preexisting_errors: Vec<PreexistingError>,
}

// A repository's interface contract, stored inside the spex repository's
// `contracts/` folder. `path` is relative to the spex directory.
#[derive(Debug, Deserialize)]
pub struct Contract {
    pub path: String,
}

// A baseline entry that suppresses pre-existing `error`-severity findings for an
// inherited or additional check identified by `spec` + `check`. An empty `files`
// suppresses the whole check; otherwise only findings whose location file is
// listed are suppressed, so new errors in other files still surface.
#[derive(Debug, Deserialize)]
pub struct PreexistingError {
    pub spec: String,
    pub check: String,
    #[serde(default)]
    pub files: Vec<String>,
}

pub fn find_dir(start: &Path) -> Result<PathBuf> {
    for dir in start.ancestors() {
        if dir.join(SPEX_CONFIG_FILE).is_file() {
            return Ok(dir.to_path_buf());
        }
        let nested = dir.join(SPEX_DIR_NAME);
        if nested.join(SPEX_CONFIG_FILE).is_file() {
            return Ok(nested);
        }
    }
    Err(Report::new(ConfigError::NotFound))
}

pub fn load(spex_dir: &Path) -> Result<Config> {
    let config_path = spex_dir.join(SPEX_CONFIG_FILE);
    if !config_path.exists() {
        return Err(Report::new(ConfigError::NotFound));
    }

    let raw = fs::read_to_string(&config_path)
        .change_context(ConfigError::Read)
        .attach_with(|| format!("could not read `{}`", config_path.display()))?;

    serde_yaml_ng::from_str(&raw).change_context(ConfigError::Parse)
}

// Warns when the version that wrote `spex.yaml` differs from the running CLI so
// the user knows to run `spex update` and resync the bundled templates.
pub fn version_warning(config: &Config) -> Option<String> {
    if config.cli_version == CLI_VERSION {
        return None;
    }
    Some(format!(
        "`{SPEX_CONFIG_FILE}` was written by spex {}, but the installed CLI is {CLI_VERSION} — run `spex update` to sync",
        config.cli_version
    ))
}

// A fully-resolved baseline entry to write into `spex.yaml`. An empty `files`
// renders a bare `spec`/`check` entry that suppresses the whole check.
pub struct PreexistingEntry {
    pub spec: String,
    pub check: String,
    pub files: Vec<String>,
}

// Rewrites each in-scope repository's `checks_preexisting_errors` block in raw
// `spex.yaml` text, leaving the rest of the file byte-stable. `by_repo` is keyed
// on `repository_name` and holds every repository whose block should be
// regenerated (an empty entry list removes a now-stale block); repositories
// absent from the map are left untouched. The regenerated block is appended to
// the end of the repository's section.
pub fn set_preexisting_errors(
    raw: &str,
    by_repo: &HashMap<String, Vec<PreexistingEntry>>,
) -> String {
    let lines: Vec<&str> = raw.lines().collect();
    let mut out = String::with_capacity(raw.len());
    let mut pending: Option<String> = None;
    let mut in_scope = false;

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let indent = indent_of(line);
        let trimmed = line.trim_start();

        let repo_start = indent == 2 && trimmed.starts_with("- repository_name:");
        let boundary = repo_start || (indent == 0 && !trimmed.is_empty());

        // A repository start or any top-level key closes the previous section,
        // so flush its regenerated block before emitting the boundary line.
        if boundary {
            if let Some(block) = pending.take() {
                out.push_str(&block);
            }
            in_scope = false;
        }

        if repo_start {
            let name = trimmed["- repository_name:".len()..].trim();
            if let Some(entries) = by_repo.get(name) {
                in_scope = true;
                if !entries.is_empty() {
                    pending = Some(render_preexisting_block(entries));
                }
            }
            out.push_str(line);
            out.push('\n');
            i += 1;
            continue;
        }

        // Drop the existing block (its key line plus every more-indented child)
        // for in-scope repositories; the regenerated block replaces it.
        if in_scope && indent == 4 && trimmed.starts_with("checks_preexisting_errors:") {
            i += 1;
            while i < lines.len() && indent_of(lines[i]) > 4 {
                i += 1;
            }
            continue;
        }

        out.push_str(line);
        out.push('\n');
        i += 1;
    }

    if let Some(block) = pending.take() {
        out.push_str(&block);
    }
    out
}

fn indent_of(line: &str) -> usize {
    line.chars().take_while(|ch| *ch == ' ').count()
}

fn render_preexisting_block(entries: &[PreexistingEntry]) -> String {
    let mut block = String::from("    checks_preexisting_errors:\n");
    for entry in entries {
        block.push_str(&format!("      - spec: {}\n", entry.spec));
        block.push_str(&format!("        check: {}\n", entry.check));
        if !entry.files.is_empty() {
            block.push_str("        files:\n");
            for file in &entry.files {
                block.push_str(&format!("          - {file}\n"));
            }
        }
    }
    block
}

// Rewrites the `cli_version` line in raw `spex.yaml` text, preserving comments
// and field order. Inserts it after `project_name` when absent.
pub fn set_cli_version(raw: &str, version: &str) -> String {
    let new_line = format!("cli_version: {version}");

    if raw.lines().any(|line| line.starts_with("cli_version:")) {
        let mut out = String::with_capacity(raw.len());
        for line in raw.lines() {
            if line.starts_with("cli_version:") {
                out.push_str(&new_line);
            } else {
                out.push_str(line);
            }
            out.push('\n');
        }
        return out;
    }

    let mut out = String::with_capacity(raw.len() + new_line.len() + 1);
    let mut inserted = false;
    for line in raw.lines() {
        out.push_str(line);
        out.push('\n');
        if !inserted && line.starts_with("project_name:") {
            out.push_str(&new_line);
            out.push('\n');
            inserted = true;
        }
    }
    if !inserted {
        out.insert_str(0, &format!("{new_line}\n"));
    }
    out
}
