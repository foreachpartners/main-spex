use std::fs;
use std::path::Path;

use tempfile::tempdir;

use crate::common::{
    add_packages, assert_fails_mentioning, copy_no_spex, copy_project, dummy_project_fixture,
    report_text, run_spex, runners_project_fixture, temp_project, with_version_header,
};

mod ast_grep;
mod glob;
mod regexp;
mod script;

const TODO_FILE: &str = "src/todo.rs";

const OTHER_TODO_FILE: &str = "src/other_todo.rs";

const BASELINE_TODO_FILE: &str = r#"check_ignore:
  - rule_id: no-todo-comments
    check: no-todo
    files: [src/todo.rs]
"#;

fn project_with_todo_errors() -> (tempfile::TempDir, std::path::PathBuf) {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    copy_project(&dummy_project_fixture(), &project);
    for file in [TODO_FILE, OTHER_TODO_FILE] {
        fs::write(
            project.join("dummy-project-api").join(file),
            "// TODO later\n",
        )
        .expect("write file with a TODO");
    }
    (temp_dir, project)
}

fn write_api_package_config(project: &Path, text: &str) {
    fs::write(project.join("dummy-project-api/spex.package.yaml"), text)
        .expect("write spex.package.yaml");
}

const SPEX_REPORT: &str = r#"runners-project-spex
  no issues

"#;

fn expected_report(sections: &[&str], summary: &str) -> String {
    with_version_header(&format!("{}{summary}\n", sections.concat()))
}

fn project_with_rules(rules: &[&str]) -> (tempfile::TempDir, std::path::PathBuf) {
    let (temp_dir, project) = temp_project(&runners_project_fixture());
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
    (temp_dir, project)
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
fn check_package_rejects_an_unknown_package() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    copy_project(&runners_project_fixture(), &project);

    let check = run_spex(&project, &["check", "--package", "bogus"], "");

    assert_fails_mentioning(&check, "unknown package `bogus`");
    assert!(
        check.stdout.is_empty(),
        "no report should print when the package is unknown"
    );
}

#[test]
fn check_package_matches_a_package_by_short_or_full_name() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    copy_project(&runners_project_fixture(), &project);

    let by_short_name = run_spex(&project, &["check", "--package", "frontend"], "");
    let by_full_name = run_spex(
        &project,
        &["check", "--package", "runners-project-frontend"],
        "",
    );

    assert_eq!(by_short_name.status.code(), Some(1));
    assert_eq!(
        report_text(&by_full_name.stdout),
        report_text(&by_short_name.stdout),
        "--package should match a package by full name as well as short name",
    );
}

