use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use super::common::{
    dummy_project_fixture, report_text, run_spex_with_env, take_rules, temp_project,
};

const CONTRACT: &str = "+openapi: 3.0.3
info:
  title: Example API
  version: '1.0'
paths: {}
components:
  schemas:
    Item:
      $ref: './model.yaml#/Item'
";

const OASDIFF: &str = r#"#!/bin/sh
set -eu
[ "$#" -eq 6 ]
[ "$1" = breaking ]
[ "$4" = --fail-on=WARN ]
[ "$5" = --fetch=false ]
[ "$6" = --open=false ]
[ "$(cat "$(dirname -- "$2")/model.yaml")" = baseline ]
[ "$(cat "$(dirname -- "$3")/model.yaml")" = current ]
case "$OASDIFF_RESULT" in
  breaking) printf 'the GET operation was removed\n'; exit 1 ;;
  malformed) printf 'cannot parse the baseline document\n'; exit 2 ;;
esac
"#;

fn checkout_contract(root: &Path, marker: &str) {
    let directory = root.join("contracts");
    fs::create_dir_all(&directory).expect("create contracts directory");
    fs::write(directory.join("client.openapi.yaml"), CONTRACT).expect("write contract");
    fs::write(directory.join("model.yaml"), marker).expect("write referenced schema");
}

fn fixture() -> (tempfile::TempDir, PathBuf, PathBuf, PathBuf) {
    let (temp, project) = temp_project(&dummy_project_fixture());
    let spex = project.join("dummy-project-spex");
    let baseline = temp.path().join("baseline checkout");
    let bin = temp.path().join("bin");
    fs::create_dir(&bin).expect("create tool directory");
    let oasdiff = bin.join("oasdiff");
    fs::write(&oasdiff, OASDIFF).expect("write tool fixture");
    fs::set_permissions(oasdiff, fs::Permissions::from_mode(0o755)).expect("enable tool fixture");

    checkout_contract(&spex, "current");
    checkout_contract(&baseline, "baseline");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(&spex)
            .status()
            .expect("initialize test Git checkout")
            .success()
    );
    take_rules(&spex, &["openapi-compatibility"]);
    let config_path = spex.join("spex.yaml");
    let config = fs::read_to_string(&config_path).expect("read config");
    fs::write(
        config_path,
        config
            .replace(
                "path: ./contracts.md",
                "path: contracts/client.openapi.yaml",
            )
            .replace(
                "      - no-todo-comments",
                "      - no-todo-comments\n      - openapi-compatibility",
            ),
    )
    .expect("attach rule to the declaring package");
    (temp, project, baseline, bin)
}

fn check(project: &Path, baseline: &str, bin: &Path, outcome: &str) -> Output {
    let path = env::var_os("PATH").expect("PATH");
    let tool_path = env::join_paths(std::iter::once(bin.to_owned()).chain(env::split_paths(&path)))
        .expect("tool PATH");
    run_spex_with_env(
        project,
        &["check", "--package", "api"],
        "",
        &[
            ("PATH", tool_path.to_str().expect("UTF-8 PATH")),
            ("SPEX_OPENAPI_BASELINE_DIR", baseline),
            ("OASDIFF_RESULT", outcome),
        ],
    )
}

#[test]
fn compatibility_compares_the_declared_cross_repo_contract_at_its_real_paths() {
    let (_temp, project, baseline, bin) = fixture();
    let baseline = baseline.to_str().expect("UTF-8 baseline");
    let clean = check(&project, baseline, &bin, "ok");
    assert!(
        clean.status.success(),
        "{}\n{}",
        report_text(&clean.stdout),
        String::from_utf8_lossy(&clean.stderr)
    );

    let changed = check(&project, baseline, &bin, "breaking");
    assert_eq!(changed.status.code(), Some(1));
    let report = report_text(&changed.stdout);
    assert!(report.contains("openapi-compatibility/oasdiff-breaking"));
    assert!(report.contains("../dummy-project-spex/contracts/client.openapi.yaml"));
    assert!(report.contains("the GET operation was removed"));
}

#[test]
fn compatibility_reports_missing_and_malformed_baselines() {
    let (_temp, project, baseline, bin) = fixture();
    let unspecified = check(&project, "", &bin, "ok");
    assert_eq!(unspecified.status.code(), Some(1));
    assert!(report_text(&unspecified.stdout).contains("set SPEX_OPENAPI_BASELINE_DIR"));

    let malformed = check(
        &project,
        baseline.to_str().expect("UTF-8 baseline"),
        &bin,
        "malformed",
    );
    assert_eq!(malformed.status.code(), Some(1));
    let report = report_text(&malformed.stdout);
    assert!(report.contains("oasdiff could not compare contracts"));
    assert!(report.contains("cannot parse the baseline document"));

    fs::remove_file(baseline.join("contracts/client.openapi.yaml"))
        .expect("remove fixture contract");
    let missing = check(
        &project,
        baseline.to_str().expect("UTF-8 baseline"),
        &bin,
        "ok",
    );
    assert_eq!(missing.status.code(), Some(1));
    assert!(report_text(&missing.stdout).contains("OpenAPI baseline contract is missing"));
}
