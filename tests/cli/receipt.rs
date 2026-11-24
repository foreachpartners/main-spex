use std::fs;
use std::path::Path;
use std::process::Output;
use std::thread;

use serde_json::{Value, json};

use super::common::{assert_fails_mentioning, run_spex};

fn example(id: &str) -> Value {
    json!({
        "schema_version": 1,
        "id": id,
        "context_digest": "b".repeat(64),
        "revisions": {"api": "a".repeat(40)},
        "decision": {
            "approval": "approved",
            "reason": "Use the existing order endpoint.",
            "reference": "issues/42",
        },
        "verification": {
            "rules": {"status": "passed", "reference": "artifacts/rules.json"},
            "trace_coverage": {"status": "failed", "reference": "artifacts/trace.json"},
            "contract_validation": {"status": "unavailable"},
            "executed_tests": {"status": "not-run"},
            "manual_acceptance": {"status": "waived", "reference": "issues/42"},
        },
        "exception_refs": ["issues/42"],
        "unresolved_refs": ["issues/44"],
    })
}

fn append(root: &Path, body: &Value) -> Output {
    run_spex(
        root,
        &["receipt", "--journal", "journal", "append", "-"],
        &body.to_string(),
    )
}

fn json_result(output: Output) -> Value {
    assert!(
        output.status.success(),
        "receipt failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("parse receipt output")
}

#[test]
fn receipt_roundtrips_distinct_stage_evidence_and_lineage() {
    let temp = tempfile::tempdir().expect("create audit root");
    let root = temp.path();
    let original = example("change-42");
    let stored = json_result(append(root, &original));
    assert_eq!(stored["receipt"], original);
    assert!(stored["appended_at_unix_seconds"].as_u64().unwrap() > 0);

    let mut fork = example("branch-42");
    fork["parent"] = json!("change-42");
    fs::write(root.join("fork.json"), fork.to_string()).expect("write fork source");
    let fork_record = json_result(run_spex(
        root,
        &["receipt", "--journal", "journal", "append", "fork.json"],
        "",
    ));
    assert_eq!(fork_record["receipt"], fork);
    let shown = json_result(run_spex(
        root,
        &["receipt", "--journal", "journal", "show", "change-42"],
        "",
    ));
    assert_eq!(shown, stored);
    let list = json_result(run_spex(
        root,
        &["receipt", "--journal", "journal", "list"],
        "",
    ));
    assert_eq!(list[0]["id"], "branch-42");
    assert_eq!(list[0]["parent"], "change-42");
    assert_eq!(list[1]["id"], "change-42");
    assert_eq!(list.as_array().expect("receipt list").len(), 2);
}

#[test]
fn receipt_append_is_idempotent_and_preserves_existing_data() {
    let temp = tempfile::tempdir().expect("create audit root");
    let root = temp.path();
    let mut receipt = example("same-id");
    let original = append(root, &receipt);
    let repeated = append(root, &receipt);
    assert!(original.status.success());
    assert!(repeated.status.success());
    assert_eq!(original.stdout, repeated.stdout);

    let stored = root.join("journal/same-id.json");
    let original_bytes = fs::read(&stored).expect("read stored receipt");
    receipt["decision"]["reason"] = json!("Choose a different endpoint.");
    let changed = append(root, &receipt);
    assert_fails_mentioning(&changed, "already exists with different data");
    assert_eq!(
        fs::read(stored).expect("read preserved receipt"),
        original_bytes
    );
    assert_eq!(
        fs::read_dir(root.join("journal"))
            .expect("read journal")
            .count(),
        1
    );
}

#[test]
fn receipt_has_one_winner_when_concurrent_writers_disagree() {
    let temp = tempfile::tempdir().expect("create audit root");
    let root = temp.path();
    let original = example("racing-writers");
    let mut different = original.clone();
    different["decision"]["reason"] = json!("Keep the new endpoint.");
    let (first, second) = thread::scope(|scope| {
        let first = scope.spawn(|| append(root, &original));
        let second = scope.spawn(|| append(root, &different));
        (
            first.join().expect("first append"),
            second.join().expect("second append"),
        )
    });
    assert_ne!(first.status.success(), second.status.success());
    let (winner, loser) = if first.status.success() {
        (first, second)
    } else {
        (second, first)
    };
    assert_fails_mentioning(&loser, "already exists with different data");

    let persisted: Value = serde_json::from_slice(
        &fs::read(root.join("journal/racing-writers.json")).expect("read winning receipt"),
    )
    .expect("parse winning receipt");
    assert_eq!(persisted, json_result(winner));
}

#[test]
fn receipt_rejects_anonymous_gates_bad_ids_and_missing_parents() {
    let temp = tempfile::tempdir().expect("create audit root");
    let root = temp.path();
    let path_escape = append(root, &example("../outside"));
    assert_fails_mentioning(&path_escape, "invalid receipt ID");

    let mut receipt = example("missing-proof");
    receipt["verification"]["executed_tests"] = json!({"status": "passed"});
    let false_pass = append(root, &receipt);
    assert_fails_mentioning(&false_pass, "`executed_tests` needs an evidence");

    let mut receipt = example("unresolved-ancestor");
    receipt["parent"] = json!("missing");
    let orphan = append(root, &receipt);
    assert_fails_mentioning(&orphan, "parent receipt `missing`");
    assert!(!root.join("journal").exists());
    assert!(!root.join("outside.json").exists());
}