#[test]
fn check_honors_a_package_spex_repo_yaml_baseline_for_its_own_files_only() {
    let (_temp_dir, project) = project_with_todo_errors();
    let unbaselined = run_spex(&project, &["check"], "");
    let report = report_text(&unbaselined.stdout);
    assert!(
        report.contains(TODO_FILE) && report.contains(OTHER_TODO_FILE),
        "{report}"
    );

    write_api_package_config(&project, BASELINE_TODO_FILE);
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

fn exclude_in_api_package_entry(project: &Path, patterns: &str) {
    let spex_yaml = project.join("dummy-project-spex/spex.yaml");
    let config = fs::read_to_string(&spex_yaml).expect("read spex.yaml");
    let patched = config.replace(
        "    rules:\n      - no-todo-comments\n",
        &format!("    rules:\n      - no-todo-comments\n    exclude:\n{patterns}"),
    );
    assert_ne!(patched, config, "api package not found in spex.yaml");
    fs::write(&spex_yaml, patched).expect("write spex.yaml");
}

#[test]
fn check_skips_the_files_a_spex_yaml_package_exclude_matches() {
    let (_temp_dir, project) = project_with_todo_errors();
    exclude_in_api_package_entry(&project, "      - src/todo.rs\n");

    let check = run_spex(&project, &["check"], "");

    let report = report_text(&check.stdout);
    assert!(
        !report.contains(TODO_FILE),
        "excluded file still reported\n{report}"
    );
    assert!(
        report.contains(OTHER_TODO_FILE),
        "file outside the exclude hidden\n{report}"
    );
    assert_eq!(check.status.code(), Some(1), "{report}");
}

#[test]
fn check_skips_the_folders_and_files_a_package_spex_package_yaml_excludes() {
    let (_temp_dir, project) = project_with_todo_errors();
    write_api_package_config(&project, "exclude:\n  - src\n");

    let check = run_spex(&project, &["check"], "");

    let report = report_text(&check.stdout);
    assert!(
        check.status.success(),
        "everything under the excluded folder should be skipped (exit {:?})\n{report}",
        check.status.code()
    );
    assert!(report.contains("0 errors"), "{report}");
}

#[test]
fn check_unions_the_exclude_of_the_spex_yaml_entry_and_the_package_spex_package_yaml() {
    let (_temp_dir, project) = project_with_todo_errors();
    exclude_in_api_package_entry(&project, "      - src/todo.rs\n");
    write_api_package_config(&project, "exclude:\n  - src/other_todo.rs\n");

    let check = run_spex(&project, &["check"], "");

    let report = report_text(&check.stdout);
    assert!(
        check.status.success(),
        "both excludes should apply (exit {:?})\n{report}",
        check.status.code()
    );
}

#[test]
fn check_does_not_apply_the_exclude_of_one_package_to_another() {
    let (_temp_dir, project) = project_with_todo_errors();
    fs::write(
        project.join("dummy-project-frontend/spex.package.yaml"),
        "exclude:\n  - src\n",
    )
    .expect("write frontend spex.package.yaml");

    let check = run_spex(&project, &["check"], "");

    let report = report_text(&check.stdout);
    assert!(
        report.contains(TODO_FILE) && report.contains(OTHER_TODO_FILE),
        "a frontend exclude hid api findings\n{report}"
    );
}

#[test]
fn check_with_files_skips_a_listed_file_its_package_excludes_without_an_error() {
    let (_temp_dir, project) = project_with_todo_errors();
    write_api_package_config(&project, "exclude:\n  - src/todo.rs\n");

    let check = run_spex(&project, &["check", "--files", TODO_FILE], "");

    let report = report_text(&check.stdout);
    assert!(
        check.status.success(),
        "an excluded listed file should be skipped, not fail (exit {:?})\n{report}",
        check.status.code()
    );
    assert!(!report.contains(TODO_FILE), "{report}");
}

#[test]
fn check_with_files_still_checks_a_listed_file_its_package_does_not_exclude() {
    let (_temp_dir, project) = project_with_todo_errors();
    write_api_package_config(&project, "exclude:\n  - src/todo.rs\n");

    let check = run_spex(
        &project,
        &["check", "--files", TODO_FILE, OTHER_TODO_FILE],
        "",
    );

    let report = report_text(&check.stdout);
    assert!(
        !report.contains(TODO_FILE) && report.contains(OTHER_TODO_FILE),
        "{report}"
    );
    assert_eq!(check.status.code(), Some(1), "{report}");
}

#[test]
fn check_fails_naming_the_package_on_an_exclude_that_is_not_a_valid_glob() {
    let (_temp_dir, project) = project_with_todo_errors();
    write_api_package_config(&project, "exclude:\n  - \"src/{a,b\"\n");

    let check = run_spex(&project, &["check"], "");

    assert_fails_mentioning(&check, "invalid `exclude` of package `dummy-project-api`");
    assert_fails_mentioning(&check, "src/{a,b");
}

#[test]
fn check_merges_the_spex_yaml_baseline_with_the_package_spex_repo_yaml() {
    let (_temp_dir, project) = project_with_todo_errors();
    write_api_package_config(&project, BASELINE_TODO_FILE);
    let spex_yaml = project.join("dummy-project-spex/spex.yaml");
    let config = fs::read_to_string(&spex_yaml).expect("read spex.yaml");
    let patched = config.replace(
        "    rules:\n      - no-todo-comments\n",
        "    rules:\n      - no-todo-comments\n    check_ignore:\n      - rule_id: no-todo-comments\n        check: no-todo\n        files: [src/other_todo.rs]\n",
    );
    assert_ne!(patched, config, "api package not found in spex.yaml");
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
fn check_flags_an_unknown_key_in_a_package_spex_repo_yaml() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    copy_project(&dummy_project_fixture(), &project);
    write_api_package_config(&project, "ignored_error:\n  - rule_id: a\n    check: b\n");

    let check = run_spex(&project, &["check"], "");

    let report = report_text(&check.stdout);
    assert_eq!(check.status.code(), Some(1), "{report}");
    assert!(report.contains("spex.package.yaml"), "{report}");
    assert!(report.contains("ignored_error"), "{report}");
}

#[test]
fn check_accepts_a_valid_package_spex_repo_yaml() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    copy_project(&dummy_project_fixture(), &project);
    write_api_package_config(
        &project,
        "coverage_ignore:\n  - spec_id: none-with-gap\n    layer: impl\n    ac: resumes-favorite\ncheck_ignore:\n  - rule_id: no-todo-comments\n    check: no-todo\n",
    );

    let check = run_spex(&project, &["check"], "");

    assert!(
        check.status.success(),
        "a valid spex.package.yaml should pass\n{}",
        report_text(&check.stdout)
    );
}

