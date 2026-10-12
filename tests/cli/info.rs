use std::path::Path;

use crate::common::{
    assert_fails_mentioning, dummy_project_fixture, run_spex, run_spex_with_env, temp_project,
};

const EXPECTED_INFO: &str = r#"{
  "spex_dir": {spex_dir},
  "project_name": "dummy-project",
  "cli_version": "0.8.8",
  "description": "Minimal dummy project used by spex's own test suite to exercise `spex check` and `spex coverage` on a workspace whose specs cover every lifecycle state.",
  "rules_unused": [],
  "repositories": [
    {
      "repository_name": "dummy-project-api",
      "repository_short_name": "api",
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
      "ignored_errors": []
    },
    {
      "repository_name": "dummy-project-frontend",
      "repository_short_name": "frontend",
      "path": {frontend_dir},
      "contracts": [],
      "rules": [],
      "ignored_errors": []
    },
    {
      "repository_name": "dummy-project-spex",
      "repository_short_name": "spex",
      "path": {spex_dir},
      "contracts": [],
      "rules": [],
      "ignored_errors": []
    }
  ],
  "coverage_ignore": []
}
"#;

fn with_workspace_dirs(template: &str, project: &Path) -> String {
    let as_json = |repository_dir: &str| {
        let dir = project
            .join(repository_dir)
            .canonicalize()
            .expect("canonical repository dir");
        serde_json::to_string(&dir).expect("repository dir as JSON")
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
  "description": "Minimal dummy project used by spex's own test suite to exercise `spex check` and `spex coverage` on a workspace whose specs cover every lifecycle state.",
  "rules_unused": [],
  "repositories": [
    {
      "repository_name": "dummy-project-api",
      "repository_short_name": "api",
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
      "ignored_errors": []
    }
  ],
  "coverage_ignore": []
}
"#;

fn listed_short_names(stdout: &[u8]) -> Vec<String> {
    let info: serde_json::Value = serde_json::from_slice(stdout).expect("info prints JSON");
    info["repositories"]
        .as_array()
        .expect("repositories list")
        .iter()
        .map(|repository| {
            repository["repository_short_name"]
                .as_str()
                .expect("repository_short_name")
                .to_string()
        })
        .collect()
}

#[test]
fn info_prints_the_resolved_configuration_as_json() {
    let (_workspace, project) = temp_project(&dummy_project_fixture());

    let info = run_spex(&project, &["info"], "");

    assert!(info.status.success(), "info should exit 0");
    assert_eq!(
        String::from_utf8_lossy(&info.stdout),
        with_workspace_dirs(EXPECTED_INFO, &project)
    );
}

#[test]
fn info_reports_the_spex_dir_from_any_workspace_repository() {
    let (_workspace, project) = temp_project(&dummy_project_fixture());

    let info = run_spex(&project.join("dummy-project-api"), &["info"], "");

    assert!(info.status.success(), "info should exit 0");
    assert_eq!(
        String::from_utf8_lossy(&info.stdout),
        with_workspace_dirs(EXPECTED_INFO, &project)
    );
}

#[test]
fn info_repo_filters_only_the_repositories_list() {
    let (_workspace, project) = temp_project(&dummy_project_fixture());

    let by_short_name = run_spex(&project, &["info", "--repo", "api"], "");
    let by_full_name = run_spex(&project, &["info", "--repo", "dummy-project-api"], "");
    let several = run_spex(&project, &["info", "--repo", "frontend", "api"], "");

    let expected = with_workspace_dirs(EXPECTED_INFO_API, &project);

    assert!(by_short_name.status.success());
    assert_eq!(String::from_utf8_lossy(&by_short_name.stdout), expected);
    assert_eq!(
        String::from_utf8_lossy(&by_full_name.stdout),
        expected,
        "--repo should match a repository by full name as well as short name",
    );
    assert_eq!(listed_short_names(&several.stdout), ["api", "frontend"]);
}

#[test]
fn info_repo_rejects_an_unknown_repository() {
    let (_workspace, project) = temp_project(&dummy_project_fixture());

    let info = run_spex(&project, &["info", "--repo", "api", "bogus"], "");

    assert_fails_mentioning(&info, "unknown workspace repository `bogus`");
    assert!(info.stdout.is_empty());
}

#[test]
fn tool_failure_prints_one_plain_line_and_exits_two() {
    let (_workspace, project) = temp_project(&dummy_project_fixture());

    let info = run_spex(&project, &["info", "--repo", "bogus"], "");

    let stderr = String::from_utf8_lossy(&info.stderr);
    assert_eq!(info.status.code(), Some(2));
    assert!(
        stderr.ends_with("error: unknown workspace repository `bogus` — not listed in spex.yaml\n"),
        "unexpected stderr:\n{stderr}",
    );
    assert!(
        !stderr.contains('\u{1b}') && !stderr.contains("╰╴"),
        "stderr carries colour or a source location:\n{stderr}",
    );
}

#[test]
fn tool_failure_with_rust_backtrace_follows_the_error_line_with_a_detailed_report() {
    let (_workspace, project) = temp_project(&dummy_project_fixture());

    let info = run_spex_with_env(
        &project,
        &["info", "--repo", "bogus"],
        "",
        &[("RUST_BACKTRACE", "1")],
    );

    let stderr = String::from_utf8_lossy(&info.stderr);
    assert_eq!(info.status.code(), Some(2));
    assert!(
        stderr
            .contains("error: unknown workspace repository `bogus` — not listed in spex.yaml\n\n"),
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
