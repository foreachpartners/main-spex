use std::path::{Path, PathBuf};

use error_stack::ResultExt;

use crate::error::{CliError, Result};
use crate::support::fs::relative_to;
use crate::support::repo_files::RepoFiles;

pub(super) struct ScopedFile {
    pub(super) abs: PathBuf,
    pub(super) rel: PathBuf,
}

impl ScopedFile {
    pub(super) fn collect(repo_path: &Path) -> Result<Vec<Self>> {
        let mut files = Vec::new();
        for result in RepoFiles::new(repo_path) {
            let abs = result.change_context_lazy(|| {
                CliError::Internal("could not walk workspace repository files".into())
            })?;
            let rel = relative_to(&abs, repo_path).to_path_buf();
            files.push(Self { abs, rel });
        }
        Ok(files)
    }

    pub(super) fn provided(repo_path: &Path, files: &[String]) -> Vec<Self> {
        files
            .iter()
            .filter_map(|file| {
                let rel = PathBuf::from(file.trim_start_matches("./"));
                let abs = repo_path.join(&rel);
                abs.is_file().then_some(Self { abs, rel })
            })
            .collect()
    }
}
