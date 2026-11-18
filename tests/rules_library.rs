use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

#[derive(Deserialize)]
struct Entry {
    id: String,
    #[serde(default)]
    checks: Vec<Check>,
}

#[derive(Deserialize)]
struct Check {
    #[serde(default)]
    script: Option<String>,
}

const LIBRARY_DIR: &str = "templates/rules";
const RULE_SCHEMA_FILE: &str = "templates/schemas/rule.schema.json";
const SCHEMAS_DIR: &str = "templates/schemas";
const CONFORMANCE_RULE_ID: &str = "spex-internal-conformance";

fn repository() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn entry_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_entry_files(&repository().join(LIBRARY_DIR), &mut files);
    files.sort();
    assert!(!files.is_empty(), "rules library has no entries");
    files
}

// A subfolder groups entries unless an entry of its name sits beside it, making it that entry's script folder.
fn collect_entry_files(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("rules library folder") {
        let path = entry.expect("rules library entry").path();
        if path.is_dir() {
            if !path.with_extension("yaml").is_file() {
                collect_entry_files(&path, files);
            }
        } else if path
            .extension()
            .is_some_and(|extension| extension == "yaml")
        {
            files.push(path);
        }
    }
}

fn read_entry(file: &Path) -> Entry {
    let text = fs::read_to_string(file).expect("read entry");
    serde_yaml_ng::from_str(&text).unwrap_or_else(|error| panic!("{}: {error}", file.display()))
}

