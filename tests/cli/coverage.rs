use std::fs;
use std::io::Write;

use tempfile::tempdir;

use crate::common;
use crate::common::{
    assert_fails_mentioning, dummy_project_fixture, report_text, run_spex, with_version_header,
};

fn waive_in_package_entry(project: &std::path::Path, entry_end: &str, waivers: &str) {
    let spex_yaml = project.join("dummy-project-spex/spex.yaml");
    let config = fs::read_to_string(&spex_yaml).expect("read spex.yaml");
    let indented = waivers.replace('\n', "\n    ");
    let patched = config.replace(
        &format!("{entry_end}\n"),
        &format!("{entry_end}{}", indented.trim_end_matches(' ')),
    );
    assert_ne!(patched, config, "package entry not found in spex.yaml");
    fs::write(&spex_yaml, patched).expect("write spex.yaml");
}

fn append(path: &std::path::Path, text: &str) {
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(path)
        .expect("open file for append");
    file.write_all(text.as_bytes()).expect("append to file");
}

const API_ENTRY_END: &str = "    rules:\n      - no-todo-comments";

const FRONTEND_ENTRY_END: &str = "    path: ../dummy-project-frontend\n    rules: []";

const NONE_WITH_GAP_WAIVERS: &str = r#"
coverage_ignore:
  - spec_id: none-with-gap
    layer: impl
    ac: resumes-favorite
  - spec_id: none-with-gap
    layer: test
    ac: resumes-favorite
"#;

const EXPECTED_COVERAGE: &str = include_str!("../fixtures/dummy-project/expected_coverage.txt");

const EXPECTED_COVERAGE_VERBOSE: &str =
    include_str!("../fixtures/dummy-project/expected_coverage_verbose.txt");

const EXPECTED_COVERAGE_PACKAGE_FRONTEND: &str =
    include_str!("../fixtures/dummy-project/expected_coverage_package_frontend.txt");

const NONE_WITH_GAP_IMPL_WAIVER: &str = r#"
coverage_ignore:
  - spec_id: none-with-gap
    layer: impl
    ac: resumes-favorite
"#;

const NONE_WITH_GAP_TEST_WAIVER: &str = r#"
coverage_ignore:
  - spec_id: none-with-gap
    layer: test
    ac: resumes-favorite
"#;

const IN_PROGRESS_SPEC: &str = r#"id: ci-check
title: Verify a change before its spec is updated
description: >-
  A criterion still flagged not done in the spec package.
acceptance_criteria:
  - id: rejects-empty-cart
    must: "reject checkout of an empty cart"
    is_done: false
packages:
  - api
status: in-progress
contract: single
"#;

#[test]
fn coverage_report_matches_expected() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
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
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
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
fn coverage_package_limits_the_report_to_specs_of_that_package() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);

    let coverage = run_spex(&project, &["coverage", "--package", "frontend"], "");
    assert_eq!(
        report_text(&coverage.stdout),
        with_version_header(EXPECTED_COVERAGE_PACKAGE_FRONTEND),
        "coverage --package report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&coverage.stderr),
    );
    assert_eq!(
        coverage.status.code(),
        Some(1),
        "coverage should exit 1 when an active spec of the package has a gap"
    );
}

#[test]
fn coverage_package_accepts_a_full_package_name() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);

    let by_short_name = run_spex(&project, &["coverage", "--package", "frontend"], "");
    let by_full_name = run_spex(
        &project,
        &["coverage", "--package", "dummy-project-frontend"],
        "",
    );

    assert_eq!(
        report_text(&by_full_name.stdout),
        report_text(&by_short_name.stdout)
    );
}

#[test]
fn coverage_package_with_several_packages_leaves_out_only_specs_without_packages() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);

    let everything = report_text(&run_spex(&project, &["coverage"], "").stdout);
    let packaged =
        report_text(&run_spex(&project, &["coverage", "--package", "api", "frontend"], "").stdout);

    assert!(everything.contains("Specs (13)"), "{everything}");
    assert!(everything.contains("in-progress-pending"), "{everything}");
    assert!(packaged.contains("Specs (12)"), "{packaged}");
    assert!(
        !packaged.contains("in-progress-pending"),
        "the spec without packages should be left out\n{packaged}"
    );
    assert!(
        packaged.ends_with("planned (2), in-progress (1), active (9), gaps (7)\n"),
        "{packaged}"
    );
}

