use std::fs;
use std::path::Path;

use error_stack::{Report, ResultExt};
use ignore::{Walk, WalkBuilder};
use serde::de::DeserializeOwned;

pub fn plural(count: usize, noun: &str) -> String {
    if count == 1 {
        format!("1 {noun}")
    } else {
        format!("{count} {noun}s")
    }
}

// Walks a workspace repository's files, honoring gitignore rules while still
// surfacing dotfiles (e.g. `.env`) and always skipping the `.git` directory.
pub fn repo_walker(root: &Path) -> Walk {
    WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .filter_entry(|entry| entry.file_name() != std::ffi::OsStr::new(".git"))
        .build()
}

// Parses every `*.yaml` file in `dir` into `T`. Missing directories yield an
// empty list; the closures let each caller surface its own error context while
// the directory walk and per-file read/parse logic stays in one place.
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

    let entries = fs::read_dir(dir)
        .change_context_lazy(&read_dir_err)
        .attach_with(|| format!("could not read `{}`", dir.display()))?;

    for entry in entries {
        let entry = entry.change_context_lazy(&read_dir_err)?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }

        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let raw =
            fs::read_to_string(&path).change_context_lazy(|| read_file_err(file_name.clone()))?;
        let item =
            serde_yaml_ng::from_str(&raw).change_context_lazy(|| parse_err(file_name.clone()))?;
        items.push(item);
    }

    Ok(items)
}
