use std::path::Path;

use tempfile::tempdir;

mod common;
use common::{
    assert_fails_mentioning, copy_dir, report_text, run_spex, runners_project_fixture,
    script_env_project_fixture, with_version_header,
};

const API_REPORT: &str = r#"runners-project-api
  error  regexp-runner/no-fixme  (src/lib.rs:2)
      matches forbidden pattern
  error  regexp-runner/no-fixme  (src/nested/mid.rs:2)
      matches forbidden pattern
  error  regexp-runner/no-fixme  (src/zeta.rs:1)
      matches forbidden pattern
  warn  regexp-runner/require-license-header  (src/util.rs)
      required pattern not found
  warn  regexp-runner/require-license-header  (src/zeta.rs)
      required pattern not found
  warn  glob-runner/no-backup-files  (archive.bak)
      forbidden path
  warn  glob-runner/no-backup-files  (docs/draft.bak)
      forbidden path
  warn  glob-runner/no-backup-files  (old.bak)
      forbidden path
  error  ast-grep-runner/no-unwrap  (src/lib.rs:5)
      matches forbidden pattern
  error  ast-grep-runner/no-unwrap  (src/nested/mid.rs:5)
      matches forbidden pattern
  error  ast-grep-runner/no-unwrap  (src/zeta.rs:4)
      matches forbidden pattern
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
  7 errors, 9 warnings

"#;

const FRONTEND_REPORT: &str = r#"runners-project-frontend
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
  error  ast-grep-runner/no-trace-call  (src/app.ts:2)
      matches forbidden pattern
  error  ast-grep-runner/no-trace-call  (src/legacy.js:2)
      matches forbidden pattern
  error  ast-grep-runner/no-trace-call  (src/view.tsx:2)
      matches forbidden pattern
  warn  ast-grep-runner/require-default-export  (src/app.ts)
      required pattern not found
  warn  ast-grep-runner/require-default-export  (src/util.ts)
      required pattern not found
  warn  script-runner/per-file-hack  (src/index.ts:3)
      HACK marker left in code
  error  script-runner/per-repo-changelog
      CHANGELOG.md is missing from runners-project-frontend
  9 errors, 4 warnings

"#;

const SPEX_REPORT: &str = r#"runners-project-spex
  no issues

"#;

const WORKSPACE_REPORT: &str = r#"workspace
  error  script-runner/workspace-versions  (../runners-project-frontend/VERSION)
      frontend version 1.3.0 differs from api version 1.2.0
  1 error, 0 warnings

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

const SCRIPT_ENV_REPORT: &str = r#"script-env-project-api
  warn  spex-dir-env/per-file  (src/lib.rs)
      SPEX_DIR is the spex directory
  warn  spex-dir-env/per-repo
      SPEX_DIR is the spex directory
  0 errors, 2 warnings

script-env-project-spex
  no issues

workspace
  warn  spex-dir-env/workspace
      SPEX_DIR is the spex directory
  0 errors, 1 warning

script-env-project: 0 errors, 3 warnings
"#;

fn expected_report(sections: &[&str], summary: &str) -> String {
    with_version_header(&format!("{}{summary}\n", sections.concat()))
}

fn copy_runners_project(project: &Path) {
    copy_dir(&runners_project_fixture(), project);
    copy_dir(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("templates/schemas"),
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
        expected_report(
            &[API_REPORT, FRONTEND_REPORT, SPEX_REPORT, WORKSPACE_REPORT],
            "runners-project: 17 errors, 13 warnings",
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

#[test]
fn check_repo_limits_the_report_to_the_named_repository() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");
    copy_runners_project(&project);

    let by_short_name = run_spex(&project, &["check", "--repo", "frontend"], "");
    assert_eq!(
        report_text(&by_short_name.stdout),
        expected_report(
            &[FRONTEND_REPORT, WORKSPACE_REPORT],
            "runners-project: 10 errors, 4 warnings",
        ),
        "check --repo report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&by_short_name.stderr),
    );
    assert_eq!(by_short_name.status.code(), Some(1));

    let by_full_name = run_spex(
        &project,
        &["check", "--repo", "runners-project-frontend"],
        "",
    );
    assert_eq!(
        report_text(&by_full_name.stdout),
        report_text(&by_short_name.stdout),
        "--repo should match a repository by full name as well as short name",
    );
}

#[test]
fn check_repo_runs_a_workspace_script_only_when_a_repository_using_it_is_selected() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");
    copy_runners_project(&project);

    let check = run_spex(&project, &["check", "--repo", "spex"], "");

    assert_eq!(
        report_text(&check.stdout),
        expected_report(&[SPEX_REPORT], "runners-project: 0 errors, 0 warnings"),
        "check --repo spex report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&check.stderr),
    );
    assert!(
        check.status.success(),
        "check should exit 0 when the selected repository has no findings"
    );
}

#[test]
fn check_repo_rejects_an_unknown_repository() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");
    copy_runners_project(&project);

    let check = run_spex(&project, &["check", "--repo", "bogus"], "");

    assert_fails_mentioning(&check, "unknown workspace repository `bogus`");
    assert!(
        check.stdout.is_empty(),
        "no report should print when the repository is unknown"
    );
}

#[test]
fn check_exposes_the_spex_dir_to_a_script_at_every_run_scope() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");
    copy_dir(&script_env_project_fixture(), &project);
    copy_dir(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("templates/schemas"),
        &project.join("script-env-project-spex/schemas"),
    );

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