#[test]
fn check_fails_on_an_unparsable_package_spex_repo_yaml() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    copy_project(&dummy_project_fixture(), &project);
    write_api_package_config(&project, "coverage_ignore: [unclosed\n");

    let check = run_spex(&project, &["check"], "");

    assert_fails_mentioning(&check, "spex.package.yaml");
}

#[test]
fn check_flags_a_spex_yaml_package_coverage_ignore_that_mismatches_the_spec_contract() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    copy_project(&dummy_project_fixture(), &project);
    let spex_yaml = project.join("dummy-project-spex/spex.yaml");
    let config = fs::read_to_string(&spex_yaml).expect("read spex.yaml");
    let patched = config.replace(
        "    rules:\n      - no-todo-comments\n",
        "    rules:\n      - no-todo-comments\n    coverage_ignore:\n      - spec_id: none-with-gap\n        layer: contract\n",
    );
    assert_ne!(patched, config, "api package not found in spex.yaml");
    fs::write(&spex_yaml, patched).expect("write spex.yaml");

    let check = run_spex(&project, &["check"], "");

    let report = report_text(&check.stdout);
    assert_eq!(check.status.code(), Some(1), "{report}");
    assert!(report.contains("none-with-gap/coverage-ignore"), "{report}");
}

#[test]
fn check_rejects_a_root_coverage_ignore_in_spex_yaml() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    copy_project(&dummy_project_fixture(), &project);
    let spex_yaml = project.join("dummy-project-spex/spex.yaml");
    let mut config = fs::read_to_string(&spex_yaml).expect("read spex.yaml");
    config.push_str("coverage_ignore:\n  - spec_id: none-with-gap\n    layer: impl\n");
    fs::write(&spex_yaml, config).expect("write spex.yaml");

    let check = run_spex(&project, &["check"], "");

    let report = report_text(&check.stdout);
    assert_eq!(check.status.code(), Some(1), "{report}");
    assert!(report.contains("spex.yaml"), "{report}");
    assert!(report.contains("coverage_ignore"), "{report}");
}

#[test]
fn check_flags_a_package_coverage_ignore_naming_a_spec_that_does_not_list_the_package() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    copy_project(&dummy_project_fixture(), &project);
    fs::write(
        project.join("dummy-project-frontend/spex.package.yaml"),
        "coverage_ignore:\n  - spec_id: none-with-gap\n    layer: impl\n    ac: resumes-favorite\n",
    )
    .expect("write spex.package.yaml");

    let check = run_spex(&project, &["check"], "");

    let report = report_text(&check.stdout);
    assert_eq!(check.status.code(), Some(1), "{report}");
    assert!(report.contains("none-with-gap/coverage-ignore"), "{report}");
    assert!(report.contains("package `frontend`"), "{report}");
}

