use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use error_stack::{Report, ResultExt};
use ignore::{Walk, WalkBuilder};
use serde::de::DeserializeOwned;

#[cfg(test)]
#[path = "util_test.rs"]
mod util_test;

pub fn plural(count: usize, noun: &str) -> String {
    if count == 1 {
        format!("1 {noun}")
    } else {
        format!("{count} {noun}s")
    }
}

// Dependency lock files are generated artifacts: never spec-annotated and
// irrelevant to checks, so the walker skips them in every repository.
const LOCK_FILES: &[&str] = &[
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

// Walks a workspace repository's files, honoring gitignore rules while still
// surfacing dotfiles (e.g. `.env`) and always skipping the `.git` directory and
// well-known dependency lock files (e.g. `Cargo.lock`, `package-lock.json`).
// `require_git(false)` keeps gitignore rules in effect even when the repository
// has no `.git` directory; otherwise the `ignore` crate silently ignores them.
pub fn repo_walker(root: &Path) -> Walk {
    WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .require_git(false)
        .filter_entry(|entry| !is_skipped(entry.file_name()))
        .build()
}

fn is_skipped(name: &OsStr) -> bool {
    name == OsStr::new(".git") || LOCK_FILES.iter().any(|lock| name == OsStr::new(lock))
}

// A rules/specs directory may group files one level deep (e.g.
// `rules/frontend/frontend-testing.yaml`); deeper nesting is not walked.
const MAX_SUBDIR_DEPTH: usize = 1;

fn collect_yaml_paths<E>(
    dir: &Path,
    depth: usize,
    read_dir_err: &impl Fn() -> E,
    paths: &mut Vec<PathBuf>,
) -> Result<(), Report<E>>
where
    E: std::error::Error + Send + Sync + 'static,
{
    let entries = fs::read_dir(dir)
        .change_context_lazy(read_dir_err)
        .attach_with(|| format!("could not read `{}`", dir.display()))?;

    for entry in entries {
        let entry = entry.change_context_lazy(read_dir_err)?;
        let path = entry.path();
        if path.is_dir() {
            if depth < MAX_SUBDIR_DEPTH {
                collect_yaml_paths(&path, depth + 1, read_dir_err, paths)?;
            }
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("yaml") {
            paths.push(path);
        }
    }

    Ok(())
}

// Lists every `*.yaml` file directly under `dir`, or one subdirectory level
// down, sorted for stable output. A missing or unreadable directory (or
// subdirectory) yields an empty list.
pub fn yaml_files(dir: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.filter_map(|entry| entry.ok()) {
            let path = entry.path();
            if path.is_dir() {
                if depth < MAX_SUBDIR_DEPTH {
                    walk(&path, depth + 1, out);
                }
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) == Some("yaml") {
                out.push(path);
            }
        }
    }

    let mut files = Vec::new();
    walk(dir, 0, &mut files);
    files.sort();
    files
}

// Parses every `*.yaml` file in `dir`, or one subdirectory level down, into
// `T`. Missing directories yield an empty list; the closures let each caller
// surface its own error context while the directory walk and per-file
// read/parse logic stays in one place.
pub fn load_yaml_dir<T, E>(
    dir: &Path,
    read_dir_err: impl Fn() -> E,
    read_file_err: impl Fn(String) -> E,
    parse_err: impl Fn(String) -> E,
) -> Result<Vec<T>, Report<E>>
where
    T: DeserializeOwned,
    E: std::error::Error + Send + Sync + 'static,
{
    let mut items = Vec::new();
    if !dir.is_dir() {
        return Ok(items);
    }

    let mut paths = Vec::new();
    collect_yaml_paths(dir, 0, &read_dir_err, &mut paths)?;
    paths.sort();

    for path in paths {
        let rel_name = path
            .strip_prefix(dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .into_owned();
        let raw =
            fs::read_to_string(&path).change_context_lazy(|| read_file_err(rel_name.clone()))?;
        let item =
            serde_yaml_ng::from_str(&raw).change_context_lazy(|| parse_err(rel_name.clone()))?;
        items.push(item);
    }

    Ok(items)
}
