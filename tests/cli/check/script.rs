use tempfile::tempdir;

use super::{SPEX_REPORT, expected_report, project_with_rules};
use crate::common::{
    copy_project, report_text, run_spex, runners_project_fixture, script_env_project_fixture,
    with_version_header,
};

const API_REPORT: &str = r#"runners-project-api
  warn  script-runner/per-file-hack  (src/lib.rs:3)
      HACK marker left in code
  warn  script-runner/per-file-hack  (src/nested/mid.rs:3)
      HACK marker left in code
  warn  script-runner/per-file-hack  (src/zeta.rs:2)
      HACK marker left in code
  error  script-failures/stderr-only
      tooling unavailable
  warn  script-failures/exit-status-only
      script exited with status 3
  1 error, 4 warnings

"#;
const FRONTEND_REPORT: &str = r#"runners-project-frontend
  warn  script-runner/per-file-hack  (src/index.ts:3)
      HACK marker left in code
  error  script-runner/per-package-changelog
      CHANGELOG.md is missing from runners-project-frontend
  1 error, 1 warning

"#;
const PROJECT_REPORT: &str = r#"project
  error  script-runner/project-versions  (../runners-project-frontend/VERSION)
      frontend version 1.3.0 differs from api version 1.2.0
  1 error, 0 warnings

"#;
const SCRIPT_ENV_REPORT: &str = r#"script-env-project-api
  warn  spex-dir-env/per-file  (src/lib.rs)
      SPEX_DIR is the spex directory
  warn  spex-dir-env/per-package
      SPEX_DIR is the spex directory
  0 errors, 2 warnings

script-env-project-spex
  no issues

project
  warn  spex-dir-env/project
      SPEX_DIR is the spex directory
  0 errors, 1 warning

script-env-project: 0 errors, 3 warnings
"#;

#[test]
fn check_reports_every_script_finding_at_each_run_scope() {
    let (_temp_dir, project) = project_with_rules(&["script-runner", "script-failures"]);

    let check = run_spex(&project, &["check"], "");

    assert_eq!(
        report_text(&check.stdout),
        expected_report(
            &[API_REPORT, FRONTEND_REPORT, SPEX_REPORT, PROJECT_REPORT],
            "runners-project: 3 errors, 5 warnings"
        ),
        "check report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&check.stderr),
    );
    assert_eq!(
        check.status.code(),
        Some(1),
        "check should exit 1 when errors are present"
    );
}

#[test]
fn check_files_narrows_script_findings_at_each_run_scope() {
    let (_temp_dir, project) = project_with_rules(&["script-runner", "script-failures"]);

    let check = run_spex(&project, &["check", "--files", "src/index.ts"], "");

    assert_eq!(
        report_text(&check.stdout),
        expected_report(
            &[FRONTEND_REPORT, PROJECT_REPORT],
            "runners-project: 2 errors, 1 warning"
        ),
        "check --files report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&check.stderr),
    );
    assert_eq!(
        check.status.code(),
        Some(1),
        "check should exit 1 when errors are present"
    );
}

#[test]
fn check_package_limits_script_findings_to_the_named_package() {
    let (_temp_dir, project) = project_with_rules(&["script-runner", "script-failures"]);

    let check = run_spex(&project, &["check", "--package", "frontend"], "");

    assert_eq!(
        report_text(&check.stdout),
        expected_report(
            &[FRONTEND_REPORT, PROJECT_REPORT],
            "runners-project: 2 errors, 1 warning"
        ),
        "check --package report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&check.stderr),
    );
    assert_eq!(
        check.status.code(),
        Some(1),
        "check should exit 1 when errors are present"
    );
}

#[test]
fn check_package_runs_a_project_script_only_when_a_package_using_it_is_selected() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    copy_project(&runners_project_fixture(), &project);

    let check = run_spex(&project, &["check", "--package", "spex"], "");

    assert_eq!(
        report_text(&check.stdout),
        expected_report(&[SPEX_REPORT], "runners-project: 0 errors, 0 warnings"),
        "check --package spex report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&check.stderr),
    );
    assert!(
        check.status.success(),
        "check should exit 0 when the selected package has no findings"
    );
}

#[test]
fn check_exposes_the_spex_dir_to_a_script_at_every_run_scope() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    copy_project(&script_env_project_fixture(), &project);

    let check = run_spex(&project, &["check"], "");

    assert_eq!(
        report_text(&check.stdout),
        with_version_header(SCRIPT_ENV_REPORT),
        "check report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&check.stderr),
    );
    assert!(
        check.status.success(),
        "only warnings, so check should exit 0"
    );
}
