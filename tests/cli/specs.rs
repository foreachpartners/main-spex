use std::fs;

use serde_json::json;
use tempfile::tempdir;

use crate::common::{
    assert_fails_mentioning, canonical_display, copy_project, dummy_project_fixture, listed_ids,
    queried_by_jq, run_spex, temp_project,
};

const EXPECTED_SPECS: &str = r#"- id: active-with-gap
  path: {spex_dir}/specs/active-with-gap.yaml
  title: Validate registration input
  description: Validates a new account's registration input before it is created, rejecting invalid or duplicate submissions.
  acceptance_criteria:
  - id: validates-input
    must: reject a registration request missing a required field
  - id: normalizes-format
    must: normalize the submitted email to lowercase before storing it
  - id: rejects-duplicate
    must: reject a registration request for an email already registered
  - id: logs-attempt
    must: log every registration attempt, successful or not
  packages:
  - frontend
  status: active
  contract: single

- id: planned-one
  path: {spex_dir}/specs/planned-one.yaml
  title: Let customers save a favorite order
  description: Lets a returning customer save an order as a favorite so they can reorder it in one click next time.
  acceptance_criteria:
  - id: save-as-favorite
    must: let a customer save a completed order as a favorite
    is_done: false
  packages:
  - frontend
  status: planned
  contract: single
"#;

#[test]
fn specs_id_limits_output_to_the_named_specs() {
    let (_temp_dir, project) = temp_project(&dummy_project_fixture());

    let specs = run_spex(
        &project,
        &["specs", "--id", "planned-one", "active-with-gap"],
        "",
    );

    assert!(specs.status.success(), "specs should exit 0");
    assert_eq!(
        listed_ids(&specs.stdout),
        ["active-with-gap", "planned-one"]
    );
}

#[test]
fn specs_prints_the_spec_path_after_the_id() {
    let (_temp_dir, project) = temp_project(&dummy_project_fixture());

    let specs = run_spex(
        &project,
        &["specs", "--id", "planned-one", "active-with-gap"],
        "",
    );

    assert!(specs.status.success());
    let spex_dir = canonical_display(&project.join("dummy-project-spex"));
    assert_eq!(
        String::from_utf8_lossy(&specs.stdout),
        EXPECTED_SPECS.replace("{spex_dir}", &spex_dir)
    );
}

#[test]
fn specs_fields_prints_only_those_fields_of_each_spec() {
    let (_temp_dir, project) = temp_project(&dummy_project_fixture());

    let specs = run_spex(
        &project,
        &[
            "specs",
            "--id",
            "planned-one",
            "--fields",
            "title",
            "id",
            "path",
        ],
        "",
    );

    assert!(specs.status.success());
    let spex_dir = canonical_display(&project.join("dummy-project-spex"));
    assert_eq!(
        String::from_utf8_lossy(&specs.stdout),
        format!(
            "- id: planned-one\n  path: {spex_dir}/specs/planned-one.yaml\n  title: Let customers save a favorite order\n"
        )
    );
}

#[test]
fn specs_fields_rejects_a_name_that_is_no_spec_field() {
    let (_temp_dir, project) = temp_project(&dummy_project_fixture());

    let specs = run_spex(&project, &["specs", "--fields", "id", "titel"], "");

    assert_fails_mentioning(&specs, "unknown field `titel` — a spec has: id, ");
    assert!(specs.stdout.is_empty());
}

#[test]
fn specs_search_prints_only_specs_containing_a_keyword_ignoring_case() {
    let (_temp_dir, project) = temp_project(&dummy_project_fixture());

    let specs = run_spex(
        &project,
        &[
            "specs",
            "--search",
            "ONE CLICK",
            "no-such-word",
            "--fields",
            "id",
        ],
        "",
    );

    assert!(specs.status.success());
    assert_eq!(
        String::from_utf8_lossy(&specs.stdout),
        "- id: planned-one\n"
    );
}

#[test]
fn specs_search_matching_nothing_prints_an_empty_list_and_nothing_found_on_stderr() {
    let (_temp_dir, project) = temp_project(&dummy_project_fixture());

    let specs = run_spex(&project, &["specs", "--search", "no-such-word"], "");

    assert!(specs.status.success());
    assert_eq!(String::from_utf8_lossy(&specs.stdout), "[]\n");
    assert!(String::from_utf8_lossy(&specs.stderr).contains("nothing found\n"));
}

#[test]
fn specs_selecting_no_spec_by_id_and_package_prints_an_empty_list_and_nothing_found_on_stderr() {
    let (_temp_dir, project) = temp_project(&dummy_project_fixture());

    let specs = run_spex(
        &project,
        &["specs", "--id", "planned-one", "--package", "api"],
        "",
    );

    assert!(specs.status.success());
    assert_eq!(String::from_utf8_lossy(&specs.stdout), "[]\n");
    assert!(String::from_utf8_lossy(&specs.stderr).contains("nothing found\n"));
}

