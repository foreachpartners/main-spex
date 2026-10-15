use super::{SPEX_REPORT, expected_report, project_with_rules};
use crate::common::{report_text, run_spex};

const API_REPORT: &str = r#"runners-project-api
  error  ast-grep-runner/no-unwrap  (src/lib.rs:5)
      matches forbidden pattern
  error  ast-grep-runner/no-unwrap  (src/nested/mid.rs:5)
      matches forbidden pattern
  error  ast-grep-runner/no-unwrap  (src/zeta.rs:4)
      matches forbidden pattern
  3 errors, 0 warnings

"#;
const FRONTEND_REPORT: &str = r#"runners-project-frontend
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
  6 errors, 2 warnings

"#;
const FRONTEND_INDEX_REPORT: &str = r#"runners-project-frontend
  error  ast-grep-runner/no-console-log  (src/index.ts:5)
      matches forbidden pattern
  1 error, 0 warnings

"#;

#[test]
fn check_reports_every_ast_grep_finding() {
    let (_temp_dir, project) = project_with_rules(&["ast-grep-runner"]);

    let check = run_spex(&project, &["check"], "");

    assert_eq!(
        report_text(&check.stdout),
        expected_report(
            &[API_REPORT, FRONTEND_REPORT, SPEX_REPORT],
            "runners-project: 9 errors, 2 warnings"
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
fn check_files_narrows_ast_grep_findings_and_skips_negated_checks() {
    let (_temp_dir, project) = project_with_rules(&["ast-grep-runner"]);

    let check = run_spex(&project, &["check", "--files", "src/index.ts"], "");

    assert_eq!(
        report_text(&check.stdout),
        expected_report(
            &[FRONTEND_INDEX_REPORT, SPEX_REPORT],
            "runners-project: 1 error, 0 warnings"
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
fn check_package_limits_ast_grep_findings_to_the_named_package() {
    let (_temp_dir, project) = project_with_rules(&["ast-grep-runner"]);

    let check = run_spex(&project, &["check", "--package", "frontend"], "");

    assert_eq!(
        report_text(&check.stdout),
        expected_report(
            &[FRONTEND_REPORT, SPEX_REPORT],
            "runners-project: 6 errors, 2 warnings"
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
