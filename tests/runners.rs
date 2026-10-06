use std::path::Path;

use tempfile::tempdir;

mod common;
use common::{copy_dir, report_text, run_spex, with_version_header};

const EXPECTED_CHECK_REPORT: &str = r#"runners-project-api
  error  regexp-runner/no-fixme  (src/lib.rs:2)
      matches forbidden pattern
  warn  regexp-runner/require-license-header  (src/util.rs)
      required pattern not found
  warn  glob-runner/no-backup-files  (old.bak)
      forbidden path
  error  ast-grep-runner/no-unwrap  (src/lib.rs:5)
      matches forbidden pattern
  warn  script-runner/per-file-hack  (src/lib.rs:3)
      HACK marker left in code
  error  script-failures/stderr-only
      tooling unavailable
  warn  script-failures/exit-status-only
      script exited with status 3
  3 errors, 4 warnings

runners-project-frontend
  error  regexp-runner/no-fixme  (src/index.ts:2)
      matches forbidden pattern
  warn  regexp-runner/require-license-header  (src/util.ts)
      required pattern not found
  error  glob-runner/require-readme
      required path not found: README.md
  error  ast-grep-runner/no-console-log  (src/index.ts:5)
      matches forbidden pattern
  error  ast-grep-runner/no-debugger-or-alert  (src/debug.ts:3)
      matches forbidden pattern
  error  ast-grep-runner/no-debugger-or-alert  (src/debug.ts:4)
      matches forbidden pattern
  warn  ast-grep-runner/require-default-export  (src/util.ts)
      required pattern not found
  warn  script-runner/per-file-hack  (src/index.ts:3)
      HACK marker left in code
  error  script-runner/per-repo-changelog
      CHANGELOG.md is missing from runners-project-frontend
  6 errors, 3 warnings

runners-project-spex
  no issues

workspace
  error  script-runner/workspace-versions  (../runners-project-frontend/VERSION)
      frontend version 1.3.0 differs from api version 1.2.0
  1 error, 0 warnings

runners-project: 10 errors, 7 warnings
"#;

const EXPECTED_CHECK_FILES_REPORT: &str = r#"runners-project-frontend
  error  regexp-runner/no-fixme  (src/index.ts:2)
      matches forbidden pattern
  error  ast-grep-runner/no-console-log  (src/index.ts:5)
      matches forbidden pattern
  warn  script-runner/per-file-hack  (src/index.ts:3)
      HACK marker left in code
  error  script-runner/per-repo-changelog
      CHANGELOG.md is missing from runners-project-frontend
  3 errors, 1 warning

workspace
  error  script-runner/workspace-versions  (../runners-project-frontend/VERSION)
      frontend version 1.3.0 differs from api version 1.2.0
  1 error, 0 warnings

runners-project: 4 errors, 1 warning
"#;

fn copy_runners_project(project: &Path) {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    copy_dir(&manifest.join("tests/fixtures/runners-project"), project);
    copy_dir(
        &manifest.join("templates/schemas"),
        &project.join("runners-project-spex/schemas"),
    );
}

#[test]
fn check_reports_every_runner_and_script_scope() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");
    copy_runners_project(&project);

    let check = run_spex(&project, &["check"], "");
    assert_eq!(
        report_text(&check.stdout),
        with_version_header(EXPECTED_CHECK_REPORT),
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
fn check_files_narrows_each_runner_and_script_scope() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");
    copy_runners_project(&project);

    let check = run_spex(&project, &["check", "--files", "src/index.ts"], "");
    assert_eq!(
        report_text(&check.stdout),
        with_version_header(EXPECTED_CHECK_FILES_REPORT),
        "check --files report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&check.stderr),
    );
    assert_eq!(
        check.status.code(),
        Some(1),
        "check should exit 1 when errors are present"
    );
}
