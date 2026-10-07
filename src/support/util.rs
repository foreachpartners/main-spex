use std::fs;
use std::path::{Path, PathBuf};

use error_stack::ResultExt;
use serde::de::DeserializeOwned;

use crate::error::{CliError, Result};

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

pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

pub struct YamlFile<T> {
    pub item: T,
    pub rel_path: String,
    pub raw: String,
}

pub fn yaml_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if dir.is_dir() {
        collect_yaml_files(dir, &mut files)?;
    }
    files.sort();
    Ok(files)
}

fn collect_yaml_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let entries = fs::read_dir(dir).change_context_lazy(|| CliError::ReadDir(dir.to_path_buf()))?;

    for entry in entries {
        let entry = entry.change_context_lazy(|| CliError::ReadDir(dir.to_path_buf()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_yaml_files(&path, files)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("yaml") {
            files.push(path);
        }
    }

    Ok(())
}

pub fn load_yaml_dir<T: DeserializeOwned>(dir: &Path) -> Result<Vec<YamlFile<T>>> {
    yaml_files(dir)?
        .into_iter()
        .map(|path| {
            let raw = fs::read_to_string(&path)
                .change_context_lazy(|| CliError::ReadFile(path.clone()))?;
            let item = serde_yaml_ng::from_str(&raw)
                .change_context_lazy(|| CliError::ParseFile(path.clone()))?;
            let rel_path = path
                .strip_prefix(dir)
                .unwrap_or(&path)
                .to_string_lossy()
                .into_owned();
            Ok(YamlFile {
                item,
                rel_path,
                raw,
            })
        })
        .collect()
}

// The 1-indexed source line of each `acceptance_criteria` list item (`- ...`),
// in appearance order — the criterion's own line for a rule's plain-string
// items, or its `- id: ...` line for a spec's object items. Serde doesn't
// track source spans, so callers zip this against the parsed list by index.
pub fn acceptance_criteria_lines(text: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    let mut in_block = false;
    for (index, line) in text.lines().enumerate() {
        if line.starts_with("acceptance_criteria:") {
            in_block = true;
            continue;
        }
        if !in_block {
            continue;
        }
        if !line.starts_with(' ') && !line.starts_with('\t') {
            in_block = false;
            continue;
        }
        if line.trim_start().starts_with("- ") {
            lines.push(index + 1);
        }
    }
    lines
}

pub fn number_criteria<A>(criteria: &mut [A], raw: &str, set_line: impl Fn(&mut A, usize)) {
    for (criterion, line) in criteria.iter_mut().zip(acceptance_criteria_lines(raw)) {
        set_line(criterion, line);
    }
}