#[test]
fn coverage_package_rejects_an_unknown_package() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);

    let coverage = run_spex(&project, &["coverage", "--package", "bogus"], "");

    assert_fails_mentioning(&coverage, "unknown package `bogus`");
    assert!(coverage.stdout.is_empty());
}

#[test]
fn coverage_fails_an_in_progress_not_done_criterion_once_its_impl_lacks_a_test() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
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
        report_text(&nothing_started.stdout)
            .contains("●  ci-check  0/1 AC  in-progress: impl(rejects-empty-cart)\n"),
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

#[test]
fn coverage_honors_a_spex_yaml_package_entry_coverage_ignore_for_a_spec() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);
    waive_in_package_entry(&project, API_ENTRY_END, NONE_WITH_GAP_WAIVERS);

    let coverage = run_spex(&project, &["coverage"], "");

    let report = report_text(&coverage.stdout);
    assert!(
        !report.contains("none-with-gap"),
        "the waived spec should have no gap\n{report}"
    );
    assert!(
        report.ends_with("planned (2), in-progress (2), active (9), gaps (6)\n"),
        "{report}"
    );
}

#[test]
fn coverage_honors_a_package_spex_repo_yaml_coverage_ignore() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);
    fs::write(
        project.join("dummy-project-api/spex.package.yaml"),
        NONE_WITH_GAP_WAIVERS.trim_start(),
    )
    .expect("write spex.package.yaml");

    let coverage = run_spex(&project, &["coverage"], "");

    let report = report_text(&coverage.stdout);
    assert!(
        !report.contains("none-with-gap"),
        "the waived spec should have no gap\n{report}"
    );
    assert!(
        report.ends_with("planned (2), in-progress (2), active (9), gaps (6)\n"),
        "{report}"
    );
}

#[test]
fn coverage_merges_coverage_ignore_from_the_spex_yaml_entry_and_the_spex_package_yaml_of_a_package()
{
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);

    waive_in_package_entry(&project, API_ENTRY_END, NONE_WITH_GAP_IMPL_WAIVER);
    let partly_waived = report_text(&run_spex(&project, &["coverage"], "").stdout);
    assert!(
        partly_waived.contains("missing: test(resumes-favorite)"),
        "waiving only impl should leave the test gap\n{partly_waived}"
    );
    assert!(
        !partly_waived.contains("impl(resumes-favorite)"),
        "the impl waiver should apply\n{partly_waived}"
    );

    fs::write(
        project.join("dummy-project-api/spex.package.yaml"),
        NONE_WITH_GAP_TEST_WAIVER.trim_start(),
    )
    .expect("write spex.package.yaml");
    let fully_waived = report_text(&run_spex(&project, &["coverage"], "").stdout);
    assert!(
        !fully_waived.contains("none-with-gap"),
        "the impl and test waivers should merge\n{fully_waived}"
    );
}

#[test]
fn coverage_skips_annotations_in_the_files_a_spex_yaml_package_exclude_matches() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);
    let before = report_text(&run_spex(&project, &["coverage"], "").stdout);
    assert!(!before.contains("active-full-coverage"), "{before}");

    waive_in_package_entry(&project, API_ENTRY_END, "\nexclude:\n  - src/lib.rs\n");
    let coverage = run_spex(&project, &["coverage"], "");

    let report = report_text(&coverage.stdout);
    assert!(
        report.contains("active-full-coverage"),
        "the spec traced only from the excluded file should now have a gap\n{report}"
    );
    assert_eq!(coverage.status.code(), Some(1), "{report}");
}