#[test]
fn fill_preexisting_errors_writes_spex_yaml_and_leaves_spex_repo_yaml_untouched() {
    let (_temp_dir, project) = project_with_todo_errors();
    write_api_package_config(&project, BASELINE_TODO_FILE);

    let fill = run_spex(&project, &["check", "--fill-preexisting-errors"], "");

    assert!(fill.status.success(), "fill should exit 0");
    assert_eq!(
        fs::read_to_string(project.join("dummy-project-api/spex.package.yaml"))
            .expect("read spex.package.yaml"),
        BASELINE_TODO_FILE,
        "fill must not touch spex.package.yaml"
    );
    let config =
        fs::read_to_string(project.join("dummy-project-spex/spex.yaml")).expect("read spex.yaml");
    assert!(
        config.contains("src/other_todo.rs") && !config.contains("src/todo.rs"),
        "fill should baseline in spex.yaml the errors spex.package.yaml does not already baseline\n{config}"
    );
    assert!(run_spex(&project, &["check"], "").status.success());
}

#[test]
fn fill_preexisting_errors_with_package_writes_the_package_spex_yaml_only() {
    let (_temp_dir, project) = project_with_todo_errors();
    let spex_yaml = project.join("dummy-project-spex/spex.yaml");
    let spex_yaml_before = fs::read_to_string(&spex_yaml).expect("read spex.yaml");

    let fill = run_spex(
        &project,
        &["check", "--package", "api", "--fill-preexisting-errors"],
        "",
    );

    assert!(fill.status.success(), "fill should exit 0");
    assert!(
        report_text(&fill.stdout).contains("spex.package.yaml"),
        "{}",
        report_text(&fill.stdout)
    );
    assert_eq!(
        fs::read_to_string(&spex_yaml).expect("read spex.yaml"),
        spex_yaml_before,
        "fill with --package must not touch spex.yaml"
    );
    let package_config = fs::read_to_string(project.join("dummy-project-api/spex.package.yaml"))
        .expect("fill should create the package spex.package.yaml");
    assert!(
        package_config.contains("src/todo.rs") && package_config.contains("src/other_todo.rs"),
        "fill should baseline every current error in spex.package.yaml\n{package_config}"
    );
    assert!(
        !project
            .join("dummy-project-frontend/spex.package.yaml")
            .exists(),
        "an unselected package must stay untouched"
    );
    assert!(run_spex(&project, &["check"], "").status.success());
}

#[test]
fn fill_preexisting_errors_with_package_regenerates_the_block_and_keeps_the_rest() {
    let (_temp_dir, project) = project_with_todo_errors();
    write_api_package_config(
        &project,
        "# owned by the api team\ncheck_ignore:\n  - rule_id: no-todo-comments\n    check: no-todo\n    files: [src/gone.rs]\n",
    );

    let fill = run_spex(
        &project,
        &["check", "--package", "api", "--fill-preexisting-errors"],
        "",
    );

    assert!(fill.status.success(), "fill should exit 0");
    let package_config = fs::read_to_string(project.join("dummy-project-api/spex.package.yaml"))
        .expect("read spex.package.yaml");
    assert!(
        package_config.starts_with("# owned by the api team\ncheck_ignore:\n"),
        "{package_config}"
    );
    assert!(
        !package_config.contains("src/gone.rs") && package_config.contains("src/todo.rs"),
        "{package_config}"
    );
}

