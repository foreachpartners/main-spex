use std::fs;
use std::io::Write;

use tempfile::tempdir;

use crate::common;
use crate::common::{
    assert_fails_mentioning, dummy_project_fixture, report_text, run_spex, with_version_header,
};

fn waive_in_package_entry(project: &std::path::Path, entry_end: &str, waivers: &str) {
    let spex_yaml = project.join("dummy-project-spex/spex.yaml");
    let config = fs::read_to_string(&spex_yaml).expect("read spex.yaml");
    let indented = waivers.replace('\n', "\n    ");
    let patched = config.replace(
        &format!("{entry_end}\n"),
        &format!("{entry_end}{}", indented.trim_end_matches(' ')),
    );
    assert_ne!(patched, config, "package entry not found in spex.yaml");
    fs::write(&spex_yaml, patched).expect("write spex.yaml");
}

fn append(path: &std::path::Path, text: &str) {
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(path)
        .expect("open file for append");
    file.write_all(text.as_bytes()).expect("append to file");
}

const API_ENTRY_END: &str = "    rules:\n      - no-todo-comments";

const FRONTEND_ENTRY_END: &str = "    path: ../dummy-project-frontend\n    rules: []";

const NONE_WITH_GAP_WAIVERS: &str = r#"
coverage_ignore:
  - spec_id: none-with-gap
    layer: impl
    ac: resumes-favorite
    reason: waived
  - spec_id: none-with-gap
    layer: test
    ac: resumes-favorite
    reason: waived
"#;

const EXPECTED_COVERAGE: &str = include_str!("../fixtures/dummy-project/expected_coverage.txt");

const EXPECTED_COVERAGE_VERBOSE: &str =
    include_str!("../fixtures/dummy-project/expected_coverage_verbose.txt");

const EXPECTED_COVERAGE_PACKAGE_FRONTEND: &str =
    include_str!("../fixtures/dummy-project/expected_coverage_package_frontend.txt");

const NONE_WITH_GAP_IMPL_WAIVER: &str = r#"
coverage_ignore:
  - spec_id: none-with-gap
    layer: impl
    ac: resumes-favorite
    reason: waived
"#;

const NONE_WITH_GAP_TEST_WAIVER: &str = r#"
coverage_ignore:
  - spec_id: none-with-gap
    layer: test
    ac: resumes-favorite
    reason: waived
"#;

const IN_PROGRESS_SPEC: &str = r#"id: ci-check
title: Verify a change before its spec is updated
description: >-
  A criterion still flagged not done in the spec package.
acceptance_criteria:
  - id: rejects-empty-cart
    must: "reject checkout of an empty cart"
    is_done: false
packages:
  - api
status: in-progress
contract: single
"#;

#[test]
fn coverage_report_matches_expected() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);

    let coverage = run_spex(&project, &["coverage"], "");
    assert_eq!(
        report_text(&coverage.stdout),
        with_version_header(EXPECTED_COVERAGE),
        "coverage report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&coverage.stderr),
    );
    assert_eq!(
        coverage.status.code(),
        Some(1),
        "coverage should exit 1 when an active spec has a gap"
    );
}

#[test]
fn coverage_verbose_report_matches_expected() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);

    let coverage = run_spex(&project, &["coverage", "--verbose"], "");
    assert_eq!(
        report_text(&coverage.stdout),
        with_version_header(EXPECTED_COVERAGE_VERBOSE),
        "coverage --verbose report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&coverage.stderr),
    );
    assert_eq!(
        coverage.status.code(),
        Some(1),
        "coverage should exit 1 when an active spec has a gap"
    );
}

#[test]
fn coverage_package_limits_the_report_to_specs_of_that_package() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);

    let coverage = run_spex(&project, &["coverage", "--package", "frontend"], "");
    assert_eq!(
        report_text(&coverage.stdout),
        with_version_header(EXPECTED_COVERAGE_PACKAGE_FRONTEND),
        "coverage --package report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&coverage.stderr),
    );
    assert_eq!(
        coverage.status.code(),
        Some(1),
        "coverage should exit 1 when an active spec of the package has a gap"
    );
}

