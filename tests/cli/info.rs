use std::fs;
use std::path::Path;

use crate::common::{
    assert_fails_mentioning, dummy_project_fixture, run_spex, run_spex_with_env, temp_project,
};

const EXPECTED_INFO: &str = r#"{
  "spex_dir": {spex_dir},
  "project_name": "dummy-project",
  "cli_version": "0.8.8",
  "layout": "multi-repo",
  "agent_root": {spex_dir},
  "description": "Minimal dummy project used by spex's own test suite to exercise `spex check` and `spex coverage` on a project whose specs cover every lifecycle state.",
  "rules_unused": [],
  "packages": [
    {
      "package_name": "dummy-project-api",
      "package_short_name": "api",
      "path": {api_dir},
      "contracts": [
        {
          "path": "./contracts.md",
          "description": "Interface contract notes for the api."
        }
      ],
      "rules": [
        "no-todo-comments"
      ],
      "check_ignore": []
    },
    {
      "package_name": "dummy-project-frontend",
      "package_short_name": "frontend",
      "path": {frontend_dir},
      "contracts": [],
      "rules": [],
      "check_ignore": []
    },
    {
      "package_name": "dummy-project-spex",
      "package_short_name": "spex",
      "path": {spex_dir},
      "contracts": [],
      "rules": [],
      "check_ignore": []
    }
  ],
  "coverage_ignore": []
}
"#;

fn with_project_dirs(template: &str, project: &Path) -> String {
    let as_json = |package_dir: &str| {
        let dir = project
            .join(package_dir)
            .canonicalize()
            .expect("canonical package dir");
        serde_json::to_string(&dir).expect("package dir as JSON")
    };
    template
        .replace("{spex_dir}", &as_json("dummy-project-spex"))
        .replace("{api_dir}", &as_json("dummy-project-api"))
        .replace("{frontend_dir}", &as_json("dummy-project-frontend"))
}

const EXPECTED_INFO_API: &str = r#"{
  "spex_dir": {spex_dir},
  "project_name": "dummy-project",
  "cli_version": "0.8.8",
  "layout": "multi-repo",
  "agent_root": {spex_dir},
  "description": "Minimal dummy project used by spex's own test suite to exercise `spex check` and `spex coverage` on a project whose specs cover every lifecycle state.",
  "rules_unused": [],
  "packages": [
    {
      "package_name": "dummy-project-api",
      "package_short_name": "api",
      "path": {api_dir},
      "contracts": [
        {
          "path": "./contracts.md",
          "description": "Interface contract notes for the api."
        }
      ],
      "rules": [
        "no-todo-comments"
      ],
      "check_ignore": []
    }
  ],
  "coverage_ignore": []
}
"#;

fn listed_short_names(stdout: &[u8]) -> Vec<String> {
    let info: serde_json::Value = serde_json::from_slice(stdout).expect("info prints JSON");
    info["packages"]
        .as_array()
        .expect("packages list")
        .iter()
        .map(|package| {
            package["package_short_name"]
                .as_str()
                .expect("package_short_name")
                .to_string()
        })
        .collect()
}

#[test]
fn info_prints_the_resolved_configuration_as_json() {
    let (_temp_dir, project) = temp_project(&dummy_project_fixture());

    let info = run_spex(&project, &["info"], "");

    assert!(info.status.success(), "info should exit 0");
    assert_eq!(
        String::from_utf8_lossy(&info.stdout),
        with_project_dirs(EXPECTED_INFO, &project)
    );
}

#[test]
fn info_reports_the_spex_dir_from_any_project_package() {
    let (_temp_dir, project) = temp_project(&dummy_project_fixture());

    let info = run_spex(&project.join("dummy-project-api"), &["info"], "");

    assert!(info.status.success(), "info should exit 0");
    assert_eq!(
        String::from_utf8_lossy(&info.stdout),
        with_project_dirs(EXPECTED_INFO, &project)
    );
}

