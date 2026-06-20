use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use tempfile::tempdir;

// Snapshot of `spex check` for a freshly initialized copy of the
// `pepperoni-pizza-no-spex` example. The findings are intentional: the example
// ships demo violations and the workspace is not a git repository, so the
// commit-format checks warn. The trailing `workspace` section reports the
// workspace-scoped checks that run once across every repository. Update this
// when the example or bundled templates change.
const EXPECTED_CHECK_REPORT: &str = r#"api
  warn  TECH-COMMIT-MESSAGE-FORMAT/commit-conventional-format
      Not a git repository — cannot verify commit message format
  warn  TECH-COMMIT-MESSAGE-FORMAT/commit-spec-id-trace
      Not a git repository — cannot verify commit message format
  warn  TECH-SECRET-HANDLING/no-backup-files  (data.backup)
      Backup and scratch files may carry pre-redaction content
  0 errors, 3 warnings

frontend
  warn  TECH-COMMIT-MESSAGE-FORMAT/commit-conventional-format
      Not a git repository — cannot verify commit message format
  warn  TECH-COMMIT-MESSAGE-FORMAT/commit-spec-id-trace
      Not a git repository — cannot verify commit message format
  error  TECH-WRITING-STYLE/no-cyrillic  (README.md:1)
      Requirements and documentation must be in English
  1 error, 2 warnings

spex
  warn  TECH-COMMIT-MESSAGE-FORMAT/commit-conventional-format
      Not a git repository — cannot verify commit message format
  warn  TECH-COMMIT-MESSAGE-FORMAT/commit-spec-id-trace
      Not a git repository — cannot verify commit message format
  0 errors, 2 warnings

workspace
  no issues

proj: 1 error, 7 warnings
"#;

// Snapshot of `spex coverage` for the freshly initialized no-spex example.
// `init` does not scaffold any business specs, so coverage reports the empty
// project and exits successfully.
const EXPECTED_COVERAGE_REPORT: &str = r#"spex coverage · proj

Business specs (0)
  none found

Active spec coverage (0)
  no active specs

proj: no gaps
"#;

// Snapshot of `spex check` for the already-initialized `pepperoni-pizza`
// example. Like the no-spex run the workspace is not a git repository, so the
// commit-format checks warn; the example also ships demo writing-style
// violations in its frontend README. No errors are present, so check succeeds.
const EXPECTED_WORKFLOW_CHECK_REPORT: &str = r#"api
  warn  TECH-COMMIT-MESSAGE-FORMAT/commit-conventional-format
      Not a git repository — cannot verify commit message format
  warn  TECH-COMMIT-MESSAGE-FORMAT/commit-spec-id-trace
      Not a git repository — cannot verify commit message format
  warn  TECH-SECRET-HANDLING/no-backup-files  (data.backup)
      Backup and scratch files may carry pre-redaction content
  0 errors, 3 warnings

contracts
  warn  TECH-COMMIT-MESSAGE-FORMAT/commit-conventional-format
      Not a git repository — cannot verify commit message format
  warn  TECH-COMMIT-MESSAGE-FORMAT/commit-spec-id-trace
      Not a git repository — cannot verify commit message format
  0 errors, 2 warnings

frontend
  warn  TECH-COMMIT-MESSAGE-FORMAT/commit-conventional-format
      Not a git repository — cannot verify commit message format
  warn  TECH-COMMIT-MESSAGE-FORMAT/commit-spec-id-trace
      Not a git repository — cannot verify commit message format
  warn  TECH-WRITING-STYLE/no-cyrillic  (README.md:1)
      Requirements and documentation must be in English
  warn  TECH-WRITING-STYLE/readme-has-title
      README.md must start with a '# ' title
  0 errors, 4 warnings

spex
  warn  TECH-COMMIT-MESSAGE-FORMAT/commit-conventional-format
      Not a git repository — cannot verify commit message format
  warn  TECH-COMMIT-MESSAGE-FORMAT/commit-spec-id-trace
      Not a git repository — cannot verify commit message format
  0 errors, 2 warnings

workspace
  no issues

pepperoni-pizza: 0 errors, 11 warnings
"#;

// Snapshot of `spex coverage` for the `pepperoni-pizza` example. Its single
// business spec is `active` and annotated across the contract, impl, and test
// layers, so it reports as fully covered with no gaps.
const EXPECTED_WORKFLOW_COVERAGE_REPORT: &str = r#"spex coverage · pepperoni-pizza

Business specs (1)
  active      1

Active spec coverage (1)
  ✗  BIZ-LIST-PIZZA-MENU  missing: test

  0 fully covered · 1 gap

pepperoni-pizza: 1 gap
"#;