#[test]
fn coverage_package_accepts_a_full_package_name() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);

    let by_short_name = run_spex(&project, &["coverage", "--package", "frontend"], "");
    let by_full_name = run_spex(
        &project,
        &["coverage", "--package", "dummy-project-frontend"],
        "",
    );

    assert_eq!(
        report_text(&by_full_name.stdout),
        report_text(&by_short_name.stdout)
    );
}

#[test]
fn coverage_package_with_several_packages_counts_a_spec_in_both_once() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);

    let everything = report_text(&run_spex(&project, &["coverage"], "").stdout);
    let packaged =
        report_text(&run_spex(&project, &["coverage", "--package", "api", "frontend"], "").stdout);

    assert!(everything.contains("Specs (13)"), "{everything}");
    assert_eq!(packaged, everything);
}

#[test]
fn coverage_package_rejects_an_unknown_package() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);

    let coverage = run_spex(&project, &["coverage", "--package", "bogus"], "");

    assert_fails_mentioning(&coverage, "unknown package `bogus`");
    assert!(coverage.stdout.is_empty());
}

#[test]
fn coverage_fails_an_in_progress_not_done_criterion_once_its_impl_lacks_a_test() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);
    let specs = project.join("dummy-project-spex/specs");
    fs::remove_dir_all(&specs).expect("clear specs");
    fs::create_dir(&specs).expect("recreate specs");
    fs::write(specs.join("ci-check.yaml"), IN_PROGRESS_SPEC).expect("write spec");
    let contracts = project.join("dummy-project-spex/contracts.md");
    let api_source = project.join("dummy-project-api/src/lib.rs");
    fs::write(&api_source, "").expect("clear api source");
    fs::write(&contracts, "<!-- @spec contract ci-check -->\n").expect("write contract");

    let nothing_started = run_spex(&project, &["coverage"], "");
    assert!(
        report_text(&nothing_started.stdout)
            .contains("●  ci-check  0/1 AC  in-progress: impl(rejects-empty-cart)\n"),
        "{}",
        report_text(&nothing_started.stdout),
    );
    assert_eq!(
        nothing_started.status.code(),
        Some(0),
        "a criterion not started yet is pending, not an error"
    );

    append(
        &api_source,
        "// @spec impl ci-check#rejects-empty-cart\nfn reject_empty_cart() {}\n",
    );
    let implemented = run_spex(&project, &["coverage"], "");
    assert!(
        report_text(&implemented.stdout)
            .contains("✗  ci-check  1/1 AC  missing: test(rejects-empty-cart)"),
        "{}",
        report_text(&implemented.stdout),
    );
    assert_eq!(
        implemented.status.code(),
        Some(1),
        "an implemented criterion without a test is an error even while flagged not done"
    );

    append(
        &api_source,
        "// @spec test ci-check#rejects-empty-cart\nfn reject_empty_cart_test() {}\n",
    );
    let tested = run_spex(&project, &["coverage"], "");
    assert_eq!(
        tested.status.code(),
        Some(0),
        "{}",
        report_text(&tested.stdout)
    );
}

#[test]
fn coverage_honors_a_spex_yaml_package_entry_coverage_ignore_for_a_spec() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);
    waive_in_package_entry(&project, API_ENTRY_END, NONE_WITH_GAP_WAIVERS);

    let coverage = run_spex(&project, &["coverage"], "");

    let report = report_text(&coverage.stdout);
    assert!(
        !report.contains("none-with-gap"),
        "the waived spec should have no gap\n{report}"
    );
    assert!(
        report.ends_with("planned (2), in-progress (2), active (9), gaps (6)\n"),
        "{report}"
    );
}

#[test]
fn coverage_honors_a_package_spex_repo_yaml_coverage_ignore() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);
    fs::write(
        project.join("dummy-project-api/spex.package.yaml"),
        NONE_WITH_GAP_WAIVERS.trim_start(),
    )
    .expect("write spex.package.yaml");

    let coverage = run_spex(&project, &["coverage"], "");

    let report = report_text(&coverage.stdout);
    assert!(
        !report.contains("none-with-gap"),
        "the waived spec should have no gap\n{report}"
    );
    assert!(
        report.ends_with("planned (2), in-progress (2), active (9), gaps (6)\n"),
        "{report}"
    );
}