#[test]
fn specs_selecting_some_specs_does_not_say_nothing_found() {
    let (_temp_dir, project) = temp_project(&dummy_project_fixture());

    let specs = run_spex(&project, &["specs", "--id", "planned-one"], "");

    assert!(specs.status.success());
    assert!(!String::from_utf8_lossy(&specs.stderr).contains("nothing found"));
}

#[test]
fn specs_package_limits_output_to_specs_of_that_package() {
    let (_temp_dir, project) = temp_project(&dummy_project_fixture());

    let specs = run_spex(&project, &["specs", "--package", "frontend"], "");

    assert!(specs.status.success());
    assert_eq!(
        listed_ids(&specs.stdout),
        [
            "active-with-gap",
            "foreach-not-covered",
            "in-progress-mixed",
            "in-progress-pending",
            "planned-one"
        ]
    );
}

#[test]
fn specs_package_accepts_a_full_package_name() {
    let (_temp_dir, project) = temp_project(&dummy_project_fixture());

    let by_short_name = run_spex(&project, &["specs", "--package", "frontend"], "");
    let by_full_name = run_spex(
        &project,
        &["specs", "--package", "dummy-project-frontend"],
        "",
    );

    assert!(by_full_name.status.success());
    assert_eq!(
        listed_ids(&by_full_name.stdout),
        listed_ids(&by_short_name.stdout)
    );
}

#[test]
fn specs_package_lists_a_spec_in_several_packages_once() {
    let (_temp_dir, project) = temp_project(&dummy_project_fixture());

    let everything = run_spex(&project, &["specs"], "");
    let packaged = run_spex(&project, &["specs", "--package", "api", "frontend"], "");

    let packaged_ids = listed_ids(&packaged.stdout);
    let mut distinct = packaged_ids.clone();
    distinct.dedup();
    assert_eq!(
        packaged_ids, distinct,
        "a spec in both packages was listed twice"
    );
    assert!(packaged_ids.contains(&"in-progress-mixed".to_string()));
    assert_eq!(packaged_ids, listed_ids(&everything.stdout));
}

#[test]
fn specs_rejects_an_unknown_spec_id() {
    let (_temp_dir, project) = temp_project(&dummy_project_fixture());

    let specs = run_spex(&project, &["specs", "--id", "planned-one", "bogus"], "");

    assert_fails_mentioning(&specs, "unknown spec id `bogus`");
    assert!(specs.stdout.is_empty());
}

#[test]
fn specs_rejects_an_unknown_package() {
    let (_temp_dir, project) = temp_project(&dummy_project_fixture());

    let specs = run_spex(&project, &["specs", "--package", "bogus"], "");

    assert_fails_mentioning(&specs, "unknown package `bogus`");
    assert!(specs.stdout.is_empty());
}

#[test]
fn specs_output_is_a_valid_yaml_list_that_jq_reads() {
    let (_temp_dir, project) = temp_project(&dummy_project_fixture());

    let all = run_spex(&project, &["specs"], "");
    let fields = run_spex(&project, &["specs", "--fields", "id", "description"], "");
    let none = run_spex(&project, &["specs", "--search", "no-such-word"], "");

    for output in [&all, &fields, &none] {
        assert!(output.status.success(), "{output:?}");
    }
    assert_eq!(
        queried_by_jq(&project, &all.stdout, "map(.id) | sort"),
        json!(listed_ids(&all.stdout)),
    );
    assert_eq!(
        queried_by_jq(&project, &fields.stdout, "map(keys) | unique"),
        json!([["description", "id"]]),
    );
    assert_eq!(queried_by_jq(&project, &none.stdout, "length"), json!(0));
}

#[test]
fn specs_output_flag_writes_file_matching_stdout() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");

    copy_project(&dummy_project_fixture(), &project);

    let stdout_run = run_spex(&project, &["specs"], "");
    assert!(
        stdout_run.status.success(),
        "specs failed (exit {:?})\nstderr:\n{}",
        stdout_run.status.code(),
        String::from_utf8_lossy(&stdout_run.stderr),
    );
    let stdout_yaml = String::from_utf8_lossy(&stdout_run.stdout).into_owned();
    assert!(!stdout_yaml.is_empty(), "expected specs on stdout");

    let output_path = temp_dir.path().join("specs.yaml");
    let file_run = run_spex(
        &project,
        &[
            "specs",
            "--output",
            output_path.to_str().expect("utf8 path"),
        ],
        "",
    );
    assert!(
        file_run.status.success(),
        "specs --output failed (exit {:?})\nstderr:\n{}",
        file_run.status.code(),
        String::from_utf8_lossy(&file_run.stderr),
    );
    assert!(
        file_run.stdout.is_empty(),
        "specs --output should not print to stdout"
    );

    let file_yaml = fs::read_to_string(&output_path).expect("read --output file");
    assert_eq!(
        file_yaml, stdout_yaml,
        "--output file contents should match stdout output"
    );
}
