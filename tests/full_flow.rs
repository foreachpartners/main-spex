use std::fs;
use std::path::Path;

use tempfile::tempdir;

mod common;
use common::{copy_dir, dummy_project_fixture, report_text, run_spex, with_version_header};

// Snapshot of `spex check` for a freshly initialized copy of the
// `no-spex-project` fixture. The findings are intentional: the fixture ships
// demo violations.
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

// Snapshot of `spex coverage` for the freshly initialized no-spex-project fixture.
const EXPECTED_COVERAGE_REPORT: &str = r#"Specs (0)
  none found

Gaps (0)
  no active or in-progress specs

planned (0), in-progress (0), active (0), gaps (0)
"#;

// Snapshot of `spex check` for the dummy-project fixture, whose specs cover
// every lifecycle state.
const EXPECTED_DUMMY_CHECK_REPORT: &str = r#"dummy-project-api
  no issues

dummy-project-frontend
  no issues

dummy-project-spex
  no issues

dummy-project: 0 errors, 0 warnings
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

#[test]
fn preexisting_errors_baseline_suppresses_matching_error() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");

    copy_no_spex(&project);

    let init = run_spex(&project, &["init"], "\n");
    assert!(init.status.success(), "init failed");

    add_repositories(&project);

    // Baseline the frontend's pre-existing no-cyrillic error on README.md. A new
    // error in any other file would still surface.
    let config_path = project.join("proj-spex/spex.yaml");
    let config = fs::read_to_string(&config_path).expect("read spex.yaml");
    let patched = config.replace(
        "  - repository_name: proj-frontend\n    repository_short_name: frontend\n    path: ../proj-frontend\n    rules:\n      - writing-style\n",
        "  - repository_name: proj-frontend\n    repository_short_name: frontend\n    path: ../proj-frontend\n    rules:\n      - writing-style\n    ignored_errors:\n      - rule_id: writing-style\n        check: no-cyrillic\n        files: [README.md]\n",
    );
    assert_ne!(
        patched, config,
        "frontend repository not found in spex.yaml"
    );
    fs::write(&config_path, patched).expect("write spex.yaml");

    let check = run_spex(&project, &["check"], "");
    let report = report_text(&check.stdout);
    assert!(
        !report.contains("error  writing-style/no-cyrillic"),
        "baseline did not suppress the pre-existing error\n{report}",
    );
    assert!(
        report.contains("proj: 0 errors"),
        "expected no remaining errors\n{report}",
    );
    assert!(
        check.status.success(),
        "check should exit 0 once the only error is baselined (exit {:?})",
        check.status.code(),
    );
}

#[test]
fn fill_preexisting_errors_baselines_then_suppresses() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");

    copy_no_spex(&project);

    let init = run_spex(&project, &["init"], "\n");
    assert!(init.status.success(), "init failed");

    add_repositories(&project);

    // The freshly initialized fixture has one error: the frontend README.
    let fill = run_spex(&project, &["check", "--fill-preexisting-errors"], "");
    assert!(
        fill.status.success(),
        "--fill-preexisting-errors should exit 0 (exit {:?})\nstderr:\n{}",
        fill.status.code(),
        String::from_utf8_lossy(&fill.stderr),
    );

    let config = fs::read_to_string(project.join("proj-spex/spex.yaml")).expect("read spex.yaml");
    assert!(
        config.contains("ignored_errors:")
            && config.contains("check: no-cyrillic")
            && config.contains("- README.md"),
        "baseline was not written into spex.yaml:\n{config}",
    );

    // Running it again is idempotent: nothing new to baseline.
    let refill = run_spex(&project, &["check", "--fill-preexisting-errors"], "");
    let config_after =
        fs::read_to_string(project.join("proj-spex/spex.yaml")).expect("reread spex.yaml");
    assert_eq!(config, config_after, "second fill changed spex.yaml");
    assert!(refill.status.success());

    // With the baseline in place, the error is suppressed and check passes.
    let check = run_spex(&project, &["check"], "");
    let report = report_text(&check.stdout);
    assert!(
        !report.contains("error  writing-style/no-cyrillic"),
        "generated baseline did not suppress the error\n{report}",
    );
    assert!(
        check.status.success(),
        "check should exit 0 after the baseline is generated (exit {:?})",
        check.status.code(),
    );
}

#[test]
fn check_dummy_project_matches_expected() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");

    copy_dir(&dummy_project_fixture(), &project);

    let check = run_spex(&project, &["check"], "");
    assert_eq!(
        report_text(&check.stdout),
        with_version_header(EXPECTED_DUMMY_CHECK_REPORT),
        "check report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&check.stderr),
    );
    assert!(
        check.status.success(),
        "check should exit 0 when there are no errors (exit {:?})",
        check.status.code(),
    );
}