#[test]
fn coverage_merges_coverage_ignore_from_the_spex_yaml_entry_and_the_spex_package_yaml_of_a_package()
{
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);

    waive_in_package_entry(&project, API_ENTRY_END, NONE_WITH_GAP_IMPL_WAIVER);
    let partly_waived = report_text(&run_spex(&project, &["coverage"], "").stdout);
    assert!(
        partly_waived.contains("missing: test(resumes-favorite)"),
        "waiving only impl should leave the test gap\n{partly_waived}"
    );
    assert!(
        !partly_waived.contains("impl(resumes-favorite)"),
        "the impl waiver should apply\n{partly_waived}"
    );

    fs::write(
        project.join("dummy-project-api/spex.package.yaml"),
        NONE_WITH_GAP_TEST_WAIVER.trim_start(),
    )
    .expect("write spex.package.yaml");
    let fully_waived = report_text(&run_spex(&project, &["coverage"], "").stdout);
    assert!(
        !fully_waived.contains("none-with-gap"),
        "the impl and test waivers should merge\n{fully_waived}"
    );
}

#[test]
fn coverage_skips_annotations_in_the_files_a_spex_yaml_package_exclude_matches() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);
    let before = report_text(&run_spex(&project, &["coverage"], "").stdout);
    assert!(!before.contains("active-full-coverage"), "{before}");

    waive_in_package_entry(&project, API_ENTRY_END, "\nexclude:\n  - src/lib.rs\n");
    let coverage = run_spex(&project, &["coverage"], "");

    let report = report_text(&coverage.stdout);
    assert!(
        report.contains("active-full-coverage"),
        "the spec traced only from the excluded file should now have a gap\n{report}"
    );
    assert_eq!(coverage.status.code(), Some(1), "{report}");
}

#[test]
fn coverage_skips_annotations_in_the_folders_a_package_spex_package_yaml_excludes() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);
    fs::write(
        project.join("dummy-project-api/spex.package.yaml"),
        "exclude:\n  - src\n",
    )
    .expect("write spex.package.yaml");

    let report = report_text(&run_spex(&project, &["coverage"], "").stdout);

    assert!(report.contains("active-full-coverage"), "{report}");
}

#[test]
fn coverage_does_not_apply_the_exclude_of_one_package_to_another() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);
    let before = report_text(&run_spex(&project, &["coverage"], "").stdout);
    assert!(before.contains("in-progress-pending  1/2 AC"), "{before}");
    fs::write(
        project.join("dummy-project-frontend/spex.package.yaml"),
        "exclude:\n  - src\n",
    )
    .expect("write spex.package.yaml");

    let report = report_text(&run_spex(&project, &["coverage"], "").stdout);

    assert!(
        report.contains("in-progress-pending  0/2 AC"),
        "the frontend exclude should drop its own annotations\n{report}"
    );
    assert!(
        !report.contains("active-full-coverage"),
        "the frontend exclude should not hide the api annotations\n{report}"
    );
}

#[test]
fn coverage_fails_naming_the_package_on_an_exclude_that_is_not_a_valid_glob() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);
    fs::write(
        project.join("dummy-project-api/spex.package.yaml"),
        "exclude:\n  - \"src/{a,b\"\n",
    )
    .expect("write spex.package.yaml");

    let coverage = run_spex(&project, &["coverage"], "");

    assert_fails_mentioning(
        &coverage,
        "invalid `exclude` of package `dummy-project-api`",
    );
}

#[test]
fn coverage_ignores_a_package_waiver_of_a_package_the_spec_does_not_list() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);
    fs::write(
        project.join("dummy-project-frontend/spex.package.yaml"),
        NONE_WITH_GAP_WAIVERS.trim_start(),
    )
    .expect("write spex.package.yaml");

    let coverage = run_spex(&project, &["coverage"], "");

    let report = report_text(&coverage.stdout);
    assert!(
        report.contains("none-with-gap"),
        "a frontend waiver should not waive a spec that lists only api\n{report}"
    );
}

