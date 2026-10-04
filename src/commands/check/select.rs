use std::path::Path;

use error_stack::Report;

use crate::config::{Config, Repository};
use crate::error::{CliError, Result};

use super::ScopedFile;

#[cfg(test)]
#[path = "select_test.rs"]
mod select_test;

// Selects the repositories to check, preserving `spex.yaml` order. Without
// `--files`, that is the `--repo` names (or every repository). With `--files`,
// it is every repository that owns at least one provided file, unioned with any
// explicitly named `--repo`, so a `per-repo` script runs only where the files
// live (plus any repository the user named outright).
pub(super) fn select_repos<'a>(
    config: &'a Config,
    spex_dir: &Path,
    names: &[String],
    files: &[String],
) -> Result<Vec<&'a Repository>> {
    for name in names {
        if !config
            .repositories
            .iter()
            .any(|repo| repo.matches_any_repo_name(name))
        {
            return Err(Report::new(CliError::UnknownRepo(name.clone())));
        }
    }

    for file in files {
        let found = config.repositories.iter().any(|repo| {
            !ScopedFile::provided(&spex_dir.join(&repo.path), std::slice::from_ref(file)).is_empty()
        });
        if !found {
            return Err(Report::new(CliError::Custom(format!(
                "unknown file `{file}` — not found in any workspace repository"
            ))));
        }
    }

    if files.is_empty() {
        return Ok(config
            .repositories
            .iter()
            .filter(|repo| {
                names.is_empty() || names.iter().any(|name| repo.matches_any_repo_name(name))
            })
            .collect());
    }

    Ok(config
        .repositories
        .iter()
        .filter(|repo| {
            names.iter().any(|name| repo.matches_any_repo_name(name))
                || !ScopedFile::provided(&spex_dir.join(&repo.path), files).is_empty()
        })
        .collect())
}
