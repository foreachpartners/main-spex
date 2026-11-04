use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

#[derive(Deserialize)]
struct Entry {
    id: String,
    #[serde(default)]
    checks: Vec<Check>,
}

#[derive(Deserialize)]
struct Check {
    #[serde(default)]
    script: Option<String>,
}

const LIBRARY_DIR: &str = "templates/rules";
const RULE_SCHEMA_FILE: &str = "templates/schemas/rule.schema.json";

fn repository() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn entry_files() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(repository().join(LIBRARY_DIR))
        .expect("rules library folder")
        .map(|entry| entry.expect("rules library entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "yaml")
        })
        .collect();
    files.sort();
    assert!(!files.is_empty(), "rules library has no entries");
    files
}

fn read_entry(file: &Path) -> Entry {
    let text = fs::read_to_string(file).expect("read entry");
    serde_yaml_ng::from_str(&text).unwrap_or_else(|error| panic!("{}: {error}", file.display()))
}

#[test]
fn rules_library_entries_validate_against_the_rule_schema() {
    let output = Command::new(env!("CARGO_BIN_EXE_spex"))
        .args(["check-schema", "--schema"])
        .arg(repository().join(RULE_SCHEMA_FILE))
        .arg("--files")
        .args(entry_files())
        .output()
        .expect("run spex");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn rules_library_entries_have_unique_ids() {
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
fn rules_library_entries_are_named_by_id() {
    for file in entry_files() {
        let entry = read_entry(&file);

        let file_stem = file.file_stem().and_then(|stem| stem.to_str());
        assert_eq!(file_stem, Some(entry.id.as_str()), "{}", file.display());
    }
}

#[test]
fn rules_library_entries_set_sync_with_upstream_true_right_after_their_title() {
    for file in entry_files() {
        let text = fs::read_to_string(&file).expect("read entry");
        let lines: Vec<&str> = text.lines().take(3).collect();

        assert!(
            lines[1].starts_with("title: ") && lines[2] == "sync_with_upstream: true",
            "{} must set sync_with_upstream: true right after its title",
            file.display()
        );
    }
}

#[test]
fn rules_library_entries_have_no_leftover_template_only_fields() {
    for file in entry_files() {
        let text = fs::read_to_string(&file).expect("read entry");

        assert!(
            !text.contains("first_created_in_spex_version"),
            "{} still carries the removed `first_created_in_spex_version` field",
            file.display()
        );
    }
}

#[test]
fn rules_library_entries_ship_every_script_their_checks_run() {
    for file in entry_files() {
        let entry = read_entry(&file);
        let script_folder = file.with_extension("");

        for script in entry
            .checks
            .iter()
            .filter_map(|check| check.script.as_ref())
        {
            assert!(
                script_folder.join(script).is_file(),
                "{} runs script `{script}` missing from {}",
                file.display(),
                script_folder.display()
            );
        }
    }
}