#[test]
fn check_flags_annotation_referencing_rule_id_as_orphan() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");

    copy_dir(&dummy_project_fixture(), &project);
    fs::write(
        project.join("dummy-project-api/src/rule_reference.rs"),
        "// @spec impl no-todo-comments\n",
    )
    .expect("write rule reference");

    let check = run_spex(&project, &["check"], "");
    let report = report_text(&check.stdout);
    assert!(
        report.contains("orphan annotation references unknown spec `no-todo-comments`"),
        "annotation referencing a rule id was not flagged\n{report}",
    );
    assert!(
        !check.status.success(),
        "check should exit non-zero on an orphan annotation",
    );
}

#[test]
fn rules_output_flag_writes_file_matching_stdout() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");

    copy_dir(&dummy_project_fixture(), &project);

    let stdout_run = run_spex(&project, &["rules"], "");
    assert!(
        stdout_run.status.success(),
        "rules failed (exit {:?})\nstderr:\n{}",
        stdout_run.status.code(),
        String::from_utf8_lossy(&stdout_run.stderr),
    );
    let stdout_yaml = String::from_utf8_lossy(&stdout_run.stdout).into_owned();
    assert!(!stdout_yaml.is_empty(), "expected rules on stdout");

    let output_path = workspace.path().join("rules.yaml");
    let file_run = run_spex(
        &project,
        &[
            "rules",
            "--output",
            output_path.to_str().expect("utf8 path"),
        ],
        "",
    );
    assert!(
        file_run.status.success(),
        "rules --output failed (exit {:?})\nstderr:\n{}",
        file_run.status.code(),
        String::from_utf8_lossy(&file_run.stderr),
    );
    assert!(
        file_run.stdout.is_empty(),
        "rules --output should not print to stdout"
    );

    let file_yaml = fs::read_to_string(&output_path).expect("read --output file");
    assert_eq!(
        file_yaml, stdout_yaml,
        "--output file contents should match stdout output"
    );
}

#[test]
fn specs_output_flag_writes_file_matching_stdout() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");

    copy_dir(&dummy_project_fixture(), &project);

    let stdout_run = run_spex(&project, &["specs"], "");
    assert!(
        stdout_run.status.success(),
        "specs failed (exit {:?})\nstderr:\n{}",
        stdout_run.status.code(),
        String::from_utf8_lossy(&stdout_run.stderr),
    );
    let stdout_yaml = String::from_utf8_lossy(&stdout_run.stdout).into_owned();
    assert!(!stdout_yaml.is_empty(), "expected specs on stdout");

    let output_path = workspace.path().join("specs.yaml");
    let file_run = run_spex(
        &project,
        &[
            "specs",
            "--output",
            output_path.to_str().expect("utf8 path"),
        ],
        "",
    );
    assert!(
        file_run.status.success(),
        "specs --output failed (exit {:?})\nstderr:\n{}",
        file_run.status.code(),
        String::from_utf8_lossy(&file_run.stderr),
    );
    assert!(
        file_run.stdout.is_empty(),
        "specs --output should not print to stdout"
    );

    let file_yaml = fs::read_to_string(&output_path).expect("read --output file");
    assert_eq!(
        file_yaml, stdout_yaml,
        "--output file contents should match stdout output"
    );
}

// Copies the no-spex-project fixture into `project` and, mirroring tutorial
// Step 2.1, renames its repositories to follow naming-convention
// (`<project-name>-<repository-short-name>`). The project name defaults to the
// workspace folder name, so the prefix is derived from it.
fn copy_no_spex(project: &Path) {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/no-spex-project");
    copy_dir(&fixture, project);

    let project_name = project
        .file_name()
        .and_then(|name| name.to_str())
        .expect("project folder name")
        .to_string();
    for short in ["api", "frontend"] {
        fs::rename(
            project.join(short),
            project.join(format!("{project_name}-{short}")),
        )
        .expect("rename repository to follow naming convention");
    }
}

// `spex init` lists only the spex repository (adding the other repositories is
// `spex-setup`'s job). Mimic what the skill would do for the no-spex-project
// fixture, so `check` exercises the demo violations it ships.
fn add_repositories(project: &Path) {
    let config_path = project.join("proj-spex/spex.yaml");
    let mut config = fs::read_to_string(&config_path).expect("read spex.yaml");
    config.push_str(
        "  - repository_name: proj-api\n    repository_short_name: api\n    path: ../proj-api\n    rules:\n      - secret-handling\n  - repository_name: proj-frontend\n    repository_short_name: frontend\n    path: ../proj-frontend\n    rules:\n      - writing-style\n",
    );
    fs::write(&config_path, config).expect("write spex.yaml");
}
