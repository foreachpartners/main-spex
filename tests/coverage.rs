use tempfile::tempdir;

mod common;
use common::{dummy_project_fixture, report_text, run_spex};

const EXPECTED_COVERAGE: &str = include_str!("fixtures/dummy-project/expected_coverage.txt");
const EXPECTED_COVERAGE_VERBOSE: &str =
    include_str!("fixtures/dummy-project/expected_coverage_verbose.txt");

// `dummy-project` is a small, dedicated fixture (see
// tests/fixtures/dummy-project) whose few specs are picked to exercise
// every state `spex coverage` reports: a fully covered active spec, an
// active spec missing its contract and some impl/test annotations, an
// in-progress spec with only a pending (not-yet-traced) criterion, an
// in-progress spec mixing a real gap with a pending criterion (and spanning
// two scopes), and two planned specs.
#[test]
fn coverage_report_matches_expected() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");
    common::copy_dir(&dummy_project_fixture(), &project);

    let coverage = run_spex(&project, &["coverage"], "");
    assert_eq!(
        report_text(&coverage.stdout),
        EXPECTED_COVERAGE,
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
        EXPECTED_COVERAGE_VERBOSE,
        "coverage --verbose report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&coverage.stderr),
    );
    assert_eq!(
        coverage.status.code(),
        Some(1),
        "coverage should exit 1 when an active spec has a gap"
    );
}
