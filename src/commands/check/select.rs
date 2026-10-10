use error_stack::Report;

use crate::config::Repository;
use crate::config::spex::Spex;
use crate::error::{CliError, Result};

use super::scoped_file::ScopedFile;

#[cfg(test)]
#[path = "select_test.rs"]
mod select_test;

pub(super) fn select_repos<'a>(
    spex: &'a Spex,
    names: &[String],
    files: &[String],
) -> Result<Vec<&'a Repository>> {
    if files.is_empty() {
        return spex.select_repositories(names);
    }

    spex.validate_repository_names(names)?;

    for file in files {
        let found = spex
            .repositories
            .iter()
            .any(|repo| !ScopedFile::provided(&repo.path, std::slice::from_ref(file)).is_empty());
        if !found {
            return Err(Report::new(CliError::Custom(format!(
                "unknown file `{file}` — not found in any workspace repository"
            ))));
        }
    }

    Ok(spex
        .repositories
        .iter()
        .filter(|repo| {
            names.iter().any(|name| repo.matches_any_repo_name(name))
                || !ScopedFile::provided(&repo.path, files).is_empty()
        })
        .collect())
}
