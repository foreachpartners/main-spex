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

proj: 1 error, 9 warnings
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
    // `/bin/sh` echo interprets the literal `\b` in writing-style.sh into
    // backspace bytes; drop them so the snapshot stays portable across shells.
    let report = String::from_utf8_lossy(&check.stdout).replace('\u{8}', "");
    assert_eq!(
        report,
        EXPECTED_CHECK_REPORT,
        "check report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&check.stderr),
    );
    assert_eq!(
        check.status.code(),
        Some(1),
        "check should exit 1 when errors are present"
    );
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
        if entry.file_type().expect("read file type").is_dir() {
            copy_dir(&from, &to);
        } else {
            fs::copy(&from, &to).expect("copy file");
        }
    }
}
