use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use tempfile::tempdir;

// Snapshot of `spex check` for a freshly initialized copy of the
// `pepperoni-pizza-no-spex` example. The findings are intentional: the example
// ships demo violations.
const EXPECTED_CHECK_REPORT: &str = r#"proj-api
  warn  TECH-SECRET-HANDLING/no-backup-files  (data.backup)
      Backup and scratch files may carry pre-redaction content
  0 errors, 1 warning

proj-frontend
  error  TECH-WRITING-STYLE/no-cyrillic  (README.md:1)
      Requirements and documentation must be in English
  1 error, 0 warnings

proj-spex
  no issues

workspace
  no issues

proj: 1 error, 1 warning
"#;

// Snapshot of `spex coverage` for the freshly initialized no-spex example.
const EXPECTED_COVERAGE_REPORT: &str = r#"spex coverage · proj

Business specs (0)
  none found

Active spec coverage (0)
  no active specs

proj: no gaps
"#;

// Snapshot of `spex check` for the already-initialized `pepperoni-pizza`
// example
const EXPECTED_WORKFLOW_CHECK_REPORT: &str = r#"pepperoni-pizza-api
  no issues

pepperoni-pizza-frontend
  no issues

pepperoni-pizza-spex
  warn  TECH-WRITING-STYLE/writing  (tutorial-completion.md:46)
      matches forbidden writing pattern
  0 errors, 1 warning

workspace
  no issues

pepperoni-pizza: 0 errors, 1 warning
"#;

// Snapshot of `spex coverage` for the `pepperoni-pizza` example. Its single
// business spec is `active`, has all of its acceptance criteria checked off,
// and is annotated across the contract, impl, and test layers, so it reports
// as fully covered with no gaps.
const EXPECTED_WORKFLOW_COVERAGE_REPORT: &str = r#"spex coverage · pepperoni-pizza

Business specs (1)
  active      1

Active spec coverage (1)
  ✓  BIZ-DELIVERY-TIME  10/10 AC

  1 fully covered · 0 gaps

pepperoni-pizza: no gaps
"#;

#[test]
fn init_then_check_pepperoni_pizza() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");

    copy_no_spex(&project);

    let init = run_spex(&project, &["init"], "\n\n");
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
        project.join("proj-spex/tech-specs").is_dir(),
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

    copy_no_spex(&project);

    let init = run_spex(&project, &["init"], "\n\n");
    assert!(init.status.success(), "init failed");

    // Baseline the frontend's pre-existing no-cyrillic error on README.md. A new
    // error in any other file would still surface.
    let config_path = project.join("proj-spex/spex.yaml");
    let config = fs::read_to_string(&config_path).expect("read spex.yaml");
    let patched = config.replace(
        "  - repository_name: proj-frontend\n",
        "  - repository_name: proj-frontend\n    checks_preexisting_errors:\n      - spec: TECH-WRITING-STYLE\n        check: no-cyrillic\n        files: [README.md]\n",
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

    copy_no_spex(&project);

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

    let config = fs::read_to_string(project.join("proj-spex/spex.yaml")).expect("read spex.yaml");
    assert!(
        config.contains("checks_preexisting_errors:")
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
        Some(0),
        "coverage should exit 0 when every active spec is fully covered"
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

// Copies the no-spex example into `project` and, mirroring tutorial Step 2.1,
// renames its repositories to follow TECH-NAMING-CONVENTION
// (`<project-name>-<repository-short-name>`). The project name defaults to the
// workspace folder name, so the prefix is derived from it.
fn copy_no_spex(project: &Path) {
    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/pepperoni-pizza-no-spex");
    copy_dir(&example, project);

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
