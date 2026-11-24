use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::{Value, json};

use super::common::{assert_fails_mentioning, dummy_project_fixture, run_spex, temp_project};

fn with_dependency(project: &Path) {
    let file = project.join("dummy-project-spex/specs/planned-one.yaml");
    let mut spec = fs::read_to_string(&file).expect("read spec");
    spec.push_str("blocked_by:\n  - planned-two\n");
    fs::write(file, spec).expect("write spec");
}

fn context(project: &Path, args: &[&str]) -> Value {
    let mut command = vec!["context", "--id"];
    command.extend_from_slice(args);
    let result = run_spex(project, &command, "");
    assert!(
        result.status.success(),
        "context failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    serde_json::from_slice(&result.stdout).expect("parse context")
}

fn git(path: &Path, args: &[&str]) -> String {
    let result = Command::new("git")
        .args(args)
        .current_dir(path)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .expect("run git");
    assert!(
        result.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    String::from_utf8_lossy(&result.stdout).trim().to_owned()
}

#[test]
fn context_includes_blockers_and_explains_package_and_rule_selection() {
    let (_temp, project) = temp_project(&dummy_project_fixture());
    with_dependency(&project);
    let result = context(&project, &["planned-one"]);
    assert_eq!(result["schema_version"], 1);
    let bundle = &result["context"];
    assert_eq!(bundle["project"], "dummy-project");
    assert_eq!(bundle["prd"], "docs/prd.md");
    assert_eq!(bundle["specs"].as_array().expect("specs").len(), 2);
    assert_eq!(bundle["specs"][0]["id"], "planned-one");
    assert_eq!(bundle["specs"][0]["requested"], true);
    assert_eq!(bundle["specs"][0]["path"], "specs/planned-one.yaml");
    assert_eq!(bundle["specs"][0]["status"], "planned");
    assert_eq!(
        bundle["specs"][0]["acceptance_criteria"][0]["id"],
        "save-as-favorite"
    );
    assert!(
        bundle["specs"][0]["acceptance_criteria"][0]["line"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert_eq!(bundle["specs"][1]["id"], "planned-two");
    assert_eq!(bundle["specs"][1]["requested"], false);
    assert_eq!(bundle["specs"][1]["required_by"], json!(["planned-one"]));
    assert_eq!(bundle["packages"].as_array().expect("packages").len(), 2);
    assert_eq!(bundle["packages"][0]["name"], "dummy-project-api");
    assert_eq!(bundle["packages"][0]["required_by"], json!(["planned-two"]));
    assert_eq!(
        bundle["packages"][0]["contracts"][0]["path"],
        "./contracts.md"
    );
    assert_eq!(
        bundle["packages"][0]["contracts"][0]["sha256"]
            .as_str()
            .unwrap()
            .len(),
        64
    );
    assert_eq!(bundle["rules"][0]["id"], "no-todo-comments");
    assert_eq!(bundle["rules"][0]["packages"], json!(["dummy-project-api"]));

    let extra = context(&project, &["planned-one", "--package", "spex"]);
    assert_eq!(extra["context"]["packages"][2]["short_name"], "spex");
    assert_eq!(extra["context"]["packages"][2]["requested"], true);
}

#[test]
fn context_is_reproducible_across_roots_and_tracks_contract_changes() {
    let (_first_root, first) = temp_project(&dummy_project_fixture());
    let (_other_root, other) = temp_project(&dummy_project_fixture());
    with_dependency(&first);
    with_dependency(&other);
    let original = context(&first, &["planned-one", "planned-two"]);
    let reordered = context(&other, &["planned-two", "planned-one", "planned-two"]);
    assert_eq!(original, reordered);

    let body = "An expanded contract description for reviewers.";
    fs::write(other.join("dummy-project-spex/contracts.md"), body).expect("edit contract");
    let modified = context(&other, &["planned-one", "planned-two"]);
    assert_ne!(original["context_digest"], modified["context_digest"]);
    assert!(!modified.to_string().contains(body));
}

#[test]
fn context_records_current_revision_and_dirty_state() {
    let (_temp, project) = temp_project(&dummy_project_fixture());
    let api = project.join("dummy-project-api");
    git(&api, &["init", "-q"]);
    git(&api, &["add", "."]);
    git(
        &api,
        &[
            "-c",
            "user.name=Tester",
            "-c",
            "user.email=tester@example.com",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-qm",
            "Initial commit",
        ],
    );
    let expected_head = git(&api, &["rev-parse", "HEAD"]);
    let clean = context(&project, &["planned-two"]);
    assert_eq!(clean["context"]["packages"][0]["revision"], expected_head);
    assert_eq!(clean["context"]["packages"][0]["dirty"], false);

    fs::write(api.join("new-code.rs"), "").expect("create working-tree edit");
    let dirty = context(&project, &["planned-two"]);
    assert_eq!(dirty["context"]["packages"][0]["revision"], expected_head);
    assert_eq!(dirty["context"]["packages"][0]["dirty"], true);
    assert_ne!(clean["context_digest"], dirty["context_digest"]);
}

#[test]
fn context_requires_opt_in_for_deprecated_dependencies() {
    let (_temp, project) = temp_project(&dummy_project_fixture());
    with_dependency(&project);
    let path = project.join("dummy-project-spex/specs/planned-two.yaml");
    let spec = fs::read_to_string(&path).expect("read spec");
    fs::write(&path, spec.replace("status: planned", "status: deprecated"))
        .expect("deprecate spec");

    let rejected = run_spex(&project, &["context", "--id", "planned-one"], "");
    assert_fails_mentioning(&rejected, "spec `planned-two` is deprecated");
    let accepted = context(&project, &["planned-one", "--include-deprecated"]);
    assert_eq!(accepted["context"]["specs"][1]["status"], "deprecated");
}

#[test]
fn context_reports_missing_declared_contract() {
    let (_temp, project) = temp_project(&dummy_project_fixture());
    fs::remove_file(project.join("dummy-project-spex/contracts.md")).expect("remove test contract");
    let result = run_spex(&project, &["context", "--id", "planned-two"], "");
    assert_fails_mentioning(&result, "declared contract is unavailable");
}
