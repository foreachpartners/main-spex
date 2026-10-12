use super::{SPEX_REPORT, expected_report, project_with_rules};
use crate::common::{report_text, run_spex};

const API_REPORT: &str = r#"runners-project-api
  warn  glob-runner/no-backup-files  (archive.bak)
      forbidden path
  warn  glob-runner/no-backup-files  (docs/draft.bak)
      forbidden path
  warn  glob-runner/no-backup-files  (old.bak)
      forbidden path
  0 errors, 3 warnings

"#;
const FRONTEND_REPORT: &str = r#"runners-project-frontend
  error  glob-runner/require-readme
      required path not found: README.md
  1 error, 0 warnings

"#;
const FRONTEND_NO_ISSUES_REPORT: &str = r#"runners-project-frontend
  no issues

"#;

#[test]
fn check_reports_every_glob_finding() {
    let (_workspace, project) = project_with_rules(&["glob-runner"]);

    let check = run_spex(&project, &["check"], "");

    assert_eq!(
        report_text(&check.stdout),
        expected_report(
            &[API_REPORT, FRONTEND_REPORT, SPEX_REPORT],
            "runners-project: 1 error, 3 warnings"
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
fn check_files_narrows_glob_findings_and_skips_negated_checks() {
    let (_workspace, project) = project_with_rules(&["glob-runner"]);

    let check = run_spex(&project, &["check", "--files", "src/index.ts"], "");

    assert_eq!(
        report_text(&check.stdout),
        expected_report(
            &[FRONTEND_NO_ISSUES_REPORT],
            "runners-project: 0 errors, 0 warnings"
        ),
        "check --files report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&check.stderr),
    );
    assert!(check.status.success(), "no errors, so check should exit 0");
}

#[test]
fn check_repo_limits_glob_findings_to_the_named_repository() {
    let (_workspace, project) = project_with_rules(&["glob-runner"]);

    let check = run_spex(&project, &["check", "--repo", "frontend"], "");

    assert_eq!(
        report_text(&check.stdout),
        expected_report(&[FRONTEND_REPORT], "runners-project: 1 error, 0 warnings"),
        "check --repo report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&check.stderr),
    );
    assert_eq!(
        check.status.code(),
        Some(1),
        "check should exit 1 when errors are present"
    );
}
