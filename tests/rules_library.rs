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
const SCHEMAS_DIR: &str = "templates/schemas";
const CONFORMANCE_RULE_ID: &str = "spex-internal-conformance";

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

#[test]
fn rules_library_entries_pass_the_conformance_checks_of_spex_check() {
    let ids: Vec<String> = entry_files()
        .iter()
        .map(|file| read_entry(file).id)
        .collect();
    let project = tempfile::tempdir().expect("temp dir");
    let spex_dir = project.path().join("proj-spex");
    let schemas = spex_dir.join("schemas");
    fs::create_dir_all(&schemas).expect("create schemas dir");
    for schema in fs::read_dir(repository().join(SCHEMAS_DIR)).expect("schemas folder") {
        let path = schema.expect("schema").path();
        fs::copy(&path, schemas.join(path.file_name().expect("schema name"))).expect("copy schema");
    }
    fs::create_dir_all(spex_dir.join("docs")).expect("create docs dir");
    fs::write(spex_dir.join("docs/prd.md"), "# Product requirements\n").expect("write prd");
    let listed_rules: String = ids.iter().map(|id| format!("      - {id}\n")).collect();
    let version = env!("CARGO_PKG_VERSION");
    fs::write(
        spex_dir.join("spex.yaml"),
        format!(
            "project_name: proj\ncli_version: {version}\nlayout: multi-repo\nagent_root: .\ndescription: Takes every rules library entry.\nprd: docs/prd.md\npackages:\n  - package_name: proj-spex\n    package_short_name: spex\n    path: .\n    rules:\n{listed_rules}"
        ),
    )
    .expect("write spex.yaml");

    let taken = Command::new(env!("CARGO_BIN_EXE_spex"))
        .current_dir(&spex_dir)
        .args(["lib", "rules", "--copy", "rules", "--id"])
        .args(&ids)
        .output()
        .expect("run spex lib");
    assert!(
        taken.status.success(),
        "{}",
        String::from_utf8_lossy(&taken.stderr)
    );

    let check = Command::new(env!("CARGO_BIN_EXE_spex"))
        .current_dir(&spex_dir)
        .args(["check", "--files", "spex.yaml"])
        .output()
        .expect("run spex check");
    assert!(
        matches!(check.status.code(), Some(0 | 1)),
        "spex check did not run: {}",
        String::from_utf8_lossy(&check.stderr)
    );
    let report = String::from_utf8_lossy(&check.stdout);
    let conformance_findings: Vec<&str> = report
        .lines()
        .filter(|line| line.contains(CONFORMANCE_RULE_ID))
        .collect();

    assert!(
        conformance_findings.is_empty(),
        "rules library entries fail spex check conformance:\n{report}"
    );
}
