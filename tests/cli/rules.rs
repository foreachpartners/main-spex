use std::fs;

use tempfile::tempdir;

use crate::common::{
    assert_fails_mentioning, canonical_display, copy_project, dummy_project_fixture, listed_ids,
    run_spex, runners_project_fixture, temp_project,
};

const EXPECTED_GLOB_RULE: &str = r#"- id: glob-runner
  title: Glob runner
  sync_with_upstream: true
  description: Exercises the glob runner with a forbidden path and a required path.
  acceptance_criteria:
  - MUST provide a README.md in the package root
  - MUST NOT keep backup files in a package
  checks:
  - name: no-backup-files
    type: glob
    pattern: '**/*.bak'
    severity: warn
  - name: require-readme
    type: glob
    pattern: README.md
    negate: true
    severity: error
"#;

const EXPECTED_GLOB_RULE_FOR_AI: &str = r#"- id: glob-runner
  path: {spex_dir}/rules/glob-runner.yaml
  title: Glob runner
  description: Exercises the glob runner with a forbidden path and a required path.
  acceptance_criteria:
  - MUST provide a README.md in the package root
  - MUST NOT keep backup files in a package
"#;

#[test]
fn rules_lists_each_rule_once_however_many_packages_use_it() {
    let (_temp_dir, project) = temp_project(&runners_project_fixture());

    let rules = run_spex(&project, &["rules"], "");

    assert!(rules.status.success(), "rules should exit 0");
    assert_eq!(
        listed_ids(&rules.stdout),
        [
            "ast-grep-runner",
            "glob-runner",
            "regexp-runner",
            "script-failures",
            "script-runner"
        ]
    );
}

#[test]
fn rules_package_limits_output_to_the_rules_of_named_packages() {
    let (_temp_dir, project) = temp_project(&runners_project_fixture());

    let frontend = run_spex(&project, &["rules", "--package", "frontend"], "");
    let api = run_spex(&project, &["rules", "--package", "api"], "");
    let spex = run_spex(&project, &["rules", "--package", "spex"], "");

    assert_eq!(
        listed_ids(&frontend.stdout),
        [
            "ast-grep-runner",
            "glob-runner",
            "regexp-runner",
            "script-runner"
        ]
    );
    assert_eq!(
        listed_ids(&api.stdout),
        [
            "ast-grep-runner",
            "glob-runner",
            "regexp-runner",
            "script-failures",
            "script-runner"
        ]
    );
    assert!(spex.status.success());
    assert_eq!(String::from_utf8_lossy(&spex.stdout), "[]\n");
}

#[test]
fn rules_package_matches_a_package_by_short_or_full_name() {
    let (_temp_dir, project) = temp_project(&runners_project_fixture());

    let by_short_name = run_spex(&project, &["rules", "--package", "frontend"], "");
    let by_full_name = run_spex(
        &project,
        &["rules", "--package", "runners-project-frontend"],
        "",
    );

    assert!(by_full_name.status.success());
    assert_eq!(by_full_name.stdout, by_short_name.stdout);
}

#[test]
fn rules_id_limits_output_to_the_named_rules() {
    let (_temp_dir, project) = temp_project(&runners_project_fixture());

    let rules = run_spex(
        &project,
        &["rules", "--id", "script-failures", "glob-runner"],
        "",
    );

    assert!(rules.status.success());
    assert_eq!(
        listed_ids(&rules.stdout),
        ["glob-runner", "script-failures"]
    );
}

#[test]
fn rules_prints_the_full_rule_content_by_default() {
    let (_temp_dir, project) = temp_project(&runners_project_fixture());

    let rules = run_spex(&project, &["rules", "--id", "glob-runner"], "");

    assert!(rules.status.success());
    assert_eq!(String::from_utf8_lossy(&rules.stdout), EXPECTED_GLOB_RULE);
}

#[test]
fn rules_ai_drops_checks_and_the_sync_flag_and_adds_the_rule_path() {
    let (_temp_dir, project) = temp_project(&runners_project_fixture());

    let rules = run_spex(&project, &["rules", "--ai", "--id", "glob-runner"], "");

    assert!(rules.status.success());
    let spex_dir = canonical_display(&project.join("runners-project-spex"));
    assert_eq!(
        String::from_utf8_lossy(&rules.stdout),
        EXPECTED_GLOB_RULE_FOR_AI.replace("{spex_dir}", &spex_dir)
    );
}

#[test]
fn rules_rejects_an_unknown_rule_id() {
    let (_temp_dir, project) = temp_project(&runners_project_fixture());

    let rules = run_spex(&project, &["rules", "--id", "glob-runner", "bogus"], "");

    assert_fails_mentioning(&rules, "unknown rule id `bogus`");
    assert!(rules.stdout.is_empty());
}

#[test]
fn rules_rejects_an_unknown_package() {
    let (_temp_dir, project) = temp_project(&runners_project_fixture());

    let rules = run_spex(&project, &["rules", "--package", "bogus"], "");

    assert_fails_mentioning(&rules, "unknown package `bogus`");
    assert!(rules.stdout.is_empty());
}

#[test]
fn rules_output_flag_writes_file_matching_stdout() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");

    copy_project(&dummy_project_fixture(), &project);

    let stdout_run = run_spex(&project, &["rules"], "");
    assert!(
        stdout_run.status.success(),
        "rules failed (exit {:?})\nstderr:\n{}",
        stdout_run.status.code(),
        String::from_utf8_lossy(&stdout_run.stderr),
    );
    let stdout_yaml = String::from_utf8_lossy(&stdout_run.stdout).into_owned();
    assert!(!stdout_yaml.is_empty(), "expected rules on stdout");

    let output_path = temp_dir.path().join("rules.yaml");
    let file_run = run_spex(
        &project,
        &[
            "rules",
            "--output",
            output_path.to_str().expect("utf8 path"),
        ],
        "",
    );
    assert!(
        file_run.status.success(),
        "rules --output failed (exit {:?})\nstderr:\n{}",
        file_run.status.code(),
        String::from_utf8_lossy(&file_run.stderr),
    );
    assert!(
        file_run.stdout.is_empty(),
        "rules --output should not print to stdout"
    );

    let file_yaml = fs::read_to_string(&output_path).expect("read --output file");
    assert_eq!(
        file_yaml, stdout_yaml,
        "--output file contents should match stdout output"
    );
}
