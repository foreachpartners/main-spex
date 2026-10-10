use std::fs;
use std::path::Path;

use tempfile::tempdir;

mod common;
use common::{assert_fails_mentioning, copy_project, dummy_project_fixture, report_text, run_spex};

const TODO_FILE: &str = "src/todo.rs";
const OTHER_TODO_FILE: &str = "src/other_todo.rs";

const BASELINE_TODO_FILE: &str = r#"ignored_errors:
  - rule_id: no-todo-comments
    check: no-todo
    files: [src/todo.rs]
"#;

fn project_with_todo_errors() -> (tempfile::TempDir, std::path::PathBuf) {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");
    copy_project(&dummy_project_fixture(), &project);
    for file in [TODO_FILE, OTHER_TODO_FILE] {
        fs::write(
            project.join("dummy-project-api").join(file),
            "// TODO later\n",
        )
        .expect("write file with a TODO");
    }
    (workspace, project)
}

fn write_api_repo_config(project: &Path, text: &str) {
    fs::write(project.join("dummy-project-api/spex.repo.yaml"), text)
        .expect("write spex.repo.yaml");
}

#[test]
fn check_honors_a_repository_spex_repo_yaml_baseline_for_its_own_files_only() {
    let (_workspace, project) = project_with_todo_errors();
    let unbaselined = run_spex(&project, &["check"], "");
    let report = report_text(&unbaselined.stdout);
    assert!(
        report.contains(TODO_FILE) && report.contains(OTHER_TODO_FILE),
        "{report}"
    );

    write_api_repo_config(&project, BASELINE_TODO_FILE);
    let check = run_spex(&project, &["check"], "");

    let report = report_text(&check.stdout);
    assert!(
        !report.contains(TODO_FILE),
        "listed file still reported\n{report}"
    );
    assert!(
        report.contains(OTHER_TODO_FILE),
        "unlisted file hidden\n{report}"
    );
    assert_eq!(check.status.code(), Some(1), "{report}");
}

#[test]
fn check_merges_the_spex_yaml_baseline_with_the_repository_spex_repo_yaml() {
    let (_workspace, project) = project_with_todo_errors();
    write_api_repo_config(&project, BASELINE_TODO_FILE);
    let spex_yaml = project.join("dummy-project-spex/spex.yaml");
    let config = fs::read_to_string(&spex_yaml).expect("read spex.yaml");
    let patched = config.replace(
        "    rules:\n      - no-todo-comments\n",
        "    rules:\n      - no-todo-comments\n    ignored_errors:\n      - rule_id: no-todo-comments\n        check: no-todo\n        files: [src/other_todo.rs]\n",
    );
    assert_ne!(patched, config, "api repository not found in spex.yaml");
    fs::write(&spex_yaml, patched).expect("write spex.yaml");

    let check = run_spex(&project, &["check"], "");

    let report = report_text(&check.stdout);
    assert!(
        check.status.success(),
        "both baselines should apply (exit {:?})\n{report}",
        check.status.code()
    );
    assert!(report.contains("0 errors"), "{report}");
}

#[test]
fn check_flags_an_unknown_key_in_a_repository_spex_repo_yaml() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");
    copy_project(&dummy_project_fixture(), &project);
    write_api_repo_config(&project, "ignored_error:\n  - rule_id: a\n    check: b\n");

    let check = run_spex(&project, &["check"], "");

    let report = report_text(&check.stdout);
    assert_eq!(check.status.code(), Some(1), "{report}");
    assert!(report.contains("spex.repo.yaml"), "{report}");
    assert!(report.contains("ignored_error"), "{report}");
}

#[test]
fn check_accepts_a_valid_repository_spex_repo_yaml() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");
    copy_project(&dummy_project_fixture(), &project);
    write_api_repo_config(
        &project,
        "coverage_ignore:\n  - spec_id: none-with-gap\n    layer: impl\n    ac: resumes-favorite\nignored_errors:\n  - rule_id: no-todo-comments\n    check: no-todo\n",
    );

    let check = run_spex(&project, &["check"], "");

    assert!(
        check.status.success(),
        "a valid spex.repo.yaml should pass\n{}",
        report_text(&check.stdout)
    );
}

#[test]
fn check_fails_on_an_unparsable_repository_spex_repo_yaml() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");
    copy_project(&dummy_project_fixture(), &project);
    write_api_repo_config(&project, "coverage_ignore: [unclosed\n");

    let check = run_spex(&project, &["check"], "");

    assert_fails_mentioning(&check, "spex.repo.yaml");
}

#[test]
fn check_flags_a_spex_yaml_coverage_ignore_that_mismatches_the_spec_contract() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");
    copy_project(&dummy_project_fixture(), &project);
    let spex_yaml = project.join("dummy-project-spex/spex.yaml");
    let mut config = fs::read_to_string(&spex_yaml).expect("read spex.yaml");
    config.push_str("coverage_ignore:\n  - spec_id: none-with-gap\n    layer: contract\n");
    fs::write(&spex_yaml, config).expect("write spex.yaml");

    let check = run_spex(&project, &["check"], "");

    let report = report_text(&check.stdout);
    assert_eq!(check.status.code(), Some(1), "{report}");
    assert!(report.contains("none-with-gap/coverage-ignore"), "{report}");
}

#[test]
fn fill_preexisting_errors_writes_spex_yaml_and_leaves_spex_repo_yaml_untouched() {
    let (_workspace, project) = project_with_todo_errors();
    write_api_repo_config(&project, BASELINE_TODO_FILE);

    let fill = run_spex(&project, &["check", "--fill-preexisting-errors"], "");

    assert!(fill.status.success(), "fill should exit 0");
    assert_eq!(
        fs::read_to_string(project.join("dummy-project-api/spex.repo.yaml"))
            .expect("read spex.repo.yaml"),
        BASELINE_TODO_FILE,
        "fill must not touch spex.repo.yaml"
    );
    let config =
        fs::read_to_string(project.join("dummy-project-spex/spex.yaml")).expect("read spex.yaml");
    assert!(
        config.contains("src/todo.rs") && config.contains("src/other_todo.rs"),
        "fill should baseline every current error in spex.yaml\n{config}"
    );
    assert!(run_spex(&project, &["check"], "").status.success());
}