#[test]
fn rules_library_entries_validate_against_the_rule_schema() {
    let output = Command::new(env!("CARGO_BIN_EXE_spex"))
        .args(["check-schema", "--schema"])
        .arg(repository().join(RULE_SCHEMA_FILE))
        .arg("--files")
        .args(entry_files())
        .output()
        .expect("run spex");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn rules_library_entries_have_unique_ids() {
    let mut files_by_id: HashMap<String, PathBuf> = HashMap::new();
    for file in entry_files() {
        let id = read_entry(&file).id;

        if let Some(first) = files_by_id.insert(id.clone(), file.clone()) {
            panic!(
                "id `{id}` is in both {} and {}",
                first.display(),
                file.display()
            );
        }
    }
}

#[test]
fn rules_library_entries_are_named_by_id() {
    for file in entry_files() {
        let entry = read_entry(&file);

        let file_stem = file.file_stem().and_then(|stem| stem.to_str());
        assert_eq!(file_stem, Some(entry.id.as_str()), "{}", file.display());
    }
}

#[test]
fn rules_library_entries_set_sync_with_upstream_true_right_after_their_title() {
    for file in entry_files() {
        let text = fs::read_to_string(&file).expect("read entry");
        let lines: Vec<&str> = text.lines().take(3).collect();

        assert!(
            lines[1].starts_with("title: ") && lines[2] == "sync_with_upstream: true",
            "{} must set sync_with_upstream: true right after its title",
            file.display()
        );
    }
}

#[test]
fn rules_library_entries_have_no_leftover_template_only_fields() {
    for file in entry_files() {
        let text = fs::read_to_string(&file).expect("read entry");

        assert!(
            !text.contains("first_created_in_spex_version"),
            "{} still carries the removed `first_created_in_spex_version` field",
            file.display()
        );
    }
}

#[test]
fn rules_library_entries_ship_every_script_their_checks_run() {
    for file in entry_files() {
        let entry = read_entry(&file);
        let script_folder = file.with_extension("");

        for script in entry
            .checks
            .iter()
            .filter_map(|check| check.script.as_ref())
        {
            assert!(
                script_folder.join(script).is_file(),
                "{} runs script `{script}` missing from {}",
                file.display(),
                script_folder.display()
            );
        }
    }
}

#[test]
fn rules_library_entries_pass_the_conformance_checks_of_spex_check() {
    let ids: Vec<String> = entry_files()
        .iter()
        .map(|file| read_entry(file).id)
        .collect();
    let project = tempfile::tempdir().expect("temp dir");
    let spex_dir = project.path().join("proj-spex");
    let schemas = spex_dir.join("schemas");
    fs::create_dir_all(&schemas).expect("create schemas dir");
    for schema in fs::read_dir(repository().join(SCHEMAS_DIR)).expect("schemas folder") {
        let path = schema.expect("schema").path();
        fs::copy(&path, schemas.join(path.file_name().expect("schema name"))).expect("copy schema");
    }
    fs::create_dir_all(spex_dir.join("docs")).expect("create docs dir");
    fs::write(spex_dir.join("docs/prd.md"), "# Product requirements\n").expect("write prd");
    let listed_rules: String = ids.iter().map(|id| format!("      - {id}\n")).collect();
    let version = env!("CARGO_PKG_VERSION");
    fs::write(
        spex_dir.join("spex.yaml"),
        format!(
            "project_name: proj\nproject_short_name: prj\ncli_version: {version}\nlayout: multi-repo\nagent_root: .\ndescription: Takes every rules library entry.\nprd: docs/prd.md\npackages:\n  - package_name: proj-spex\n    package_short_name: spex\n    path: .\n    rules:\n{listed_rules}"
        ),
    )
    .expect("write spex.yaml");

    let taken = Command::new(env!("CARGO_BIN_EXE_spex"))
        .current_dir(&spex_dir)
        .args(["lib", "rules", "--copy", "rules", "--id"])
        .args(&ids)
        .output()
        .expect("run spex lib");
    assert!(
        taken.status.success(),
        "{}",
        String::from_utf8_lossy(&taken.stderr)
    );

    let check = Command::new(env!("CARGO_BIN_EXE_spex"))
        .current_dir(&spex_dir)
        .args(["check", "--files", "spex.yaml"])
        .output()
        .expect("run spex check");
    assert!(
        matches!(check.status.code(), Some(0 | 1)),
        "spex check did not run: {}",
        String::from_utf8_lossy(&check.stderr)
    );
    let report = String::from_utf8_lossy(&check.stdout);
    let conformance_findings: Vec<&str> = report
        .lines()
        .filter(|line| line.contains(CONFORMANCE_RULE_ID))
        .collect();

    assert!(
        conformance_findings.is_empty(),
        "rules library entries fail spex check conformance:\n{report}"
    );
}

#[cfg(unix)]
mod commit_trace {
    use super::*;
    use std::process::Output;

    fn git(dir: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .current_dir(dir)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .args([
                "-c",
                "user.name=Test developer",
                "-c",
                "user.email=developer@example.com",
                "-c",
                "core.hooksPath=/dev/null",
            ])
            .args(args)
            .output()
            .expect("run git");
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .expect("git output is UTF-8")
            .trim()
            .to_string()
    }

    fn project() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let dir = tempfile::tempdir().expect("temp project");
        let spec_dir = dir.path().join("demo-spex");
        let package = dir.path().join("demo-api");
        let specs = spec_dir.join("specs");
        fs::create_dir_all(&specs).expect("create specs");
        fs::create_dir(&package).expect("create API package");
        fs::write(
            spec_dir.join("spex.yaml"),
            format!(
                "project_name: demo\nproject_short_name: dm\ncli_version: {}\nlayout: multi-repo\nagent_root: .\nprd: docs/prd.md\npackages:\n  - package_name: demo-api\n    package_short_name: api\n    path: ../demo-api\n",
                env!("CARGO_PKG_VERSION")
            ),
        )
        .expect("write config");
        for id in ["checkout", "inventory"] {
            fs::write(
                specs.join(format!("{id}.yaml")),
                format!(
                    "id: {id}\ntitle: Validate a request\ndescription: Validate input.\nacceptance_criteria:\n  - id: validates-input\n    must: Validate input.\n  - id: preserves-label\n    must: Preserve the label.\npackages: [api]\nstatus: active\ncontract: none\n"
                ),
            )
            .expect("write spec");
        }
        git(&package, &["init", "-q"]);
        git(
            &package,
            &["commit", "--allow-empty", "-m", "initial commit"],
        );
        (dir, spec_dir, package)
    }

    fn add_trace(package: &Path, annotation: &str, commit_subject: &str) {
        fs::write(
            package.join("main.rs"),
            format!("// @spec impl {annotation}\nfn validate() {{}}\n"),
        )
        .expect("write annotation");
        git(package, &["add", "main.rs"]);
        git(package, &["commit", "-m", commit_subject]);
    }

    fn check(spec_dir: &Path, package: &Path, base: Option<&str>) -> Output {
        let bin = Path::new(env!("CARGO_BIN_EXE_spex"));
        let mut paths = vec![bin.parent().expect("binary directory").to_path_buf()];
        if let Some(path) = std::env::var_os("PATH") {
            paths.extend(std::env::split_paths(&path));
        }
        let mut command = Command::new("sh");
        command
            .arg(
                repository()
                    .join(LIBRARY_DIR)
                    .join("commit-message-format/commit-spec-id-trace.sh"),
            )
            .current_dir(package)
            .env("PATH", std::env::join_paths(paths).expect("PATH"))
            .env("SPEX_DIR", spec_dir)
            .env_remove("SPEX_COMMIT_BASE");
        if let Some(base) = base {
            command.env("SPEX_COMMIT_BASE", base);
        }
        command.output().expect("run commit trace check")
    }

    fn stdout(output: &Output) -> String {
        String::from_utf8(output.stdout.clone()).expect("trace check prints UTF-8")
    }

    #[test]
    fn bundled_commit_trace_checks_relevant_fix_tags_against_full_spec_and_criterion_ids() {
        let (_dir, spec_dir, package) = project();
        add_trace(
            &package,
            "checkout#validates-input",
            "fix(api): guard checkout @spec inventory#validates-input, checkout#preserves-label, checkout#unknown",
        );
        let output = check(&spec_dir, &package, None);
        assert_eq!(output.status.code(), Some(1));
        let findings = stdout(&output);
        assert!(
            findings.contains("unknown @spec ID checkout#unknown"),
            "{findings}"
        );
        assert!(
            findings.contains("missing an @spec tag for added trace checkout#validates-input"),
            "{findings}"
        );
    }

    #[test]
    fn bundled_commit_trace_respects_later_removal_of_a_historical_criterion_marker() {
        let (_dir, spec_dir, package) = project();
        add_trace(
            &package,
            "checkout#old-criterion",
            "feat(api): validate checkout @spec checkout#old-criterion",
        );
        add_trace(
            &package,
            "checkout#validates-input",
            "fix(api): rename criterion @spec checkout",
        );

        let output = check(&spec_dir, &package, None);
        assert!(
            output.status.success(),
            "{}\n{}",
            stdout(&output),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    #[test]
    fn bundled_commit_trace_accepts_an_exact_ancestor_range_and_rejects_an_invalid_base() {
        let (_dir, spec_dir, package) = project();
        let base = git(&package, &["rev-parse", "HEAD"]);
        add_trace(
            &package,
            "checkout#validates-input",
            "fix(api): guard checkout",
        );
        for subject in ["chore: one", "chore: two", "chore: three"] {
            git(&package, &["commit", "--allow-empty", "-m", subject]);
        }
        assert!(check(&spec_dir, &package, None).status.success());

        let ranged = check(&spec_dir, &package, Some(&base));
        assert_eq!(ranged.status.code(), Some(1));
        assert!(stdout(&ranged).contains("added trace checkout#validates-input"));

        let invalid = check(&spec_dir, &package, Some("main.rs"));
        assert_eq!(invalid.status.code(), Some(1));
        assert!(stdout(&invalid).contains("SPEX_COMMIT_BASE must name a reachable commit"));
    }
}
