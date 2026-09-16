use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use tempfile::tempdir;

// Snapshot of `spex check` for a freshly initialized copy of the
// `pepperoni-pizza-no-spex` example. The findings are intentional: the example
// ships demo violations.
const EXPECTED_CHECK_REPORT: &str = r#"proj-api
  warn  secret-handling/no-backup-files  (data.backup)
      Backup and scratch files may carry pre-redaction content
  0 errors, 1 warning

proj-frontend
  error  writing-style/no-cyrillic  (README.md:1)
      Requirements and documentation must be in English
  1 error, 0 warnings

proj-spex
  no issues

proj: 1 error, 1 warning
"#;

// Snapshot of `spex coverage` for the freshly initialized no-spex example.
const EXPECTED_COVERAGE_REPORT: &str = r#"Specs (0)
  none found

Gaps (0)
  no active or in-progress specs

in-progress (0), planned (0), active (0), gaps (0)
"#;

// Snapshot of `spex check` for the already-initialized `pepperoni-pizza`
// example
const EXPECTED_WORKFLOW_CHECK_REPORT: &str = r#"pepperoni-pizza-api
  no issues

pepperoni-pizza-frontend
  no issues

pepperoni-pizza-spex
  no issues

pepperoni-pizza: 0 errors, 0 warnings
"#;

// Snapshot of `spex coverage` for the `pepperoni-pizza` example. Its one
// active spec, `delivery-time`, has all of its acceptance criteria checked
// off and is annotated across the contract, impl, and test layers, so it
// reports as fully covered with no gaps; the other three specs are `planned`.
const EXPECTED_WORKFLOW_COVERAGE_REPORT: &str = r#"Specs (4)
  active (1)
  planned (3)
    login-api
    register-page
    register-user-api

Gaps (0)
  none

in-progress (0), planned (3), active (1), gaps (0)
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
        project.join("proj-spex/rules").is_dir(),
        "init did not scaffold rules"
    );

    add_rules(&project);

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

    add_rules(&project);

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

    let init = run_spex(&project, &["init"], "\n\n");
    assert!(init.status.success(), "init failed");

    add_rules(&project);

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

#[test]
fn rules_output_flag_writes_file_matching_stdout() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");

    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/pepperoni-pizza");
    copy_dir(&example, &project);

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

    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/pepperoni-pizza");
    copy_dir(&example, &project);

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

// `/bin/sh` echo interprets the literal `\b` in writing-style.sh into backspace
// bytes; drop them so snapshots stay portable across shells.
fn report_text(stdout: &[u8]) -> String {
    String::from_utf8_lossy(stdout).replace('\u{8}', "")
}

fn run_spex(dir: &Path, args: &[&str], stdin: &str) -> std::process::Output {
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
// renames its repositories to follow naming-convention
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

// `spex init` leaves every repository's `rules` empty (that's `spex-setup`'s
// job). Mimic what the skill would do for the no-spex example, so `check`
// exercises the demo violations it ships.
fn add_rules(project: &Path) {
    let config_path = project.join("proj-spex/spex.yaml");
    let config = fs::read_to_string(&config_path).expect("read spex.yaml");
    let patched = config
        .replace(
            "  - repository_name: proj-api\n    repository_short_name: api\n    path: ../proj-api\n    rules: []\n",
            "  - repository_name: proj-api\n    repository_short_name: api\n    path: ../proj-api\n    rules:\n      - secret-handling\n",
        )
        .replace(
            "  - repository_name: proj-frontend\n    repository_short_name: frontend\n    path: ../proj-frontend\n    rules: []\n",
            "  - repository_name: proj-frontend\n    repository_short_name: frontend\n    path: ../proj-frontend\n    rules:\n      - writing-style\n",
        );
    assert_ne!(
        patched, config,
        "expected repositories not found in spex.yaml"
    );
    fs::write(&config_path, patched).expect("write spex.yaml");
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
