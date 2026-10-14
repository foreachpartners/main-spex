use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn report_text(stdout: &[u8]) -> String {
    String::from_utf8_lossy(stdout).replace('\u{8}', "")
}

pub fn with_version_header(body: &str) -> String {
    format!("spex {}\n\n{body}", env!("CARGO_PKG_VERSION"))
}

pub fn run_spex(dir: &Path, args: &[&str], stdin: &str) -> std::process::Output {
    run_spex_with_env(dir, args, stdin, &[])
}

pub fn run_spex_with_env(
    dir: &Path,
    args: &[&str],
    stdin: &str,
    envs: &[(&str, &str)],
) -> std::process::Output {
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
        .env_remove("RUST_BACKTRACE")
        .env_remove("RUST_LIB_BACKTRACE")
        .envs(envs.iter().copied())
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

pub fn dummy_project_fixture() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/dummy-project")
}

pub fn runners_project_fixture() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/runners-project")
}

pub fn script_env_project_fixture() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/script-env-project")
}

pub fn temp_project(fixture: &Path) -> (tempfile::TempDir, std::path::PathBuf) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    copy_project(fixture, &project);
    (temp_dir, project)
}

pub fn copy_project(fixture: &Path, project: &Path) {
    copy_dir(fixture, project);
    let bundled_schemas = Path::new(env!("CARGO_MANIFEST_DIR")).join("templates/schemas");
    for entry in fs::read_dir(project).expect("read project dir") {
        let path = entry.expect("read dir entry").path();
        let is_spex_dir = path.is_dir()
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name == "spex" || name.ends_with("-spex"));
        if is_spex_dir {
            copy_dir(&bundled_schemas, &path.join("schemas"));
        }
    }
}

pub fn canonical_display(path: &Path) -> String {
    path.canonicalize()
        .expect("canonical path")
        .display()
        .to_string()
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

pub fn copy_no_spex(project: &Path) {
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
        .expect("rename package to follow naming convention");
    }
}

pub fn add_packages(project: &Path) {
    let config_path = project.join("proj-spex/spex.yaml");
    let mut config = fs::read_to_string(&config_path).expect("read spex.yaml");
    config.push_str(
        "  - package_name: proj-api\n    package_short_name: api\n    path: ../proj-api\n    rules:\n      - secret-handling\n  - package_name: proj-frontend\n    package_short_name: frontend\n    path: ../proj-frontend\n    rules:\n      - writing-style\n",
    );
    fs::write(&config_path, config).expect("write spex.yaml");
}
