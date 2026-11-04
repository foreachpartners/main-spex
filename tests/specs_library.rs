use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;
use serde_json::Value;
use tempfile::TempDir;

#[derive(Deserialize)]
struct Entry {
    id: String,
}

const LIBRARY_DIR: &str = "templates/specs";
const SCHEMAS_DIR: &str = "templates/schemas";
const SPEC_SCHEMA_FILE: &str = "spec.schema.json";
const PROJECT_FIELDS: [&str; 3] = ["packages", "contract", "status"];

fn repository() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn schemas_without_project_fields() -> TempDir {
    let dir = tempfile::tempdir().expect("temp dir");
    for schema in fs::read_dir(repository().join(SCHEMAS_DIR)).expect("schemas folder") {
        let path = schema.expect("schema").path();
        let name = path.file_name().expect("schema name");
        fs::copy(&path, dir.path().join(name)).expect("copy schema");
    }

    let spec_schema = dir.path().join(SPEC_SCHEMA_FILE);
    let text = fs::read_to_string(&spec_schema).expect("read spec schema");
    let mut schema: Value = serde_json::from_str(&text).expect("parse spec schema");
    let properties = schema["properties"].as_object_mut().expect("properties");
    for field in PROJECT_FIELDS {
        properties.remove(field);
    }
    let required = schema["required"].as_array_mut().expect("required");
    required.retain(|field| {
        !field
            .as_str()
            .is_some_and(|name| PROJECT_FIELDS.contains(&name))
    });
    fs::write(&spec_schema, schema.to_string()).expect("write spec schema");
    dir
}

fn entry_files() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(repository().join(LIBRARY_DIR))
        .expect("specs library folder")
        .map(|entry| entry.expect("specs library entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "yaml")
        })
        .collect();
    files.sort();
    assert!(!files.is_empty(), "specs library has no entries");
    files
}

fn read_entry(file: &Path) -> Entry {
    let text = fs::read_to_string(file).expect("read entry");
    serde_yaml_ng::from_str(&text).unwrap_or_else(|error| panic!("{}: {error}", file.display()))
}

#[test]
fn specs_library_entries_validate_against_the_spec_schema_without_project_fields() {
    let files = entry_files();
    let schemas = schemas_without_project_fields();

    let output = Command::new(env!("CARGO_BIN_EXE_spex"))
        .args(["check-schema", "--schema"])
        .arg(schemas.path().join(SPEC_SCHEMA_FILE))
        .arg("--files")
        .args(&files)
        .output()
        .expect("run spex");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn specs_library_entries_have_unique_ids() {
    let mut files_by_id: HashMap<String, PathBuf> = HashMap::new();
    for file in entry_files() {
        let id = read_entry(&file).id;

        if let Some(first) = files_by_id.insert(id.clone(), file.clone()) {
            panic!(
                "id `{id}` is in both {} and {}",
                first.display(),
                file.display()
            );
        }
    }
}

#[test]
fn specs_library_entries_are_named_by_id() {
    for file in entry_files() {
        let entry = read_entry(&file);

        let file_stem = file.file_stem().and_then(|stem| stem.to_str());
        assert_eq!(file_stem, Some(entry.id.as_str()), "{}", file.display());
    }
}
