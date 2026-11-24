use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

use crate::common::{
    assert_fails_mentioning, canonical_display, dummy_project_fixture, run_spex, temp_project,
};

fn report(project: &Path, args: &[&str]) -> (std::process::ExitStatus, Value) {
    let mut command = vec!["doctor", "--format", "json"];
    command.extend_from_slice(args);
    let result = run_spex(project, &command, "");
    let value = serde_json::from_slice(&result.stdout).unwrap_or_else(|_| {
        panic!(
            "expected JSON diagnostic\n{}",
            String::from_utf8_lossy(&result.stderr)
        )
    });
    (result.status, value)
}

#[test]
fn doctor_discloses_cli_and_missing_package_for_an_older_config() {
    let (_temp, project) = temp_project(&dummy_project_fixture());
    let config = project.join("dummy-project-spex/spex.yaml");
    let raw = fs::read_to_string(&config).expect("config");
    let legacy = raw.replace("project_short_name: dmy\n", "") + "rules_unused: []\n";
    fs::write(&config, &legacy).expect("write legacy config");
    fs::rename(
        project.join("dummy-project-frontend"),
        project.join("frontend-unavailable"),
    )
    .expect("hide package directory");

    let (status, diagnostic) = report(&project, &[]);

    assert_eq!(status.code(), Some(1));
    assert_eq!(diagnostic["schema_version"], 1);
    assert_eq!(diagnostic["configured_version"], "0.8.8");
    assert_eq!(diagnostic["cli_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(diagnostic["version_matches"], false);
    assert_eq!(diagnostic["config_path"], canonical_display(&config));
    assert_eq!(
        diagnostic["migration_guide"],
        "spex update --migration-guide 0.8.8"
    );
    let issues = diagnostic["config_issues"].to_string();
    assert!(issues.contains("project_short_name"), "{issues}");
    assert!(issues.contains("rules_unused"), "{issues}");
    let frontend = diagnostic["packages"]
        .as_array()
        .expect("package diagnostics")
        .iter()
        .find(|package| package["name"] == "dummy-project-frontend")
        .expect("frontend");
    assert_eq!(frontend["present"], false);
    assert!(frontend["git"]["head"].is_null());
    assert_eq!(
        fs::read_to_string(config).expect("rewritten config"),
        legacy
    );
}

#[test]
fn explicit_selection_disambiguates_a_workspace() {
    let (_temp, project) = temp_project(&dummy_project_fixture());
    let rival = project.join("other-spex");
    fs::create_dir(&rival).expect("other spex dir");
    fs::write(
        rival.join("spex.yaml"),
        "project_name: other\npackages: []\n",
    )
    .expect("write other config");

    assert_fails_mentioning(
        &run_spex(&project, &["doctor"], ""),
        "ambiguous spex project",
    );
    let (status, diagnostic) = report(&project.join("dummy-project-api/src"), &[]);
    assert!(status.success(), "{diagnostic}");
    assert_eq!(diagnostic["project"], "dummy-project");

    for path in ["dummy-project-spex", "dummy-project-spex/spex.yaml"] {
        let (status, diagnostic) = report(&project, &["--project", path]);
        assert!(status.success(), "{diagnostic}");
        assert_eq!(diagnostic["project"], "dummy-project");
    }
    assert_fails_mentioning(
        &run_spex(&project, &["--project", "unavailable", "doctor"], ""),
        "--project must point",
    );
}

fn git(path: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(path)
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
        ])
        .args(args)
        .output()
        .expect("run Git");
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("Git stdout")
        .trim()
        .to_string()
}

fn commit_fixture(path: &Path) -> String {
    git(path, &["init", "--quiet"]);
    git(path, &["add", "."]);
    git(path, &["commit", "--quiet", "-m", "Fixture"]);
    git(path, &["rev-parse", "HEAD"])
}

#[test]
fn revisions_detect_wrong_and_dirty_checkouts_before_checks() {
    let (temp, project) = temp_project(&dummy_project_fixture());
    let package = project.join("dummy-project-api");
    let spec = project.join("dummy-project-spex");
    let expected = commit_fixture(&package);
    let spec_revision = commit_fixture(&spec);
    commit_fixture(&project.join("dummy-project-frontend"));

    let capture = run_spex(&project, &["doctor", "--revisions"], "");
    assert!(
        capture.status.success(),
        "{}",
        String::from_utf8_lossy(&capture.stderr)
    );
    let manifest = serde_json::from_slice::<Value>(&capture.stdout).expect("revision manifest");
    assert_eq!(manifest["project_revision"], spec_revision);
    assert_eq!(manifest["packages"]["dummy-project-api"], expected);
    assert_eq!(
        manifest["packages"].as_object().expect("package set").len(),
        3
    );
    let lock: PathBuf = temp.path().join("revisions.json");
    fs::write(&lock, capture.stdout).expect("save manifest");
    let lock = lock.to_str().expect("manifest path");
    let (status, report_value) = report(&project, &["--verify-revisions", lock]);
    assert!(status.success(), "{report_value}");
    assert_eq!(report_value["revision_issues"], serde_json::json!([]));

    fs::write(package.join("untracked"), "fixture change").expect("dirty package");
    let (status, report_value) = report(&project, &["--verify-revisions", lock]);
    assert_eq!(status.code(), Some(1));
    assert!(
        report_value["revision_issues"]
            .to_string()
            .contains("dirty")
    );
    git(&package, &["add", "untracked"]);
    git(&package, &["commit", "--quiet", "-m", "New revision"]);

    let (status, report_value) = report(&project, &["--verify-revisions", lock]);
    assert_eq!(status.code(), Some(1));
    assert!(
        report_value["revision_issues"]
            .to_string()
            .contains(&expected)
    );

    for command in ["check", "coverage"] {
        let result = run_spex(&project, &[command, "--require-revisions", lock], "");
        assert_fails_mentioning(&result, "does not match the workspace");
        assert!(
            result.stdout.is_empty(),
            "scan ran before revision validation"
        );
    }
}

#[test]
fn manifest_refuses_an_enclosing_repo_as_a_multi_repo_package() {
    let (_temp, project) = temp_project(&dummy_project_fixture());
    commit_fixture(&project);

    let output = run_spex(&project, &["doctor", "--revisions"], "");

    assert_fails_mentioning(&output, "expected Git root");
    assert_fails_mentioning(&output, "dummy-project-api");
    assert!(output.stdout.is_empty());
}

#[test]
fn malformed_yaml_has_a_structured_diagnostic() {
    let (_temp, project) = temp_project(&dummy_project_fixture());
    let config = project.join("dummy-project-spex/spex.yaml");
    fs::write(config, "project_name: [unterminated").expect("write corrupt fixture");

    let (status, value) = report(&project, &[]);

    assert_eq!(status.code(), Some(1));
    assert!(
        value["config_issues"]
            .to_string()
            .contains("could not parse")
    );
    assert_eq!(value["packages"], serde_json::json!([]));
}
