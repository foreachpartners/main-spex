use super::{SPEX_REPORT, expected_report, project_with_rules};
use crate::common::{report_text, run_spex};

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
  3 errors, 2 warnings

"#;
const FRONTEND_REPORT: &str = r#"runners-project-frontend
  error  regexp-runner/no-fixme  (src/index.ts:2)
      matches forbidden pattern
  warn  regexp-runner/require-license-header  (src/util.ts)
      required pattern not found
  1 error, 1 warning

"#;
const FRONTEND_INDEX_REPORT: &str = r#"runners-project-frontend
  error  regexp-runner/no-fixme  (src/index.ts:2)
      matches forbidden pattern
  1 error, 0 warnings

"#;

#[test]
fn check_reports_every_regexp_finding() {
    let (_workspace, project) = project_with_rules(&["regexp-runner"]);

    let check = run_spex(&project, &["check"], "");

    assert_eq!(
        report_text(&check.stdout),
        expected_report(
            &[API_REPORT, FRONTEND_REPORT, SPEX_REPORT],
            "runners-project: 4 errors, 3 warnings"
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
fn check_files_narrows_regexp_findings_and_skips_negated_checks() {
    let (_workspace, project) = project_with_rules(&["regexp-runner"]);

    let check = run_spex(&project, &["check", "--files", "src/index.ts"], "");

    assert_eq!(
        report_text(&check.stdout),
        expected_report(
            &[FRONTEND_INDEX_REPORT],
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
fn check_repo_limits_regexp_findings_to_the_named_repository() {
    let (_workspace, project) = project_with_rules(&["regexp-runner"]);

    let check = run_spex(&project, &["check", "--repo", "frontend"], "");

    assert_eq!(
        report_text(&check.stdout),
        expected_report(&[FRONTEND_REPORT], "runners-project: 1 error, 1 warning"),
        "check --repo report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&check.stderr),
    );
    assert_eq!(
        check.status.code(),
        Some(1),
        "check should exit 1 when errors are present"
    );
}
