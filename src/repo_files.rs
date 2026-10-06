use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use ignore::{Walk, WalkBuilder};

#[cfg(test)]
#[path = "repo_files_test.rs"]
mod repo_files_test;

const SKIPPED_NAMES: &[&str] = &[
    ".git",
    "Cargo.lock",
    "Gemfile.lock",
    "Pipfile.lock",
    "bun.lock",
    "bun.lockb",
    "composer.lock",
    "flake.lock",
    "go.sum",
    "package-lock.json",
    "pnpm-lock.yaml",
    "poetry.lock",
    "uv.lock",
    "yarn.lock",
];

pub struct RepoFiles(Walk);

impl RepoFiles {
    pub fn new(root: &Path) -> Self {
        Self(
            WalkBuilder::new(root)
                .hidden(false)
                .git_ignore(true)
                .git_global(true)
                .require_git(false)
                .sort_by_file_name(|left, right| left.cmp(right))
                .filter_entry(|entry| !is_skipped(entry.file_name()))
                .build(),
        )
    }
}

impl Iterator for RepoFiles {
    type Item = Result<PathBuf, ignore::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.next()? {
                Ok(entry) if entry.file_type().is_some_and(|kind| kind.is_file()) => {
                    return Some(Ok(entry.into_path()));
                }
                Ok(_) => continue,
                Err(error) => return Some(Err(error)),
            }
        }
    }
}

fn is_skipped(name: &OsStr) -> bool {
    SKIPPED_NAMES.iter().any(|skipped| name == *skipped)
}
