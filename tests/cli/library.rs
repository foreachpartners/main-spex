use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use tempfile::tempdir;

use crate::common::{assert_fails_mentioning, queried_by_jq, run_spex};

fn bundled_entry(id: &str) -> Value {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("templates/specs")
        .join(format!("{id}.yaml"));
    let text = fs::read_to_string(file).expect("read bundled entry");
    serde_yaml_ng::from_str(&text).expect("parse bundled entry")
}

fn bundled_rule(id: &str) -> Value {
    let file = bundled_rule_files()
        .into_iter()
        .find(|path| rule_id(path) == id)
        .expect("bundled rule");
    let text = fs::read_to_string(file).expect("read bundled rule");
    serde_yaml_ng::from_str(&text).expect("parse bundled rule")
}

fn bundled_rule_ids() -> Vec<String> {
    let mut ids: Vec<String> = bundled_rule_files()
        .iter()
        .map(|path| rule_id(path))
        .collect();
    ids.sort();
    ids
}

fn rule_id(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .expect("rule file name")
        .to_string()
}

fn bundled_rule_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rule_files(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("templates/rules"),
        &mut files,
    );
    files
}

// A subfolder groups rules unless a rule file of its name sits beside it, making it that rule's script folder.
fn collect_rule_files(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read bundled rules") {
        let path = entry.expect("rule entry").path();
        if path.is_dir() {
            if !path.with_extension("yaml").is_file() {
                collect_rule_files(&path, files);
            }
        } else if path
            .extension()
            .is_some_and(|extension| extension == "yaml")
        {
            files.push(path);
        }
    }
}

fn printed_list(stdout: &[u8]) -> Vec<Value> {
    serde_yaml_ng::from_slice(stdout).expect("stdout is a YAML list")
}

#[test]
fn lib_specs_prints_every_entry_in_full_as_a_list_outside_a_project() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(dir.path(), &["lib", "specs"], "");

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        printed_list(&output.stdout),
        [
            bundled_entry("email-policy"),
            bundled_entry("password-policy")
        ]
    );
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[test]
fn lib_specs_id_prints_the_entry_as_a_one_item_list() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(dir.path(), &["lib", "specs", "--id", "password-policy"], "");

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        printed_list(&output.stdout),
        [bundled_entry("password-policy")]
    );
}

#[test]
fn lib_specs_fields_prints_only_those_fields_of_each_entry() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(
        dir.path(),
        &["lib", "specs", "--fields", "id", "description"],
        "",
    );

    assert!(output.status.success(), "{output:?}");
    let printed = String::from_utf8_lossy(&output.stdout);
    assert!(
        printed.starts_with("- id: email-policy\n  description: "),
        "{printed}"
    );
    assert!(printed.contains("\n\n- id: password-policy\n  description: "));
    assert!(!printed.contains("acceptance_criteria"));
}

#[test]
fn lib_specs_search_prints_only_entries_containing_a_keyword() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(
        dir.path(),
        &["lib", "specs", "--search", "ARGON2ID", "--fields", "id"],
        "",
    );

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "- id: password-policy\n"
    );
}

#[test]
fn lib_specs_search_matching_nothing_prints_an_empty_list_and_nothing_found_on_stderr() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(
        dir.path(),
        &["lib", "specs", "--search", "no-such-word"],
        "",
    );

    assert!(output.status.success(), "{output:?}");
    assert_eq!(String::from_utf8_lossy(&output.stdout), "[]\n");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "nothing found\n");
}

#[test]
fn lib_specs_output_is_a_valid_yaml_list_that_jq_reads() {
    let dir = tempdir().expect("tempdir");

    let all = run_spex(dir.path(), &["lib", "specs"], "");
    let fields = run_spex(dir.path(), &["lib", "specs", "--fields", "id", "title"], "");
    let none = run_spex(
        dir.path(),
        &["lib", "specs", "--search", "no-such-word"],
        "",
    );

    for output in [&all, &fields, &none] {
        assert!(output.status.success(), "{output:?}");
    }
    assert_eq!(
        queried_by_jq(dir.path(), &all.stdout, "map(.id)"),
        json!(["email-policy", "password-policy"]),
    );
    assert_eq!(
        queried_by_jq(dir.path(), &fields.stdout, "map(keys) | unique"),
        json!([["id", "title"]]),
    );
    assert_eq!(queried_by_jq(dir.path(), &none.stdout, "length"), json!(0));
}

