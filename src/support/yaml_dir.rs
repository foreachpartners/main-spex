use std::fs;
use std::path::{Path, PathBuf};

use eros::Context;
use serde::de::DeserializeOwned;
use walkdir::WalkDir;

use crate::error::{Result, could_not_parse, could_not_read, could_not_read_dir};

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
        let entry = entry.with_context(|| could_not_read_dir(dir))?;
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
            let raw = fs::read_to_string(&path).with_context(|| could_not_read(&path))?;
            let item = serde_yaml_ng::from_str(&raw).with_context(|| could_not_parse(&path))?;
            Ok(YamlFile { item, path, raw })
        })
        .collect()
}
