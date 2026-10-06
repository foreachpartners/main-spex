#![allow(dead_code)]

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

// `/bin/sh` echo interprets the literal `\b` in writing-style.sh into backspace
// bytes; drop them so snapshots stay portable across shells.
pub fn report_text(stdout: &[u8]) -> String {
    String::from_utf8_lossy(stdout).replace('\u{8}', "")
}

// Every report opens with the running CLI's version, e.g. `spex 0.10.0`,
// followed by a blank line; the snapshots hold the report body only.
pub fn with_version_header(body: &str) -> String {
    format!("spex {}\n\n{body}", env!("CARGO_PKG_VERSION"))
}

pub fn run_spex(dir: &Path, args: &[&str], stdin: &str) -> std::process::Output {
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

// The dummy `dummy-project` fixture: a lean workspace (few specs, no real
// app code beyond `@spec` annotations) used by tests that need a valid
// project but don't care about its content, plus the coverage-focused tests
// that do.
pub fn dummy_project_fixture() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/dummy-project")
}

pub fn runners_project_fixture() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/runners-project")
}

pub fn temp_project(fixture: &Path) -> (tempfile::TempDir, std::path::PathBuf) {
    let workspace = tempfile::tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");
    copy_dir(fixture, &project);
    (workspace, project)
}

pub fn listed_ids(stdout: &[u8]) -> Vec<String> {
    let mut ids: Vec<String> = String::from_utf8_lossy(stdout)
        .lines()
        .filter_map(|line| line.strip_prefix("- id: "))
        .map(str::to_string)
        .collect();
    ids.sort();
    ids
}

pub fn assert_fails_mentioning(output: &std::process::Output, needle: &str) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "expected a non-zero exit\nstdout:\n{}",
        String::from_utf8_lossy(&output.stdout),
    );
    assert!(
        stderr.contains(needle),
        "stderr did not mention `{needle}`\nstderr:\n{stderr}",
    );
}

pub fn copy_dir(src: &Path, dst: &Path) {
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