#[test]
fn init_then_check_pepperoni_pizza() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");

    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/pepperoni-pizza-no-spex");
    copy_dir(&example, &project);

    let init = run_spex(&project, &["init"], "\n\n");
    assert!(
        init.status.success(),
        "init failed (exit {:?})\nstdout:\n{}\nstderr:\n{}",
        init.status.code(),
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr),
    );
    assert!(
        project.join("spex/spex.yaml").is_file(),
        "init did not write spex/spex.yaml"
    );
    assert!(
        project.join("spex/tech-specs").is_dir(),
        "init did not scaffold tech-specs"
    );

    let check = run_spex(&project, &["check"], "");
    assert_eq!(
        report_text(&check.stdout),
        EXPECTED_CHECK_REPORT,
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
        EXPECTED_COVERAGE_REPORT,
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

    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/pepperoni-pizza-no-spex");
    copy_dir(&example, &project);

    let init = run_spex(&project, &["init"], "\n\n");
    assert!(init.status.success(), "init failed");

    // Baseline the frontend's pre-existing no-cyrillic error on README.md. A new
    // error in any other file would still surface.
    let config_path = project.join("spex/spex.yaml");
    let config = fs::read_to_string(&config_path).expect("read spex.yaml");
    let patched = config.replace(
        "  - repository_name: frontend\n",
        "  - repository_name: frontend\n    checks_preexisting_errors:\n      - spec: TECH-WRITING-STYLE\n        check: no-cyrillic\n        files: [README.md]\n",
    );
    assert_ne!(
        patched, config,
        "frontend repository not found in spex.yaml"
    );
    fs::write(&config_path, patched).expect("write spex.yaml");

    let check = run_spex(&project, &["check"], "");
    let report = report_text(&check.stdout);
    assert!(
        !report.contains("error  TECH-WRITING-STYLE/no-cyrillic"),
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

    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/pepperoni-pizza-no-spex");
    copy_dir(&example, &project);

    let init = run_spex(&project, &["init"], "\n\n");
    assert!(init.status.success(), "init failed");

    // The freshly initialized example has one error: the frontend README.
    let fill = run_spex(&project, &["check", "--fill-preexisting-errors"], "");
    assert!(
        fill.status.success(),
        "--fill-preexisting-errors should exit 0 (exit {:?})\nstderr:\n{}",
        fill.status.code(),
        String::from_utf8_lossy(&fill.stderr),
    );

    let config = fs::read_to_string(project.join("spex/spex.yaml")).expect("read spex.yaml");
    assert!(
        config.contains("checks_preexisting_errors:")
            && config.contains("check: no-cyrillic")
            && config.contains("- README.md"),
        "baseline was not written into spex.yaml:\n{config}",
    );

    // Running it again is idempotent: nothing new to baseline.
    let refill = run_spex(&project, &["check", "--fill-preexisting-errors"], "");
    let config_after =
        fs::read_to_string(project.join("spex/spex.yaml")).expect("reread spex.yaml");
    assert_eq!(config, config_after, "second fill changed spex.yaml");
    assert!(refill.status.success());

    // With the baseline in place, the error is suppressed and check passes.
    let check = run_spex(&project, &["check"], "");
    let report = report_text(&check.stdout);
    assert!(
        !report.contains("error  TECH-WRITING-STYLE/no-cyrillic"),
        "generated baseline did not suppress the error\n{report}",
    );
    assert!(
        check.status.success(),
        "check should exit 0 after the baseline is generated (exit {:?})",
        check.status.code(),
    );
}

#[test]
fn workflow() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");

    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/pepperoni-pizza");
    copy_dir(&example, &project);

    let check = run_spex(&project, &["check"], "");
    assert_eq!(
        report_text(&check.stdout),
        EXPECTED_WORKFLOW_CHECK_REPORT,
        "check report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&check.stderr),
    );
    assert!(
        check.status.success(),
        "check should exit 0 when there are no errors (exit {:?})",
        check.status.code(),
    );

    let coverage = run_spex(&project, &["coverage"], "");
    assert_eq!(
        report_text(&coverage.stdout),
        EXPECTED_WORKFLOW_COVERAGE_REPORT,
        "coverage report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&coverage.stderr),
    );
    assert_eq!(
        coverage.status.code(),
        Some(1),
        "coverage should exit 1 when there is a gap"
    );
}

// `/bin/sh` echo interprets the literal `\b` in writing-style.sh into backspace
// bytes; drop them so snapshots stay portable across shells.
fn report_text(stdout: &[u8]) -> String {
    String::from_utf8_lossy(stdout).replace('\u{8}', "")
}

fn run_spex(dir: &Path, args: &[&str], stdin: &str) -> std::process::Output {
    // The TECH-SPEX-CONFORMANCE check shells out to `spex check-schema`, so the
    // binary under test must be discoverable on PATH.
    let bin = Path::new(env!("CARGO_BIN_EXE_spex"));
    let bin_dir = bin.parent().expect("binary directory");
    let path = match std::env::var_os("PATH") {
        Some(existing) => {
            let mut dirs = vec![bin_dir.to_path_buf()];
            dirs.extend(std::env::split_paths(&existing));
            std::env::join_paths(dirs).expect("join PATH")
        }
        None => bin_dir.as_os_str().to_owned(),
    };

    let mut child = Command::new(bin)
        .args(args)
        .current_dir(dir)
        .env("PATH", path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn spex");

    child
        .stdin
        .take()
        .expect("open spex stdin")
        .write_all(stdin.as_bytes())
        .expect("write spex stdin");

    child.wait_with_output().expect("wait for spex")
}

fn copy_dir(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).expect("create destination dir");
    for entry in fs::read_dir(src).expect("read source dir") {
        let entry = entry.expect("read dir entry");
        let from = entry.path();
        let to = dst.join(entry.file_name());
        let file_type = entry.file_type().expect("read file type");
        if file_type.is_symlink() {
            let target = fs::read_link(&from).expect("read symlink target");
            std::os::unix::fs::symlink(target, &to).expect("create symlink");
        } else if file_type.is_dir() {
            // Skip build artifacts so copies stay deterministic regardless of
            // whatever a developer has built locally inside an example.
            let name = entry.file_name();
            if matches!(
                name.to_str(),
                Some("node_modules") | Some("target") | Some("dist")
            ) {
                continue;
            }
            copy_dir(&from, &to);
        } else {
            fs::copy(&from, &to).expect("copy file");
        }
    }
}
