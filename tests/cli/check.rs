use std::fs;
use std::path::Path;

use tempfile::tempdir;

use crate::common::{
    add_repositories, assert_fails_mentioning, copy_no_spex, copy_project, dummy_project_fixture,
    report_text, run_spex, runners_project_fixture, temp_project, with_version_header,
};

mod ast_grep;
mod glob;
mod regexp;
mod script;

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

const SPEX_REPORT: &str = r#"runners-project-spex
  no issues

"#;

fn expected_report(sections: &[&str], summary: &str) -> String {
    with_version_header(&format!("{}{summary}\n", sections.concat()))
}

fn project_with_rules(rules: &[&str]) -> (tempfile::TempDir, std::path::PathBuf) {
    let (workspace, project) = temp_project(&runners_project_fixture());
    let config_path = project.join("runners-project-spex/spex.yaml");
    let config = fs::read_to_string(&config_path).expect("read spex.yaml");
    let kept: Vec<&str> = config
        .lines()
        .filter(|line| {
            line.strip_prefix("      - ")
                .is_none_or(|rule| rules.contains(&rule))
        })
        .collect();
    fs::write(&config_path, format!("{}\n", kept.join("\n"))).expect("write spex.yaml");
    (workspace, project)
}

const EXPECTED_DUMMY_CHECK_REPORT: &str = r#"dummy-project-api
  no issues

dummy-project-frontend
  no issues

dummy-project-spex
  no issues

dummy-project: 0 errors, 0 warnings
"#;

#[test]
fn check_repo_rejects_an_unknown_repository() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");
    copy_project(&runners_project_fixture(), &project);

    let check = run_spex(&project, &["check", "--repo", "bogus"], "");

    assert_fails_mentioning(&check, "unknown workspace repository `bogus`");
    assert!(
        check.stdout.is_empty(),
        "no report should print when the repository is unknown"
    );
}

#[test]
fn check_repo_matches_a_repository_by_short_or_full_name() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");
    copy_project(&runners_project_fixture(), &project);

    let by_short_name = run_spex(&project, &["check", "--repo", "frontend"], "");
    let by_full_name = run_spex(
        &project,
        &["check", "--repo", "runners-project-frontend"],
        "",
    );

    assert_eq!(by_short_name.status.code(), Some(1));
    assert_eq!(
        report_text(&by_full_name.stdout),
        report_text(&by_short_name.stdout),
        "--repo should match a repository by full name as well as short name",
    );
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

#[test]
fn preexisting_errors_baseline_suppresses_matching_error() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");

    copy_no_spex(&project);

    let init = run_spex(&project, &["init"], "\n");
    assert!(init.status.success(), "init failed");

    add_repositories(&project);

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

    let init = run_spex(&project, &["init"], "\n");
    assert!(init.status.success(), "init failed");

    add_repositories(&project);

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

    let refill = run_spex(&project, &["check", "--fill-preexisting-errors"], "");
    let config_after =
        fs::read_to_string(project.join("proj-spex/spex.yaml")).expect("reread spex.yaml");
    assert_eq!(config, config_after, "second fill changed spex.yaml");
    assert!(refill.status.success());

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
fn fill_preexisting_errors_baselines_spec_conformance_errors() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");

    copy_project(&dummy_project_fixture(), &project);
    let spec_path = project.join("dummy-project-spex/specs/active-full-coverage.yaml");
    let spec = fs::read_to_string(&spec_path).expect("read spec");
    fs::write(
        &spec_path,
        spec.replace("id: shows-tax", "id: shows-subtotal"),
    )
    .expect("write spec with a duplicate acceptance criterion id");

    let broken = run_spex(&project, &["check"], "");
    assert_eq!(broken.status.code(), Some(1));
    assert!(
        report_text(&broken.stdout).contains("spex-internal-conformance/no-duplicate-ac-ids"),
        "the duplicate acceptance criterion id was not reported",
    );

    let fill = run_spex(&project, &["check", "--fill-preexisting-errors"], "");
    assert!(fill.status.success(), "fill should exit 0");

    let check = run_spex(&project, &["check"], "");
    let report = report_text(&check.stdout);
    assert!(
        !report.contains("no-duplicate-ac-ids"),
        "generated baseline did not suppress the conformance error\n{report}",
    );
    assert!(
        check.status.success(),
        "check should exit 0 after the baseline is generated (exit {:?})",
        check.status.code(),
    );
}

#[test]
fn check_dummy_project_matches_expected() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");

    copy_project(&dummy_project_fixture(), &project);

    let check = run_spex(&project, &["check"], "");
    assert_eq!(
        report_text(&check.stdout),
        with_version_header(EXPECTED_DUMMY_CHECK_REPORT),
        "check report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&check.stderr),
    );
    assert!(
        check.status.success(),
        "check should exit 0 when there are no errors (exit {:?})",
        check.status.code(),
    );
}

#[test]
fn check_flags_annotation_referencing_rule_id_as_orphan() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");

    copy_project(&dummy_project_fixture(), &project);
    fs::write(
        project.join("dummy-project-api/src/rule_reference.rs"),
        "// @spec impl no-todo-comments\n",
    )
    .expect("write rule reference");

    let check = run_spex(&project, &["check"], "");
    let report = report_text(&check.stdout);
    assert!(
        report.contains("orphan annotation references unknown spec `no-todo-comments`"),
        "annotation referencing a rule id was not flagged\n{report}",
    );
    assert!(
        !check.status.success(),
        "check should exit non-zero on an orphan annotation",
    );
}
