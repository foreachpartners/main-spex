use std::fs;
use std::path::PathBuf;

use tempfile::TempDir;

use crate::common::{assert_fails_mentioning, dummy_project_fixture, run_spex, temp_project};

fn spex_dir() -> (TempDir, PathBuf) {
    let (workspace, project) = temp_project(&dummy_project_fixture());
    let spex = project.join("dummy-project-spex");
    (workspace, spex)
}

const BAD_ID_SPEC: &str = "id: Bad_ID
title: Bad spec
description: A spec whose id breaks the sibling spec-id schema.
acceptance_criteria:
  - id: valid-criterion
    must: do something
contract: none
";

const TITLE_ONLY_SPEC: &str = "title: Only a title\n";
const VALID_JSON_RULE: &str = r#"{"id":"json-rule","title":"Json rule","description":"A rule written as JSON.","acceptance_criteria":["MUST do a thing"]}"#;
const INCOMPLETE_JSON_RULE: &str = r#"{"id":"json-rule"}"#;

#[test]
fn check_schema_accepts_valid_files_without_output() {
    let (_workspace, spex) = spex_dir();

    let check = run_spex(
        &spex,
        &[
            "check-schema",
            "--schema",
            "schemas/spec.schema.json",
            "--files",
            "specs/planned-one.yaml",
            "specs/active-with-gap.yaml",
        ],
        "",
    );

    assert!(check.status.success(), "check-schema should exit 0");
    assert!(check.stdout.is_empty());
    assert!(
        check.stderr.is_empty(),
        "stderr:\n{}",
        String::from_utf8_lossy(&check.stderr)
    );
}

#[test]
fn check_schema_reports_each_violation_per_file_and_exits_non_zero() {
    let (_workspace, spex) = spex_dir();
    fs::write(spex.join("bad-id.yaml"), BAD_ID_SPEC).expect("write bad-id spec");
    fs::write(spex.join("missing-fields.yaml"), TITLE_ONLY_SPEC).expect("write title-only spec");

    let check = run_spex(
        &spex,
        &[
            "check-schema",
            "--schema",
            "schemas/spec.schema.json",
            "--files",
            "specs/planned-one.yaml",
            "bad-id.yaml",
            "missing-fields.yaml",
        ],
        "",
    );

    assert_eq!(check.status.code(), Some(1));
    assert!(check.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&check.stderr),
        concat!(
            "bad-id.yaml: /id: \"Bad_ID\" does not match \"^[a-z0-9]+(-[a-z0-9]+)*$\"\n",
            "missing-fields.yaml: \"acceptance_criteria\" is a required property\n",
            "missing-fields.yaml: \"contract\" is a required property\n",
            "missing-fields.yaml: \"description\" is a required property\n",
            "missing-fields.yaml: \"id\" is a required property\n",
        ),
    );
}

#[test]
fn check_schema_validates_json_files() {
    let (_workspace, spex) = spex_dir();
    fs::write(spex.join("valid-rule.json"), VALID_JSON_RULE).expect("write valid json");
    fs::write(spex.join("incomplete-rule.json"), INCOMPLETE_JSON_RULE).expect("write invalid json");

    let valid = run_spex(
        &spex,
        &[
            "check-schema",
            "--schema",
            "schemas/rule.schema.json",
            "--files",
            "valid-rule.json",
        ],
        "",
    );
    let invalid = run_spex(
        &spex,
        &[
            "check-schema",
            "--schema",
            "schemas/rule.schema.json",
            "--files",
            "incomplete-rule.json",
        ],
        "",
    );

    assert!(valid.status.success());
    assert!(valid.stderr.is_empty());
    assert_eq!(invalid.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&invalid.stderr),
        concat!(
            "incomplete-rule.json: \"acceptance_criteria\" is a required property\n",
            "incomplete-rule.json: \"description\" is a required property\n",
            "incomplete-rule.json: \"title\" is a required property\n",
        ),
    );
}

#[test]
fn check_schema_fails_when_a_file_or_the_schema_cannot_be_read() {
    let (_workspace, spex) = spex_dir();

    let missing_file = run_spex(
        &spex,
        &[
            "check-schema",
            "--schema",
            "schemas/spec.schema.json",
            "--files",
            "nope.yaml",
        ],
        "",
    );
    let missing_schema = run_spex(
        &spex,
        &[
            "check-schema",
            "--schema",
            "schemas/nope.json",
            "--files",
            "specs/planned-one.yaml",
        ],
        "",
    );

    assert_fails_mentioning(&missing_file, "could not read `nope.yaml`");
    assert_fails_mentioning(&missing_schema, "could not read `schemas/nope.json`");
}

#[test]
fn check_schema_requires_a_schema_and_files() {
    let (_workspace, spex) = spex_dir();

    let check = run_spex(&spex, &["check-schema"], "");

    assert_eq!(check.status.code(), Some(2));
    assert_fails_mentioning(&check, "--schema <SCHEMA>");
    assert_fails_mentioning(&check, "--files <FILES>...");
}