#[test]
fn info_package_filters_only_the_packages_list() {
    let (_temp_dir, project) = temp_project(&dummy_project_fixture());

    let by_short_name = run_spex(&project, &["info", "--package", "api"], "");
    let by_full_name = run_spex(&project, &["info", "--package", "dummy-project-api"], "");
    let several = run_spex(&project, &["info", "--package", "frontend", "api"], "");

    let expected = with_project_dirs(EXPECTED_INFO_API, &project);

    assert!(by_short_name.status.success());
    assert_eq!(String::from_utf8_lossy(&by_short_name.stdout), expected);
    assert_eq!(
        String::from_utf8_lossy(&by_full_name.stdout),
        expected,
        "--package should match a package by full name as well as short name",
    );
    assert_eq!(listed_short_names(&several.stdout), ["api", "frontend"]);
}

fn write_api_package_config(project: &Path) {
    fs::write(
        project.join("dummy-project-api/spex.package.yaml"),
        "coverage_ignore:\n  - spec_id: place-order\n    layer: impl\ncheck_ignore:\n  - rule_id: writing-style\n    check: no-cyrillic\n    files: [README.md]\n",
    )
    .expect("write package config");
}

fn info_json(stdout: &[u8]) -> serde_json::Value {
    serde_json::from_slice(stdout).expect("info prints JSON")
}

#[test]
fn info_merges_every_package_config_into_the_configuration() {
    let (_temp_dir, project) = temp_project(&dummy_project_fixture());
    write_api_package_config(&project);

    let info = run_spex(&project, &["info"], "");

    assert!(info.status.success(), "info should exit 0");
    let info = info_json(&info.stdout);
    assert_eq!(
        info["coverage_ignore"],
        serde_json::json!([{ "spec_id": "place-order", "layer": "impl" }]),
    );
    assert_eq!(
        info["packages"][0]["check_ignore"],
        serde_json::json!([
            { "rule_id": "writing-style", "check": "no-cyrillic", "files": ["README.md"] }
        ]),
    );
    assert_eq!(info["packages"][1]["check_ignore"], serde_json::json!([]));
}

#[test]
fn info_package_still_merges_the_config_of_the_packages_it_filters_out() {
    let (_temp_dir, project) = temp_project(&dummy_project_fixture());
    write_api_package_config(&project);

    let info = run_spex(&project, &["info", "--package", "frontend"], "");

    assert!(info.status.success(), "info should exit 0");
    assert_eq!(listed_short_names(&info.stdout), ["frontend"]);
    assert_eq!(
        info_json(&info.stdout)["coverage_ignore"][0]["spec_id"],
        "place-order",
    );
}

#[test]
fn info_package_rejects_an_unknown_package() {
    let (_temp_dir, project) = temp_project(&dummy_project_fixture());

    let info = run_spex(&project, &["info", "--package", "api", "bogus"], "");

    assert_fails_mentioning(&info, "unknown package `bogus`");
    assert!(info.stdout.is_empty());
}

#[test]
fn tool_failure_prints_one_plain_line_and_exits_two() {
    let (_temp_dir, project) = temp_project(&dummy_project_fixture());

    let info = run_spex(&project, &["info", "--package", "bogus"], "");

    let stderr = String::from_utf8_lossy(&info.stderr);
    assert_eq!(info.status.code(), Some(2));
    assert!(
        stderr.ends_with("error: unknown package `bogus` — not listed in spex.yaml\n"),
        "unexpected stderr:\n{stderr}",
    );
    assert!(
        !stderr.contains('\u{1b}') && !stderr.contains("╰╴"),
        "stderr carries colour or a source location:\n{stderr}",
    );
}

#[test]
fn tool_failure_with_rust_backtrace_follows_the_error_line_with_a_detailed_report() {
    let (_temp_dir, project) = temp_project(&dummy_project_fixture());

    let info = run_spex_with_env(
        &project,
        &["info", "--package", "bogus"],
        "",
        &[("RUST_BACKTRACE", "1")],
    );

    let stderr = String::from_utf8_lossy(&info.stderr);
    assert_eq!(info.status.code(), Some(2));
    assert!(
        stderr.contains("error: unknown package `bogus` — not listed in spex.yaml\n\n"),
        "the error line is missing:\n{stderr}",
    );
    assert!(
        stderr.contains("[src/config/spex.rs:"),
        "the error location is missing:\n{stderr}",
    );
    assert!(
        stderr.contains("Backtrace (captured)"),
        "the backtrace is missing:\n{stderr}",
    );
}
