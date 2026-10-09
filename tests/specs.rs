mod common;
use common::{
    assert_fails_mentioning, canonical_display, dummy_project_fixture, listed_ids, run_spex,
    temp_project,
};

const EXPECTED_SPECS_FOR_AI: &str = r#"- id: active-with-gap
  path: {spex_dir}/specs/active-with-gap.yaml
  title: Validate registration input
  description: Validates a new account's registration input before it is created, rejecting invalid or duplicate submissions.
  acceptance_criteria:
  - id: validates-input
    must: reject a registration request missing a required field
  - id: normalizes-format
    must: normalize the submitted email to lowercase before storing it
  - id: rejects-duplicate
    must: reject a registration request for an email already registered
  - id: logs-attempt
    must: log every registration attempt, successful or not
  scope:
  - frontend
  status: active
  contract: single

- id: planned-one
  path: {spex_dir}/specs/planned-one.yaml
  title: Let customers save a favorite order
  description: Lets a returning customer save an order as a favorite so they can reorder it in one click next time.
  acceptance_criteria:
  - id: save-as-favorite
    must: let a customer save a completed order as a favorite
    is_done: false
  scope:
  - frontend
  status: planned
  contract: single
"#;

#[test]
fn specs_id_limits_output_to_the_named_specs() {
    let (_workspace, project) = temp_project(&dummy_project_fixture());

    let specs = run_spex(
        &project,
        &["specs", "--id", "planned-one", "active-with-gap"],
        "",
    );

    assert!(specs.status.success(), "specs should exit 0");
    assert_eq!(
        listed_ids(&specs.stdout),
        ["active-with-gap", "planned-one"]
    );
    assert!(
        !String::from_utf8_lossy(&specs.stdout).contains("path:"),
        "specs should not print paths without --ai",
    );
}

#[test]
fn specs_ai_adds_the_spec_path_after_the_id() {
    let (_workspace, project) = temp_project(&dummy_project_fixture());

    let specs = run_spex(
        &project,
        &["specs", "--ai", "--id", "planned-one", "active-with-gap"],
        "",
    );

    assert!(specs.status.success());
    let spex_dir = canonical_display(&project.join("dummy-project-spex"));
    assert_eq!(
        String::from_utf8_lossy(&specs.stdout),
        EXPECTED_SPECS_FOR_AI.replace("{spex_dir}", &spex_dir)
    );
}

#[test]
fn specs_scope_limits_output_to_specs_in_that_scope() {
    let (_workspace, project) = temp_project(&dummy_project_fixture());

    let specs = run_spex(&project, &["specs", "--scope", "frontend"], "");

    assert!(specs.status.success());
    assert_eq!(
        listed_ids(&specs.stdout),
        [
            "active-with-gap",
            "foreach-not-covered",
            "in-progress-mixed",
            "planned-one"
        ]
    );
}

#[test]
fn specs_scope_lists_a_spec_in_several_scopes_once_and_skips_unscoped_specs() {
    let (_workspace, project) = temp_project(&dummy_project_fixture());

    let everything = run_spex(&project, &["specs"], "");
    let scoped = run_spex(&project, &["specs", "--scope", "api", "frontend"], "");

    let all_ids = listed_ids(&everything.stdout);
    let scoped_ids = listed_ids(&scoped.stdout);
    let mut distinct = scoped_ids.clone();
    distinct.dedup();
    assert_eq!(
        scoped_ids, distinct,
        "a spec in both scopes was listed twice"
    );
    assert!(scoped_ids.contains(&"in-progress-mixed".to_string()));
    assert_eq!(
        all_ids
            .iter()
            .filter(|id| !scoped_ids.contains(id))
            .collect::<Vec<_>>(),
        ["in-progress-pending"],
        "only the unscoped spec should be left out",
    );
}

#[test]
fn specs_rejects_an_unknown_spec_id() {
    let (_workspace, project) = temp_project(&dummy_project_fixture());

    let specs = run_spex(&project, &["specs", "--id", "planned-one", "bogus"], "");

    assert_fails_mentioning(&specs, "unknown spec id `bogus`");
    assert!(specs.stdout.is_empty());
}

#[test]
fn specs_rejects_an_unknown_scope() {
    let (_workspace, project) = temp_project(&dummy_project_fixture());

    let specs = run_spex(&project, &["specs", "--scope", "bogus"], "");

    assert_fails_mentioning(&specs, "unknown repository short name `bogus`");
    assert!(specs.stdout.is_empty());
}
