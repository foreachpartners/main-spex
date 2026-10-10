use std::fs;
use std::path::{Path, PathBuf};

use error_stack::ResultExt;
use serde::de::DeserializeOwned;
use walkdir::WalkDir;

use crate::error::{CliError, Result};

#[cfg(test)]
#[path = "yaml_dir_test.rs"]
mod yaml_dir_test;

pub struct YamlFile<T> {
    pub item: T,
    pub path: PathBuf,
    pub raw: String,
}

pub fn yaml_files(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(dir).sort_by_file_name() {
        let entry = entry.change_context_lazy(|| CliError::ReadDir(dir.to_path_buf()))?;
        let is_yaml = entry.path().extension().and_then(|ext| ext.to_str()) == Some("yaml");
        if is_yaml && entry.file_type().is_file() {
            files.push(entry.into_path());
        }
    }
    Ok(files)
}

pub fn load_yaml_dir<T: DeserializeOwned>(dir: &Path) -> Result<Vec<YamlFile<T>>> {
    yaml_files(dir)?
        .into_iter()
        .map(|path| {
            let raw = fs::read_to_string(&path)
                .change_context_lazy(|| CliError::ReadFile(path.clone()))?;
            let item = serde_yaml_ng::from_str(&raw)
                .change_context_lazy(|| CliError::ParseFile(path.clone()))?;
            Ok(YamlFile { item, path, raw })
        })
        .collect()
}
