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

// `dummy-project` is a small, dedicated fixture (see
// tests/fixtures/dummy-project) whose few specs are picked to exercise
// every state `spex coverage` reports: for each `contract` level (`none`,
// `single`, `for-each-ac`) one fully covered active spec, one active spec
// with a partial gap, and one active spec with no annotations at all, plus
// an in-progress spec with only a pending (not-yet-traced) criterion, an
// in-progress spec mixing a real gap with a pending criterion (and spanning
// two scopes), and two planned specs. This exercises each contract gap
// shape: no contract entry under `none`, a spec-level `contract` under
// `single`, and a `contract(ac-id)` per criterion under `for-each-ac`.
#[test]
fn coverage_report_matches_expected() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");
    common::copy_dir(&dummy_project_fixture(), &project);

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
    common::copy_dir(&dummy_project_fixture(), &project);

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
    common::copy_dir(&dummy_project_fixture(), &project);

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
    common::copy_dir(&dummy_project_fixture(), &project);

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
    common::copy_dir(&dummy_project_fixture(), &project);

    let coverage = run_spex(&project, &["coverage", "--scope", "bogus"], "");

    assert_fails_mentioning(&coverage, "unknown repository short name `bogus`");
    assert!(coverage.stdout.is_empty());
}
