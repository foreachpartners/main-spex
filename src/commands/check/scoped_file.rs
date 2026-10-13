use std::path::{Path, PathBuf};

use eros::Context;

use crate::error::Result;
use crate::support::fs::relative_to;
use crate::support::package_files::PackageFiles;

#[cfg(test)]
#[path = "scoped_file_test.rs"]
mod scoped_file_test;

pub(super) struct ScopedFile {
    pub(super) abs: PathBuf,
    pub(super) rel: PathBuf,
}

impl ScopedFile {
    pub(super) fn collect(package_path: &Path) -> Result<Vec<Self>> {
        let mut files = Vec::new();
        for result in PackageFiles::new(package_path) {
            let abs = result.context("could not walk package files")?;
            let rel = relative_to(&abs, package_path).to_path_buf();
            files.push(Self { abs, rel });
        }
        Ok(files)
    }

    pub(super) fn provided(package_path: &Path, files: &[String]) -> Vec<Self> {
        files
            .iter()
            .filter_map(|file| {
                let rel = PathBuf::from(file.trim_start_matches("./"));
                let abs = package_path.join(&rel);
                abs.is_file().then_some(Self { abs, rel })
            })
            .collect()
    }
}
