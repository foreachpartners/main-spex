mod common;
use std::path::Path;

use common::{assert_fails_mentioning, dummy_project_fixture, run_spex, temp_project};

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
      "path": "../dummy-project-api",
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
      "path": "../dummy-project-frontend",
      "contracts": [],
      "rules": [],
      "ignored_errors": []
    },
    {
      "repository_name": "dummy-project-spex",
      "repository_short_name": "spex",
      "path": ".",
      "contracts": [],
      "rules": [],
      "ignored_errors": []
    }
  ]
}
"#;

const EXPECTED_INFO_API: &str = r#"{
  "spex_dir": {spex_dir},
  "project_name": "dummy-project",
  "cli_version": "0.8.8",
  "repositories": [
    {
      "repository_name": "dummy-project-api",
      "repository_short_name": "api",
      "path": "../dummy-project-api",
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
  ]
}
"#;

fn with_spex_dir(template: &str, spex_dir: &Path) -> String {
    let spex_dir = spex_dir.canonicalize().expect("canonical spex dir");
    let spex_dir_json = serde_json::to_string(&spex_dir).expect("spex dir as JSON");
    template.replace("{spex_dir}", &spex_dir_json)
}

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
        with_spex_dir(EXPECTED_INFO, &project.join("dummy-project-spex"))
    );
}

#[test]
fn info_reports_the_spex_dir_from_any_workspace_repository() {
    let (_workspace, project) = temp_project(&dummy_project_fixture());

    let info = run_spex(&project.join("dummy-project-api"), &["info"], "");

    assert!(info.status.success(), "info should exit 0");
    assert_eq!(
        String::from_utf8_lossy(&info.stdout),
        with_spex_dir(EXPECTED_INFO, &project.join("dummy-project-spex"))
    );
}

#[test]
fn info_repo_limits_output_to_the_general_fields_and_named_repositories() {
    let (_workspace, project) = temp_project(&dummy_project_fixture());

    let by_short_name = run_spex(&project, &["info", "--repo", "api"], "");
    let by_full_name = run_spex(&project, &["info", "--repo", "dummy-project-api"], "");
    let several = run_spex(&project, &["info", "--repo", "api", "frontend"], "");

    let expected = with_spex_dir(EXPECTED_INFO_API, &project.join("dummy-project-spex"));

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