#[test]
fn coverage_waives_a_requirement_only_once_every_listed_package_waives_it() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);
    let spec_path = project.join("dummy-project-spex/specs/none-with-gap.yaml");
    let spec = fs::read_to_string(&spec_path).expect("read spec");
    let listing_two_packages =
        spec.replace("packages:\n  - api\n", "packages:\n  - api\n  - frontend\n");
    assert_ne!(listing_two_packages, spec, "packages not found in the spec");
    fs::write(&spec_path, listing_two_packages).expect("write spec");

    fs::write(
        project.join("dummy-project-api/spex.package.yaml"),
        NONE_WITH_GAP_WAIVERS.trim_start(),
    )
    .expect("write spex.package.yaml");
    let waived_by_api_only = report_text(&run_spex(&project, &["coverage"], "").stdout);
    assert!(
        waived_by_api_only.contains("none-with-gap"),
        "a waiver of api alone should not waive a spec that lists api and frontend\n{waived_by_api_only}"
    );

    waive_in_package_entry(&project, FRONTEND_ENTRY_END, NONE_WITH_GAP_WAIVERS);
    let waived_by_both = report_text(&run_spex(&project, &["coverage"], "").stdout);
    assert!(
        !waived_by_both.contains("none-with-gap"),
        "api and frontend both waive the requirement\n{waived_by_both}"
    );
    assert!(
        waived_by_both.ends_with("planned (2), in-progress (2), active (9), gaps (6)\n"),
        "{waived_by_both}"
    );
}

#[test]
fn coverage_fails_on_an_invalid_spex_repo_yaml() {
    let temp_dir = tempdir().expect("create temp dir");
    let project = temp_dir.path().join("proj");
    common::copy_project(&dummy_project_fixture(), &project);
    fs::write(
        project.join("dummy-project-api/spex.package.yaml"),
        "coverage_ignore:\n  - spec_id: Not_A_Spec_Id\n    layer: impl\n    reason: waived\n",
    )
    .expect("write spex.package.yaml");

    let coverage = run_spex(&project, &["coverage"], "");

    assert_fails_mentioning(&coverage, "spex.package.yaml");
    assert!(coverage.stdout.is_empty());
}

const TRACE_PROBE_SPEC: &str = "id: checkout-validation
title: Validate checkout
description: Reject incomplete checkout requests.
acceptance_criteria:
  - id: rejects-empty-cart
    must: Reject an empty cart.
packages:
  - api
status: active
contract: single
";

fn trace_probe() -> (tempfile::TempDir, std::path::PathBuf) {
    let (temp_dir, project) = common::temp_project(&dummy_project_fixture());
    fs::write(
        project.join("dummy-project-spex/specs/checkout-validation.yaml"),
        TRACE_PROBE_SPEC,
    )
    .expect("write probe spec");
    fs::write(
        project.join("dummy-project-frontend/notes.md"),
        "Checkout notes\n\n@spec impl checkout-validation#rejects-empty-cart\n@spec test checkout-validation#rejects-empty-cart\n@spec contract checkout-validation\n",
    )
    .expect("write prose traces");
    (temp_dir, project)
}

