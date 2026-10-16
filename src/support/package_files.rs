use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use globset::GlobSet;
use ignore::{Walk, WalkBuilder};

use super::glob::build_globset;
use crate::error;

#[cfg(test)]
#[path = "package_files_test.rs"]
mod package_files_test;

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

#[derive(Clone, Debug)]
pub struct Exclude {
    package_path: PathBuf,
    globs: Option<GlobSet>,
}

impl Exclude {
    pub fn new(package_path: &Path, patterns: &[String]) -> error::Result<Self> {
        Ok(Self {
            package_path: package_path.to_path_buf(),
            globs: build_globset(patterns)?,
        })
    }

    pub fn matches(&self, path: &Path) -> bool {
        let relative = path.strip_prefix(&self.package_path).unwrap_or(path);
        self.globs
            .as_ref()
            .is_some_and(|globs| globs.is_match(relative))
    }
}

pub struct PackageFiles(Walk);

impl PackageFiles {
    pub fn new(root: &Path, exclude: &Exclude) -> Self {
        let exclude = exclude.clone();
        Self(
            WalkBuilder::new(root)
                .hidden(false)
                .git_ignore(true)
                .git_global(true)
                .require_git(false)
                .sort_by_file_name(|left, right| left.cmp(right))
                .filter_entry(move |entry| {
                    !is_skipped(entry.file_name()) && !exclude.matches(entry.path())
                })
                .build(),
        )
    }
}

impl Iterator for PackageFiles {
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