#[test]
fn fill_preexisting_errors_with_package_empties_the_file_once_no_errors_are_left() {
    let (_temp_dir, project) = project_with_todo_errors();
    let fill_api = || {
        run_spex(
            &project,
            &["check", "--package", "api", "--fill-preexisting-errors"],
            "",
        )
    };
    fill_api();
    for file in [TODO_FILE, OTHER_TODO_FILE] {
        fs::remove_file(project.join("dummy-project-api").join(file)).expect("remove file");
    }

    let refill = fill_api();

    assert!(refill.status.success(), "refill should exit 0");
    assert_eq!(
        fs::read_to_string(project.join("dummy-project-api/spex.package.yaml"))
            .expect("read spex.package.yaml"),
        ""
    );
    let check = run_spex(&project, &["check"], "");
    assert!(
        check.status.success(),
        "check should still pass\n{}\n{}",
        report_text(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
}

#[test]
fn fill_preexisting_errors_with_package_leaves_out_what_spex_yaml_already_baselines() {
    let (_temp_dir, project) = project_with_todo_errors();
    let spex_yaml = project.join("dummy-project-spex/spex.yaml");
    let config = fs::read_to_string(&spex_yaml).expect("read spex.yaml");
    let patched = config.replace(
        "    rules:\n      - no-todo-comments\n",
        "    rules:\n      - no-todo-comments\n    check_ignore:\n      - rule_id: no-todo-comments\n        check: no-todo\n        files: [src/todo.rs]\n",
    );
    assert_ne!(patched, config, "api package not found in spex.yaml");
    fs::write(&spex_yaml, &patched).expect("write spex.yaml");

    let fill = run_spex(
        &project,
        &["check", "--package", "api", "--fill-preexisting-errors"],
        "",
    );

    assert!(fill.status.success(), "fill should exit 0");
    assert_eq!(
        fs::read_to_string(&spex_yaml).expect("read spex.yaml"),
        patched,
        "fill with --package must not touch spex.yaml"
    );
    let package_config = fs::read_to_string(project.join("dummy-project-api/spex.package.yaml"))
        .expect("read spex.package.yaml");
    assert!(
        package_config.contains("src/other_todo.rs") && !package_config.contains("src/todo.rs"),
        "an error baselined in spex.yaml must not be filled again\n{package_config}"
    );
    assert!(run_spex(&project, &["check"], "").status.success());
}

#[test]
fn fill_preexisting_errors_with_package_creates_no_file_when_spex_yaml_baselines_every_error() {
    let (_temp_dir, project) = project_with_todo_errors();
    let first_fill = run_spex(&project, &["check", "--fill-preexisting-errors"], "");
    assert!(first_fill.status.success(), "fill should exit 0");

    let fill = run_spex(
        &project,
        &["check", "--package", "api", "--fill-preexisting-errors"],
        "",
    );

    assert!(fill.status.success(), "fill should exit 0");
    assert!(
        !project.join("dummy-project-api/spex.package.yaml").exists(),
        "everything is already baselined in spex.yaml, so there is nothing to write"
    );
}

#[test]
fn check_accepts_an_empty_package_spex_yaml() {
    let (_temp_dir, project) = temp_project(&dummy_project_fixture());
    for text in ["", "# nothing to ignore yet\n"] {
        write_api_package_config(&project, text);

        let check = run_spex(&project, &["check"], "");

        assert!(
            check.status.success(),
            "{text:?}\n{}",
            report_text(&check.stdout)
        );
    }
}

#[test]
fn fill_preexisting_errors_with_package_creates_no_file_for_a_package_without_errors() {
    let (_temp_dir, project) = project_with_todo_errors();

    let fill = run_spex(
        &project,
        &[
            "check",
            "--package",
            "frontend",
            "--fill-preexisting-errors",
        ],
        "",
    );

    assert!(fill.status.success(), "fill should exit 0");
    assert!(
        !project
            .join("dummy-project-frontend/spex.package.yaml")
            .exists(),
        "nothing to baseline, so no file"
    );
    assert!(
        report_text(&fill.stdout).contains("spex.package.yaml unchanged"),
        "{}",
        report_text(&fill.stdout)
    );
}

#[test]
fn preexisting_errors_baseline_suppresses_matching_error() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");

    copy_no_spex(&project);

    let init = run_spex(&project, &["init"], "\n");
    assert!(init.status.success(), "init failed");

    add_packages(&project);

    let config_path = project.join("proj-spex/spex.yaml");
    let config = fs::read_to_string(&config_path).expect("read spex.yaml");
    let patched = config.replace(
        "  - package_name: proj-frontend\n    package_short_name: frontend\n    path: ../proj-frontend\n    rules:\n      - writing-style\n",
        "  - package_name: proj-frontend\n    package_short_name: frontend\n    path: ../proj-frontend\n    rules:\n      - writing-style\n    check_ignore:\n      - rule_id: writing-style\n        check: no-cyrillic\n        files: [README.md]\n",
    );
    assert_ne!(patched, config, "frontend package not found in spex.yaml");
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
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");

    copy_no_spex(&project);

    let init = run_spex(&project, &["init"], "\n");
    assert!(init.status.success(), "init failed");

    add_packages(&project);

    let fill = run_spex(&project, &["check", "--fill-preexisting-errors"], "");
    assert!(
        fill.status.success(),
        "--fill-preexisting-errors should exit 0 (exit {:?})\nstderr:\n{}",
        fill.status.code(),
        String::from_utf8_lossy(&fill.stderr),
    );

    let config = fs::read_to_string(project.join("proj-spex/spex.yaml")).expect("read spex.yaml");
    assert!(
        config.contains("check_ignore:")
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
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");

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
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");

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

const EXPECTED_API_ONLY_CHECK_REPORT: &str = r#"dummy-project-api
  no issues

dummy-project-spex
  no issues

dummy-project: 0 errors, 0 warnings
"#;

#[test]
fn check_reports_the_spex_package_even_when_it_is_not_selected() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    copy_project(&dummy_project_fixture(), &project);

    let check = run_spex(&project, &["check", "--package", "api"], "");

    assert_eq!(
        report_text(&check.stdout),
        with_version_header(EXPECTED_API_ONLY_CHECK_REPORT),
    );
    assert!(check.status.success());
}

#[test]
fn check_runs_conformance_whatever_package_or_files_are_selected() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    copy_project(&dummy_project_fixture(), &project);
    let spec_path = project.join("dummy-project-spex/specs/active-full-coverage.yaml");
    let spec = fs::read_to_string(&spec_path).expect("read spec");
    fs::write(
        &spec_path,
        spec.replace("id: shows-tax", "id: shows-subtotal"),
    )
    .expect("write spec with a duplicate acceptance criterion id");

    for args in [
        vec!["check", "--package", "api"],
        vec!["check", "--package", "frontend"],
        vec!["check", "--files", "src/todo.rs"],
    ] {
        fs::write(
            project.join("dummy-project-api/src/todo.rs"),
            "fn todo() {}\n",
        )
        .expect("write file");
        let check = run_spex(&project, &args, "");
        let report = report_text(&check.stdout);
        assert!(
            report.contains("\ndummy-project-spex\n")
                && report.contains("spex-internal-conformance/no-duplicate-ac-ids"),
            "conformance was not reported for {args:?}\n{report}",
        );
        assert_eq!(check.status.code(), Some(1), "{args:?}");
    }
}

#[test]
fn check_runs_annotation_checks_only_on_selected_packages() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    copy_project(&dummy_project_fixture(), &project);
    fs::write(
        project.join("dummy-project-api/src/rule_reference.rs"),
        "// @spec impl no-todo-comments\n",
    )
    .expect("write rule reference");
    let orphan = "orphan annotation references unknown spec `no-todo-comments`";

    let api = run_spex(&project, &["check", "--package", "api"], "");
    assert!(report_text(&api.stdout).contains(orphan));
    assert_eq!(api.status.code(), Some(1));

    for other in ["spex", "frontend"] {
        let check = run_spex(&project, &["check", "--package", other], "");
        let report = report_text(&check.stdout);
        assert!(!report.contains(orphan), "--package {other}\n{report}");
        assert!(check.status.success(), "--package {other}\n{report}");
    }
}

#[test]
fn check_validates_annotations_in_markdown_files() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");

    copy_project(&dummy_project_fixture(), &project);
    fs::write(
        project.join("dummy-project-api/contract.md"),
        "<!-- @spec contract no-such-spec -->\n",
    )
    .expect("write markdown contract");

    let check = run_spex(&project, &["check"], "");
    let report = report_text(&check.stdout);
    assert!(
        report.contains("orphan annotation references unknown spec `no-such-spec`"),
        "annotation in a markdown file was not flagged\n{report}",
    );
    assert_eq!(check.status.code(), Some(1));
}

#[test]
fn check_flags_annotation_referencing_rule_id_as_orphan() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");

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
