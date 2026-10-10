use std::fs;
use std::io::Write;

use tempfile::tempdir;

mod common;
use common::{
    assert_fails_mentioning, dummy_project_fixture, report_text, run_spex, with_version_header,
};

const EXPECTED_COVERAGE: &str = include_str!("fixtures/dummy-project/expected_coverage.txt");
const EXPECTED_COVERAGE_VERBOSE: &str =
    include_str!("fixtures/dummy-project/expected_coverage_verbose.txt");
const EXPECTED_COVERAGE_SCOPE_FRONTEND: &str =
    include_str!("fixtures/dummy-project/expected_coverage_scope_frontend.txt");

#[test]
fn coverage_report_matches_expected() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);

    let coverage = run_spex(&project, &["coverage"], "");
    assert_eq!(
        report_text(&coverage.stdout),
        with_version_header(EXPECTED_COVERAGE),
        "coverage report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&coverage.stderr),
    );
    assert_eq!(
        coverage.status.code(),
        Some(1),
        "coverage should exit 1 when an active spec has a gap"
    );
}

#[test]
fn coverage_verbose_report_matches_expected() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);

    let coverage = run_spex(&project, &["coverage", "--verbose"], "");
    assert_eq!(
        report_text(&coverage.stdout),
        with_version_header(EXPECTED_COVERAGE_VERBOSE),
        "coverage --verbose report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&coverage.stderr),
    );
    assert_eq!(
        coverage.status.code(),
        Some(1),
        "coverage should exit 1 when an active spec has a gap"
    );
}

#[test]
fn coverage_scope_limits_the_report_to_specs_in_that_scope() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);

    let coverage = run_spex(&project, &["coverage", "--scope", "frontend"], "");
    assert_eq!(
        report_text(&coverage.stdout),
        with_version_header(EXPECTED_COVERAGE_SCOPE_FRONTEND),
        "coverage --scope report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&coverage.stderr),
    );
    assert_eq!(
        coverage.status.code(),
        Some(1),
        "coverage should exit 1 when an in-scope active spec has a gap"
    );
}

#[test]
fn coverage_scope_with_several_scopes_leaves_out_only_unscoped_specs() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);

    let everything = report_text(&run_spex(&project, &["coverage"], "").stdout);
    let scoped =
        report_text(&run_spex(&project, &["coverage", "--scope", "api", "frontend"], "").stdout);

    assert!(everything.contains("Specs (13)"), "{everything}");
    assert!(everything.contains("in-progress-pending"), "{everything}");
    assert!(scoped.contains("Specs (12)"), "{scoped}");
    assert!(
        !scoped.contains("in-progress-pending"),
        "the unscoped spec should be left out\n{scoped}"
    );
    assert!(
        scoped.ends_with("planned (2), in-progress (1), active (9), gaps (7)\n"),
        "{scoped}"
    );
}

#[test]
fn coverage_scope_rejects_an_unknown_repository() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);

    let coverage = run_spex(&project, &["coverage", "--scope", "bogus"], "");

    assert_fails_mentioning(&coverage, "unknown repository short name `bogus`");
    assert!(coverage.stdout.is_empty());
}

const IN_PROGRESS_SPEC: &str = r#"id: ci-check
title: Verify a change before its spec is updated
description: >-
  A criterion still flagged not done in the spec repository.
acceptance_criteria:
  - id: rejects-empty-cart
    must: "reject checkout of an empty cart"
    is_done: false
scope:
  - api
status: in-progress
contract: single
"#;

fn append(path: &std::path::Path, text: &str) {
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(path)
        .expect("open file for append");
    file.write_all(text.as_bytes()).expect("append to file");
}

#[test]
fn coverage_fails_an_in_progress_not_done_criterion_once_its_impl_lacks_a_test() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);
    let specs = project.join("dummy-project-spex/specs");
    fs::remove_dir_all(&specs).expect("clear specs");
    fs::create_dir(&specs).expect("recreate specs");
    fs::write(specs.join("ci-check.yaml"), IN_PROGRESS_SPEC).expect("write spec");
    let contracts = project.join("dummy-project-spex/contracts.md");
    let api_source = project.join("dummy-project-api/src/lib.rs");
    fs::write(&api_source, "").expect("clear api source");
    fs::write(&contracts, "<!-- @spec contract ci-check -->\n").expect("write contract");

    let nothing_started = run_spex(&project, &["coverage"], "");
    assert!(
        report_text(&nothing_started.stdout).contains(
            "●  ci-check  0/1 AC  in-progress: impl(rejects-empty-cart), test(rejects-empty-cart)"
        ),
        "{}",
        report_text(&nothing_started.stdout),
    );
    assert_eq!(
        nothing_started.status.code(),
        Some(0),
        "a criterion not started yet is pending, not an error"
    );

    append(
        &api_source,
        "// @spec impl ci-check#rejects-empty-cart\nfn reject_empty_cart() {}\n",
    );
    let implemented = run_spex(&project, &["coverage"], "");
    assert!(
        report_text(&implemented.stdout)
            .contains("✗  ci-check  1/1 AC  missing: test(rejects-empty-cart)"),
        "{}",
        report_text(&implemented.stdout),
    );
    assert_eq!(
        implemented.status.code(),
        Some(1),
        "an implemented criterion without a test is an error even while flagged not done"
    );

    append(
        &api_source,
        "// @spec test ci-check#rejects-empty-cart\nfn reject_empty_cart_test() {}\n",
    );
    let tested = run_spex(&project, &["coverage"], "");
    assert_eq!(
        tested.status.code(),
        Some(0),
        "{}",
        report_text(&tested.stdout)
    );
}