#[test]
fn coverage_skips_annotations_in_the_folders_a_package_spex_package_yaml_excludes() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);
    fs::write(
        project.join("dummy-project-api/spex.package.yaml"),
        "exclude:\n  - src\n",
    )
    .expect("write spex.package.yaml");

    let report = report_text(&run_spex(&project, &["coverage"], "").stdout);

    assert!(report.contains("active-full-coverage"), "{report}");
}

#[test]
fn coverage_does_not_apply_the_exclude_of_one_package_to_another() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);
    let before = report_text(&run_spex(&project, &["coverage"], "").stdout);
    assert!(before.contains("in-progress-pending  1/2 AC"), "{before}");
    fs::write(
        project.join("dummy-project-frontend/spex.package.yaml"),
        "exclude:\n  - src\n",
    )
    .expect("write spex.package.yaml");

    let report = report_text(&run_spex(&project, &["coverage"], "").stdout);

    assert!(
        report.contains("in-progress-pending  0/2 AC"),
        "the frontend exclude should drop its own annotations\n{report}"
    );
    assert!(
        !report.contains("active-full-coverage"),
        "the frontend exclude should not hide the api annotations\n{report}"
    );
}

#[test]
fn coverage_fails_naming_the_package_on_an_exclude_that_is_not_a_valid_glob() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);
    fs::write(
        project.join("dummy-project-api/spex.package.yaml"),
        "exclude:\n  - \"src/{a,b\"\n",
    )
    .expect("write spex.package.yaml");

    let coverage = run_spex(&project, &["coverage"], "");

    assert_fails_mentioning(
        &coverage,
        "invalid `exclude` of package `dummy-project-api`",
    );
}

#[test]
fn coverage_ignores_a_package_waiver_of_a_package_the_spec_does_not_list() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);
    fs::write(
        project.join("dummy-project-frontend/spex.package.yaml"),
        NONE_WITH_GAP_WAIVERS.trim_start(),
    )
    .expect("write spex.package.yaml");

    let coverage = run_spex(&project, &["coverage"], "");

    let report = report_text(&coverage.stdout);
    assert!(
        report.contains("none-with-gap"),
        "a frontend waiver should not waive a spec that lists only api\n{report}"
    );
}

#[test]
fn coverage_waives_a_requirement_only_once_every_listed_package_waives_it() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);
    let spec_path = project.join("dummy-project-spex/specs/none-with-gap.yaml");
    let spec = fs::read_to_string(&spec_path).expect("read spec");
    let listing_two_packages =
        spec.replace("packages:\n  - api\n", "packages:\n  - api\n  - frontend\n");
    assert_ne!(listing_two_packages, spec, "packages not found in the spec");
    fs::write(&spec_path, listing_two_packages).expect("write spec");

    fs::write(
        project.join("dummy-project-api/spex.package.yaml"),
        NONE_WITH_GAP_WAIVERS.trim_start(),
    )
    .expect("write spex.package.yaml");
    let waived_by_api_only = report_text(&run_spex(&project, &["coverage"], "").stdout);
    assert!(
        waived_by_api_only.contains("none-with-gap"),
        "a waiver of api alone should not waive a spec that lists api and frontend\n{waived_by_api_only}"
    );

    waive_in_package_entry(&project, FRONTEND_ENTRY_END, NONE_WITH_GAP_WAIVERS);
    let waived_by_both = report_text(&run_spex(&project, &["coverage"], "").stdout);
    assert!(
        !waived_by_both.contains("none-with-gap"),
        "api and frontend both waive the requirement\n{waived_by_both}"
    );
    assert!(
        waived_by_both.ends_with("planned (2), in-progress (2), active (9), gaps (6)\n"),
        "{waived_by_both}"
    );
}

#[test]
fn coverage_fails_on_an_invalid_spex_repo_yaml() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);
    fs::write(
        project.join("dummy-project-api/spex.package.yaml"),
        "coverage_ignore:\n  - spec_id: Not_A_Spec_Id\n    layer: impl\n",
    )
    .expect("write spex.package.yaml");

    let coverage = run_spex(&project, &["coverage"], "");

    assert_fails_mentioning(&coverage, "spex.package.yaml");
    assert!(coverage.stdout.is_empty());
}
