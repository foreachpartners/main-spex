use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use error_stack::{Report, ResultExt};
use serde::{Deserialize, Serialize};

use crate::error::{CliError, Result};
use crate::rule::RuleId;
use crate::support::fs::dir_name;

#[cfg(test)]
#[path = "spex_config_test.rs"]
mod spex_config_test;

pub const SPEX_CONFIG_FILE: &str = "spex.yaml";
pub const SPEX_DIR_NAME: &str = "spex";
pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize, Serialize)]
pub struct SpexConfig {
    pub project_name: String,
    pub cli_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub rules_unused: Vec<RuleId>,
    #[serde(default)]
    pub repositories: Vec<RepositoryConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RepositoryConfig {
    pub repository_name: String,
    pub repository_short_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub path: String,
    #[serde(default)]
    pub contracts: Vec<ContractConfig>,
    #[serde(default)]
    pub rules: Vec<RuleId>,
    #[serde(default)]
    pub ignored_errors: Vec<PreexistingError>,
}

impl RepositoryConfig {
    pub fn matches_any_repo_name(&self, name: &str) -> bool {
        self.repository_name == name || self.repository_short_name == name
    }
}

// A repository's interface contract, stored inside the spex repository's
// `contracts/` folder. `path` is relative to the spex directory.
#[derive(Debug, Deserialize, Serialize)]
pub struct ContractConfig {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// A baseline entry that suppresses pre-existing `error`-severity findings for an
// inherited or additional check identified by `rule_id` + `check`. An empty `files`
// suppresses the whole check; otherwise only findings whose location file is
// listed are suppressed, so new errors in other files still surface.
#[derive(Debug, Deserialize, Serialize)]
pub struct PreexistingError {
    pub rule_id: String,
    pub check: String,
    #[serde(default)]
    pub files: Vec<String>,
}

// A fully-resolved baseline entry to write into `spex.yaml`. An empty `files`
// renders a bare `rule_id`/`check` entry that suppresses the whole check.
pub struct PreexistingEntry {
    pub rule_id: String,
    pub check: String,
    pub files: Vec<String>,
}

impl SpexConfig {
    // Locates `spex.yaml`. Walks up from `start`; at each ancestor it uses a
    // directly present `spex.yaml`, then falls back to scanning that ancestor's
    // immediate child directories. The child scan lets discovery work from the
    // workspace root or a sibling repository even though the spex repository
    // directory is named `<project-name>-spex`.
    pub fn find_file(start: &Path) -> Result<PathBuf> {
        for dir in start.ancestors() {
            let config_path = dir.join(SPEX_CONFIG_FILE);
            if config_path.is_file() {
                return Ok(config_path);
            }
            if let Some(child_config_path) = Self::spex_child_config_path(dir) {
                return Ok(child_config_path);
            }
        }
        Err(Report::new(CliError::ConfigNotFound))
    }

    pub fn dir_of(config_path: &Path) -> &Path {
        config_path.parent().unwrap_or(Path::new(""))
    }

    pub fn is_spex_dir_name(name: &str) -> bool {
        name == SPEX_DIR_NAME || name.ends_with(&format!("-{SPEX_DIR_NAME}"))
    }

    fn spex_child_config_path(dir: &Path) -> Option<PathBuf> {
        let mut config_paths: Vec<PathBuf> = fs::read_dir(dir)
            .ok()?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| dir_name(path).is_some_and(Self::is_spex_dir_name))
            .map(|path| path.join(SPEX_CONFIG_FILE))
            .filter(|config_path| config_path.is_file())
            .collect();
        config_paths.sort();
        config_paths.into_iter().next()
    }

    pub fn load(config_path: &Path) -> Result<Self> {
        if !config_path.exists() {
            return Err(Report::new(CliError::ConfigNotFound));
        }

        let raw = fs::read_to_string(config_path)
            .change_context_lazy(|| CliError::ReadFile(config_path.to_path_buf()))?;

        serde_yaml_ng::from_str(&raw)
            .change_context_lazy(|| CliError::ParseFile(config_path.to_path_buf()))
    }

    // Warns when the version that wrote `spex.yaml` differs from the running CLI so
    // the user knows to run `spex update` and resync the bundled templates.
    pub fn version_warning(&self) -> Option<String> {
        if self.cli_version == CLI_VERSION {
            return None;
        }
        Some(format!(
            "`{SPEX_CONFIG_FILE}` was written by spex {}, but the installed CLI is {CLI_VERSION} — run `spex update` to sync",
            self.cli_version
        ))
    }

    // Rewrites each in-scope repository's `ignored_errors` block in raw
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
                        pending = Some(Self::render_preexisting_block(entries));
                    }
                }
                out.push_str(line);
                out.push('\n');
                i += 1;
                continue;
            }

            // Drop the existing block (its key line plus every more-indented child)
            // for in-scope repositories; the regenerated block replaces it.
            if in_scope && indent == 4 && trimmed.starts_with("ignored_errors:") {
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

    fn render_preexisting_block(entries: &[PreexistingEntry]) -> String {
        let mut block = String::from("    ignored_errors:\n");
        for entry in entries {
            block.push_str(&format!("      - rule_id: {}\n", entry.rule_id));
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
}

fn indent_of(line: &str) -> usize {
    line.chars().take_while(|ch| *ch == ' ').count()
}
