use std::fs;
use std::path::Path;

use serde_json::Value;
use tempfile::tempdir;

use crate::common::{assert_fails_mentioning, run_spex};

fn bundled_entry(id: &str) -> Value {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("templates/specs")
        .join(format!("{id}.yaml"));
    let text = fs::read_to_string(file).expect("read bundled entry");
    serde_yaml_ng::from_str(&text).expect("parse bundled entry")
}

fn printed_list(stdout: &[u8]) -> Vec<Value> {
    serde_yaml_ng::from_slice(stdout).expect("stdout is a YAML list")
}

#[test]
fn lib_specs_prints_every_entry_in_full_as_a_list_outside_a_project() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(dir.path(), &["lib", "specs"], "");

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        printed_list(&output.stdout),
        [
            bundled_entry("email-policy"),
            bundled_entry("password-policy")
        ]
    );
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[test]
fn lib_specs_id_prints_the_entry_as_a_one_item_list() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(dir.path(), &["lib", "specs", "--id", "password-policy"], "");

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        printed_list(&output.stdout),
        [bundled_entry("password-policy")]
    );
}

#[test]
fn lib_specs_fields_prints_only_those_fields_of_each_entry() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(
        dir.path(),
        &["lib", "specs", "--fields", "id", "description"],
        "",
    );

    assert!(output.status.success(), "{output:?}");
    let printed = String::from_utf8_lossy(&output.stdout);
    assert!(
        printed.starts_with("- id: email-policy\n  description: "),
        "{printed}"
    );
    assert!(printed.contains("\n\n- id: password-policy\n  description: "));
    assert!(!printed.contains("acceptance_criteria"));
}

#[test]
fn lib_specs_search_prints_only_entries_containing_a_keyword() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(
        dir.path(),
        &["lib", "specs", "--search", "ARGON2ID", "--fields", "id"],
        "",
    );

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "- id: password-policy\n"
    );
}

#[test]
fn lib_specs_search_matching_nothing_prints_an_empty_list_and_nothing_found_on_stderr() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(
        dir.path(),
        &["lib", "specs", "--search", "no-such-word"],
        "",
    );

    assert!(output.status.success(), "{output:?}");
    assert_eq!(String::from_utf8_lossy(&output.stdout), "[]\n");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "nothing found\n");
}

#[test]
fn lib_specs_rejects_an_unknown_field() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(dir.path(), &["lib", "specs", "--fields", "nope"], "");

    assert_fails_mentioning(&output, "unknown field `nope`");
}

#[test]
fn lib_specs_output_writes_the_file_instead_of_printing() {
    let dir = tempdir().expect("tempdir");
    let target = dir.path().join("out.yaml");

    let output = run_spex(
        dir.path(),
        &[
            "lib",
            "specs",
            "--id",
            "password-policy",
            "--output",
            target.to_str().expect("utf-8 path"),
        ],
        "",
    );

    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "nothing should be printed");
    assert_eq!(
        printed_list(&fs::read(target).expect("read output")),
        [bundled_entry("password-policy")]
    );
}

#[test]
fn lib_specs_rejects_an_unknown_id() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(dir.path(), &["lib", "specs", "--id", "nope"], "");

    assert_fails_mentioning(&output, "unknown id `nope`");
}

#[test]
fn lib_rejects_an_unknown_kind() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(dir.path(), &["lib", "rules"], "");

    assert_eq!(output.status.code(), Some(2));
    assert_fails_mentioning(&output, "invalid value 'rules'");
}