#[test]
fn lib_specs_rejects_an_unknown_field() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(dir.path(), &["lib", "specs", "--fields", "nope"], "");

    assert_fails_mentioning(&output, "unknown field `nope`");
}

#[test]
fn lib_specs_output_writes_the_file_instead_of_printing() {
    let dir = tempdir().expect("tempdir");
    let target = dir.path().join("out.yaml");

    let output = run_spex(
        dir.path(),
        &[
            "lib",
            "specs",
            "--id",
            "password-policy",
            "--output",
            target.to_str().expect("utf-8 path"),
        ],
        "",
    );

    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "nothing should be printed");
    assert_eq!(
        printed_list(&fs::read(target).expect("read output")),
        [bundled_entry("password-policy")]
    );
}

#[test]
fn lib_specs_rejects_an_unknown_id() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(dir.path(), &["lib", "specs", "--id", "nope"], "");

    assert_fails_mentioning(&output, "unknown id `nope`");
}

#[test]
fn lib_rejects_an_unknown_kind() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(dir.path(), &["lib", "nope"], "");

    assert_eq!(output.status.code(), Some(2));
    assert_fails_mentioning(&output, "invalid value 'nope'");
}

#[test]
fn lib_rules_prints_every_rule_in_full_as_a_list_outside_a_project() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(dir.path(), &["lib", "rules"], "");

    assert!(output.status.success(), "{output:?}");
    let expected: Vec<Value> = bundled_rule_ids()
        .iter()
        .map(|id| bundled_rule(id))
        .collect();
    assert_eq!(printed_list(&output.stdout), expected);
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[test]
fn lib_rules_prints_no_path_and_the_rules_are_synced_with_upstream() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(dir.path(), &["lib", "rules"], "");

    assert_eq!(
        queried_by_jq(
            dir.path(),
            &output.stdout,
            "map(has(\"path\") or .sync_with_upstream != true) | any"
        ),
        json!(false)
    );
}

#[test]
fn lib_rules_id_prints_only_the_named_rules_in_library_order() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(
        dir.path(),
        &["lib", "rules", "--id", "writing-style", "secret-handling"],
        "",
    );

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        printed_list(&output.stdout),
        [
            bundled_rule("secret-handling"),
            bundled_rule("writing-style")
        ]
    );
}

#[test]
fn lib_rules_fields_and_search_print_a_catalog_of_the_matches() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(
        dir.path(),
        &[
            "lib",
            "rules",
            "--search",
            "SECRET",
            "--fields",
            "id",
            "description",
        ],
        "",
    );

    assert!(output.status.success(), "{output:?}");
    let printed = printed_list(&output.stdout);
    assert!(!printed.is_empty(), "no rule matched `secret`");
    for entry in &printed {
        assert_eq!(
            entry.as_object().expect("entry").keys().collect::<Vec<_>>(),
            ["id", "description"]
        );
    }
    assert!(printed.iter().any(|entry| entry["id"] == "secret-handling"));
}

#[test]
fn lib_rules_accepts_any_field_of_the_rule_schema_and_rejects_others() {
    let dir = tempdir().expect("tempdir");

    let checks = run_spex(
        dir.path(),
        &[
            "lib",
            "rules",
            "--id",
            "secret-handling",
            "--fields",
            "checks",
        ],
        "",
    );
    let path = run_spex(dir.path(), &["lib", "rules", "--fields", "path"], "");

    assert!(checks.status.success(), "{checks:?}");
    assert!(String::from_utf8_lossy(&checks.stdout).starts_with("- checks:\n"));
    assert_fails_mentioning(&path, "unknown field `path`");
}

