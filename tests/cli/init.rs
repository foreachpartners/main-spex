use tempfile::tempdir;

use crate::common::{add_repositories, copy_no_spex, report_text, run_spex, with_version_header};

const EXPECTED_CHECK_REPORT: &str = r#"proj-spex
  no issues

proj-api
  warn  secret-handling/no-backup-files  (data.backup)
      Backup and scratch files may carry pre-redaction content
  0 errors, 1 warning

proj-frontend
  error  writing-style/no-cyrillic  (README.md:1)
      Requirements and documentation must be in English
  1 error, 0 warnings

proj: 1 error, 1 warning
"#;

const EXPECTED_COVERAGE_REPORT: &str = r#"Specs (0)
  none found

Gaps (0)
  no active or in-progress specs

planned (0), in-progress (0), active (0), gaps (0)
"#;

#[test]
fn init_then_check_no_spex_project() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");

    copy_no_spex(&project);

    let init = run_spex(&project, &["init"], "\n");
    assert!(
        init.status.success(),
        "init failed (exit {:?})\nstdout:\n{}\nstderr:\n{}",
        init.status.code(),
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr),
    );
    assert!(
        project.join("proj-spex/spex.yaml").is_file(),
        "init did not write proj-spex/spex.yaml"
    );
    assert!(
        project.join("proj-spex/rules").is_dir(),
        "init did not scaffold rules"
    );

    add_repositories(&project);

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

    let coverage = run_spex(&project, &["coverage"], "");
    assert_eq!(
        report_text(&coverage.stdout),
        with_version_header(EXPECTED_COVERAGE_REPORT),
        "coverage report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&coverage.stderr),
    );
    assert!(
        coverage.status.success(),
        "coverage should exit 0 when there are no gaps (exit {:?})",
        coverage.status.code(),
    );
}
