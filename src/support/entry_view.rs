use std::path::Path;

use eros::{Context, IntoAnyUnion, bail};
use serde_json::{Map, Value};

use super::fs::write_file;
use super::yaml_dir::YamlFile;
use crate::error::Result;
use crate::templates::{BUNDLED_TEMPLATES_DIR, SCHEMAS_DIRNAME};

#[cfg(test)]
#[path = "entry_view_test.rs"]
mod entry_view_test;

pub const PATH_FIELD: &str = "path";
const EMPTY_LIST: &str = "[]\n";
const NOTHING_FOUND: &str = "nothing found";

pub fn with_path(file: &YamlFile<Value>) -> Value {
    let Some(object) = file.item.as_object() else {
        return file.item.clone();
    };
    let path = Value::String(file.path.to_string_lossy().into_owned());
    let mut view = Map::with_capacity(object.len() + 1);
    for (key, value) in object {
        view.insert(key.clone(), value.clone());
        if key == "id" {
            view.insert(PATH_FIELD.to_string(), path.clone());
        }
    }
    Value::Object(view)
}

pub fn entries_to_yaml(entries: &[Value], what: &str) -> Result<String> {
    if entries.is_empty() {
        return Ok(EMPTY_LIST.to_string());
    }
    entries
        .iter()
        .map(|entry| {
            serde_yaml_ng::to_string(&[entry])
                .with_context(|| format!("could not serialize {what}"))
                .any_union()
        })
        .collect::<Result<Vec<_>>>()
        .map(|documents| documents.join("\n"))
}

pub fn print_entries(yaml: &str, output: Option<&str>) -> Result<()> {
    if yaml == EMPTY_LIST {
        eprintln!("{NOTHING_FOUND}");
    }
    match output {
        Some(path) => write_file(Path::new(path), yaml),
        None => {
            print!("{yaml}");
            Ok(())
        }
    }
}

pub fn keep_fields(view: Value, fields: &[String]) -> Value {
    match view {
        Value::Object(object) if !fields.is_empty() => Value::Object(
            object
                .into_iter()
                .filter(|(key, _)| fields.contains(key))
                .collect(),
        ),
        other => other,
    }
}

pub fn ensure_schema_fields(fields: &[String], kind: &str, schema_filename: &str) -> Result<()> {
    if fields.is_empty() {
        return Ok(());
    }
    let schema_path = format!("{SCHEMAS_DIRNAME}/{schema_filename}");
    let schema: Value = serde_json::from_str(
        BUNDLED_TEMPLATES_DIR
            .get_file(&schema_path)
            .and_then(|file| file.contents_utf8())
            .expect("bundled schema is missing or not UTF-8"),
    )
    .expect("bundled schema is not JSON");
    let mut known: Vec<&str> = schema["properties"]
        .as_object()
        .expect("bundled schema has no properties")
        .keys()
        .map(String::as_str)
        .collect();
    known.push(PATH_FIELD);
    ensure_known_fields(fields, kind, &known)
}

pub fn ensure_known_fields(fields: &[String], kind: &str, known: &[&str]) -> Result<()> {
    for field in fields {
        if !known.contains(&field.as_str()) {
            bail!(
                "unknown field `{field}` — a {kind} has: {}",
                known.join(", ")
            );
        }
    }
    Ok(())
}

pub struct Keywords(Vec<String>);

impl Keywords {
    pub fn new(keywords: &[String]) -> Self {
        Self(
            keywords
                .iter()
                .map(|keyword| keyword.to_lowercase())
                .collect(),
        )
    }

    pub fn matches(&self, value: &Value) -> bool {
        self.0.is_empty() || self.0.iter().any(|keyword| contains_text(value, keyword))
    }
}

fn contains_text(value: &Value, lowercase_keyword: &str) -> bool {
    match value {
        Value::String(text) => text.to_lowercase().contains(lowercase_keyword),
        Value::Array(items) => items
            .iter()
            .any(|item| contains_text(item, lowercase_keyword)),
        Value::Object(object) => object
            .values()
            .any(|item| contains_text(item, lowercase_keyword)),
        _ => false,
    }
}