#[test]
fn lib_rules_search_matching_nothing_prints_an_empty_list_and_nothing_found_on_stderr() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(
        dir.path(),
        &["lib", "rules", "--search", "no-such-word"],
        "",
    );

    assert!(output.status.success(), "{output:?}");
    assert_eq!(String::from_utf8_lossy(&output.stdout), "[]\n");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "nothing found\n");
}

#[test]
fn lib_rules_rejects_an_unknown_id() {
    let dir = tempdir().expect("tempdir");

    let output = run_spex(dir.path(), &["lib", "rules", "--id", "nope"], "");

    assert_fails_mentioning(&output, "unknown id `nope`");
}

#[test]
fn lib_rules_copy_writes_each_rule_with_its_executable_scripts_and_prints_the_rule_paths() {
    let dir = tempdir().expect("tempdir");
    let rules = dir.path().join("proj-spex/rules");

    let output = run_spex(
        dir.path(),
        &[
            "lib",
            "rules",
            "--id",
            "writing-style",
            "secret-handling",
            "--copy",
            rules.to_str().expect("utf-8 path"),
        ],
        "",
    );

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!(
            "{}\n{}\n",
            rules.join("secret-handling.yaml").display(),
            rules.join("writing-style.yaml").display()
        )
    );
    let library = Path::new(env!("CARGO_MANIFEST_DIR")).join("templates/rules");
    for file in [
        "secret-handling.yaml",
        "writing-style.yaml",
        "writing-style/writing-style.sh",
    ] {
        assert_eq!(
            fs::read(rules.join(file)).expect("copied file"),
            fs::read(library.join(file)).expect("bundled file"),
            "{file}"
        );
    }
    assert!(!rules.join("secret-handling").exists());
    let mode = fs::metadata(rules.join("writing-style/writing-style.sh"))
        .expect("script metadata")
        .permissions()
        .mode();
    assert_ne!(mode & 0o111, 0, "script is not executable");
}

#[test]
fn lib_rules_copy_never_overwrites_a_rule() {
    let dir = tempdir().expect("tempdir");
    let rules = dir.path().join("rules");
    fs::create_dir(&rules).expect("create rules");
    fs::write(rules.join("secret-handling.yaml"), "mine\n").expect("seed rule");

    let output = run_spex(
        dir.path(),
        &[
            "lib",
            "rules",
            "--id",
            "writing-style",
            "secret-handling",
            "--copy",
            rules.to_str().expect("utf-8 path"),
        ],
        "",
    );

    assert_fails_mentioning(&output, "secret-handling.yaml");
    assert_fails_mentioning(&output, "already exists");
    assert_eq!(
        fs::read_to_string(rules.join("secret-handling.yaml")).expect("read rule"),
        "mine\n"
    );
    assert!(!rules.join("writing-style.yaml").exists());
}

#[test]
fn lib_copy_needs_an_id_and_refuses_options_that_print() {
    let dir = tempdir().expect("tempdir");
    let target = dir.path().join("out");
    let target = target.to_str().expect("utf-8 path");

    let without_id = run_spex(dir.path(), &["lib", "rules", "--copy", target], "");
    assert_eq!(without_id.status.code(), Some(2));
    assert_fails_mentioning(&without_id, "--id");

    for option in [
        ["--fields", "id"],
        ["--search", "secret"],
        ["--output", "out.yaml"],
    ] {
        let mut args = vec!["lib", "rules", "--id", "secret-handling", "--copy", target];
        args.extend(option);
        let output = run_spex(dir.path(), &args, "");

        assert_eq!(output.status.code(), Some(2), "{option:?}");
        assert_fails_mentioning(&output, "cannot be used with");
    }
    assert!(!dir.path().join("out").exists(), "nothing may be written");
}

#[test]
fn lib_specs_copy_is_rejected() {
    let dir = tempdir().expect("tempdir");
    let target = dir.path().join("out");

    let output = run_spex(
        dir.path(),
        &[
            "lib",
            "specs",
            "--id",
            "password-policy",
            "--copy",
            target.to_str().expect("utf-8 path"),
        ],
        "",
    );

    assert_fails_mentioning(&output, "`--copy` copies rules only");
    assert!(!target.exists(), "nothing may be written");
}
