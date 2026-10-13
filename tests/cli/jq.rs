use std::fs;

use tempfile::tempdir;

use crate::common;
use crate::common::{assert_fails_mentioning, run_spex};

#[test]
fn jq_reads_json_from_stdin() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(
        dir.path(),
        &["jq", "-r", ".packages[].path"],
        r#"{"packages": [{"path": "../api"}, {"path": "../frontend"}]}"#,
    );

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "../api\n../frontend\n"
    );
}

#[test]
fn jq_reads_a_yaml_file() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("spex.yaml"),
        "project_name: shopping-notes\n",
    )
    .expect("write yaml");

    let output = run_spex(dir.path(), &["jq", "-r", ".project_name", "spex.yaml"], "");

    assert!(output.status.success(), "{output:?}");
    assert_eq!(String::from_utf8_lossy(&output.stdout), "shopping-notes\n");
}

#[test]
fn jq_prints_compact_json() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(dir.path(), &["jq", "-c", "."], r#"{ "a": [1, 2] }"#);

    assert!(output.status.success(), "{output:?}");
    assert_eq!(String::from_utf8_lossy(&output.stdout), "{\"a\":[1,2]}\n");
}

#[test]
fn jq_pipes_from_spex_info() {
    let dir = tempdir().expect("tempdir");
    let project = dir.path().join("proj");
    common::copy_project(&common::dummy_project_fixture(), &project);
    let info = run_spex(&project, &["info"], "");
    assert!(info.status.success(), "{info:?}");

    let output = run_spex(
        &project,
        &["jq", "-r", ".packages[].package_short_name"],
        &String::from_utf8_lossy(&info.stdout),
    );

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "api\nfrontend\nspex\n"
    );
}

#[test]
fn jq_rejects_an_invalid_filter_with_exit_code_2() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(dir.path(), &["jq", ".a["], "{}");

    assert_eq!(output.status.code(), Some(2));
    assert_fails_mentioning(&output, "error: invalid jq filter");
    assert!(output.stdout.is_empty());
}

#[test]
fn jq_rejects_a_missing_file_with_exit_code_2() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(dir.path(), &["jq", ".", "missing.yaml"], "");

    assert_eq!(output.status.code(), Some(2));
    assert_fails_mentioning(&output, "error: could not read `missing.yaml`");
}