fn report_json(output: &std::process::Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "expected JSON: {error}\n{}",
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

#[test]
fn coverage_strict_rejects_markers_in_prose_and_undeclared_contracts() {
    let (_temp_dir, project) = trace_probe();
    let legacy = run_spex(
        &project,
        &[
            "coverage",
            "--id",
            "checkout-validation",
            "--format",
            "json",
        ],
        "",
    );
    assert!(legacy.status.success());
    assert_eq!(report_json(&legacy)["specs"][0]["state"], "covered");

    let strict = run_spex(
        &project,
        &[
            "coverage",
            "--id",
            "checkout-validation",
            "--strict",
            "--format",
            "json",
        ],
        "",
    );
    assert_eq!(strict.status.code(), Some(1));
    let report = report_json(&strict);
    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["command"], "coverage");
    assert_eq!(report["cli_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(report["test_execution"], "not-run");
    assert_eq!(report["strict"], true);
    let spec = &report["specs"][0];
    assert_eq!(spec["id"], "checkout-validation");
    assert_eq!(spec["state"], "gap");
    assert_eq!(spec["contract"], "gap");
    assert_eq!(
        spec["criteria"][0]["missing"],
        serde_json::json!(["impl", "test"])
    );
    let traces = spec["traces"].as_array().expect("trace list");
    assert_eq!(traces.len(), 3);
    assert_eq!(traces[0]["package"], "dummy-project-frontend");
    assert!(
        traces[0]["path"]
            .as_str()
            .expect("trace path")
            .ends_with("notes.md")
    );
    assert_eq!(traces[0]["line"], 3);
    assert_eq!(traces[0]["rejection"], "prose-file");
    assert_eq!(traces[1]["layer"], "test");
    assert_eq!(traces[1]["rejection"], "prose-file");
    assert_eq!(traces[2]["line"], 5);
    assert_eq!(traces[2]["rejection"], "undeclared-contract");

    let text = run_spex(&project, &["coverage", "--strict"], "");
    let body = report_text(&text.stdout);
    assert!(body.contains("Trace evidence (tests not run)"));
    assert!(body.contains("notes.md:3 (rejected: prose-file)"));
    assert!(body.contains("notes.md:5 (rejected: undeclared-contract)"));
}

#[test]
fn coverage_strict_keeps_cross_package_sources_and_declared_contracts() {
    let (_temp_dir, project) = trace_probe();
    let declared = project.join("dummy-project-frontend/contracts.md");
    fs::rename(project.join("dummy-project-spex/contracts.md"), &declared)
        .expect("move declared contract outside its owning package");
    let config = project.join("dummy-project-spex/spex.yaml");
    let contents = fs::read_to_string(&config).expect("read config");
    fs::write(
        config,
        contents.replace("./contracts.md", "../dummy-project-frontend/contracts.md"),
    )
    .expect("assign contract to the API package");
    append(&declared, "<!-- @spec contract checkout-validation -->\n");
    append(
        &project.join("dummy-project-frontend/src/index.ts"),
        "// @spec impl checkout-validation#rejects-empty-cart\nconst validCart = (items: string[]) => items.length > 0;\n\n// @spec test checkout-validation#rejects-empty-cart\nif (validCart([])) throw new Error('Empty cart should fail');\n",
    );
    let args = [
        "coverage",
        "--package",
        "api",
        "--id",
        "checkout-validation",
        "--strict",
        "--format",
        "json",
    ];
    let output = run_spex(&project, &args, "");
    assert_eq!(output.status.code(), Some(0));
    let report = report_json(&output);
    assert_eq!(report["specs"][0]["state"], "covered");
    let traces = report["specs"][0]["traces"].as_array().expect("trace list");
    assert_eq!(
        traces.len(),
        6,
        "a declared contract is scanned once per owner"
    );
    let eligible: Vec<_> = traces
        .iter()
        .filter(|trace| trace["rejection"].is_null())
        .collect();
    assert_eq!(eligible.len(), 3);
    assert_eq!(eligible[0]["layer"], "contract");
    assert_eq!(eligible[0]["declared_contract"], true);
    assert_eq!(eligible[0]["package"], "dummy-project-api");
    assert_eq!(eligible[1]["package"], "dummy-project-frontend");
    assert_eq!(eligible[1]["ac_id"], "rejects-empty-cart");
    assert_eq!(eligible[2]["layer"], "test");
    assert_eq!(output.stdout, run_spex(&project, &args, "").stdout);
}

#[test]
fn coverage_reports_missing_packages_and_applies_the_selected_scope() {
    let (temp_dir, project) = trace_probe();
    fs::rename(
        project.join("dummy-project-api"),
        temp_dir.path().join("omitted-api"),
    )
    .expect("remove API from the project");

    let required = run_spex(
        &project,
        &[
            "coverage",
            "--package",
            "api",
            "--id",
            "checkout-validation",
            "--format",
            "json",
        ],
        "",
    );
    assert_eq!(required.status.code(), Some(1));
    let report = report_json(&required);
    assert_eq!(report["summary"]["gapped_specs"], 0);
    assert_eq!(report["summary"]["blocking_unavailable_inputs"], 1);
    let missing = &report["unavailable_inputs"][0];
    assert_eq!(missing["kind"], "package");
    assert_eq!(missing["package"], "dummy-project-api");
    assert_eq!(missing["state"], "missing");
    assert_eq!(missing["blocking"], true);

    let args = [
        "coverage",
        "--package",
        "frontend",
        "--id",
        "in-progress-pending",
        "--format",
        "json",
    ];
    let selected = run_spex(&project, &args, "");
    assert_eq!(selected.status.code(), Some(0));
    assert_eq!(
        report_json(&selected)["unavailable_inputs"][0]["blocking"],
        false
    );

    let mut strict_args = args.to_vec();
    strict_args.push("--strict");
    let strict = run_spex(&project, &strict_args, "");
    assert_eq!(strict.status.code(), Some(1));
    assert_eq!(
        report_json(&strict)["unavailable_inputs"][0]["blocking"],
        true
    );
}

#[test]
fn coverage_reports_unavailable_declared_contracts_even_with_legacy_traces() {
    let (temp_dir, project) = trace_probe();
    let contract = project.join("dummy-project-spex/contracts.md");
    fs::rename(&contract, temp_dir.path().join("omitted-contract.md"))
        .expect("remove contract from the project");
    let args = [
        "coverage",
        "--id",
        "checkout-validation",
        "--format",
        "json",
    ];
    let missing = run_spex(&project, &args, "");
    assert_eq!(missing.status.code(), Some(1));
    let report = report_json(&missing);
    assert_eq!(report["summary"]["gapped_specs"], 0);
    assert_eq!(report["unavailable_inputs"][0]["kind"], "contract");
    assert_eq!(report["unavailable_inputs"][0]["state"], "missing");

    fs::write(contract, [0xff]).expect("write an unreadable text contract");
    let unreadable = run_spex(&project, &args, "");
    assert_eq!(unreadable.status.code(), Some(1));
    assert_eq!(
        report_json(&unreadable)["unavailable_inputs"][0]["state"],
        "unreadable"
    );
}

#[test]
fn coverage_reports_waiver_reasons_and_effective_layers() {
    let (_temp_dir, project) = common::temp_project(&dummy_project_fixture());
    waive_in_package_entry(&project, API_ENTRY_END, NONE_WITH_GAP_TEST_WAIVER);
    let report = report_json(&run_spex(
        &project,
        &["coverage", "--id", "none-with-gap", "--format", "json"],
        "",
    ));
    let spec = &report["specs"][0];
    assert_eq!(spec["waivers"][0]["package"], "api");
    assert_eq!(spec["waivers"][0]["reason"], "waived");
    let criterion = spec["criteria"]
        .as_array()
        .expect("criteria")
        .iter()
        .find(|criterion| criterion["id"] == "resumes-favorite")
        .expect("waived criterion");
    assert_eq!(criterion["waived"], serde_json::json!(["test"]));
    assert_eq!(criterion["missing"], serde_json::json!(["impl"]));
}

#[cfg(unix)]
#[test]
fn coverage_reports_an_unreadable_package_tree_and_keeps_scanning_siblings() {
    use std::os::unix::fs::PermissionsExt;

    let (_temp_dir, project) = trace_probe();
    let source_dir = project.join("dummy-project-api/src");
    let permissions = fs::metadata(&source_dir)
        .expect("source directory metadata")
        .permissions();
    fs::set_permissions(&source_dir, fs::Permissions::from_mode(0o000))
        .expect("restrict the source directory");
    let inaccessible = fs::read_dir(&source_dir).is_err();
    let output = run_spex(
        &project,
        &[
            "coverage",
            "--id",
            "checkout-validation",
            "--format",
            "json",
        ],
        "",
    );
    fs::set_permissions(&source_dir, permissions).expect("restore source directory permissions");

    if inaccessible {
        assert_eq!(output.status.code(), Some(1));
        let report = report_json(&output);
        assert_eq!(report["specs"][0]["state"], "covered");
        assert_eq!(report["unavailable_inputs"][0]["state"], "unreadable");
        assert_eq!(
            report["unavailable_inputs"][0]["package"],
            "dummy-project-api"
        );
    }
}
