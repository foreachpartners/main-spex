use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Output;

use tempfile::tempdir;

mod common;
use common::{copy_project, dummy_project_fixture, run_spex};

const CHANGELOG: &str = include_str!("../CHANGELOG.md");
const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");
const PROJECT_SECTION: &str = "\n---\n\n# Project";

const SYNCED_RULE: &str = "rules/writing-style.yaml";
const SYNCED_RULE_SCRIPT: &str = "rules/writing-style/writing-style.sh";
const SYNCED_SKILL: &str = ".agents/skills/spex-spec/SKILL.md";

struct Project {
    root: PathBuf,
    spex: PathBuf,
}

impl Project {
    fn init(workspace: &Path) -> Self {
        let root = workspace.join("proj");
        fs::create_dir_all(&root).expect("create project root");
        let init = run_spex(&root, &["init"], "\n");
        assert!(
            init.status.success(),
            "init failed (exit {:?})\nstderr:\n{}",
            init.status.code(),
            String::from_utf8_lossy(&init.stderr),
        );
        Self {
            spex: root.join("proj-spex"),
            root,
        }
    }

    fn update(&self) -> Output {
        let update = run_spex(&self.root, &["update"], "");
        assert!(
            update.status.success(),
            "update failed (exit {:?})\nstdout:\n{}\nstderr:\n{}",
            update.status.code(),
            String::from_utf8_lossy(&update.stdout),
            String::from_utf8_lossy(&update.stderr),
        );
        update
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.spex.join(relative)
    }

    fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.path(relative)).unwrap_or_else(|_| panic!("read {relative}"))
    }

    fn write(&self, relative: &str, contents: &str) {
        let path = self.path(relative);
        fs::create_dir_all(path.parent().expect("parent directory")).expect("create parents");
        fs::write(path, contents).unwrap_or_else(|_| panic!("write {relative}"));
    }

    fn remove(&self, relative: &str) {
        fs::remove_file(self.path(relative)).unwrap_or_else(|_| panic!("remove {relative}"));
    }

    fn with_tampered_managed_section(&self, relative: &str) -> (String, String) {
        let pristine = self.read(relative);
        let tampered = pristine.replacen(
            PROJECT_SECTION,
            &format!("\nTampered managed line.{PROJECT_SECTION}"),
            1,
        );
        assert_ne!(tampered, pristine, "{relative} has no project section");
        (pristine, tampered)
    }
}

fn read_tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut tree = BTreeMap::new();
    collect_tree(root, root, &mut tree);
    tree
}

fn collect_tree(root: &Path, dir: &Path, tree: &mut BTreeMap<PathBuf, Vec<u8>>) {
    for entry in fs::read_dir(dir).expect("read dir") {
        let path = entry.expect("read dir entry").path();
        let relative = path
            .strip_prefix(root)
            .expect("relative path")
            .to_path_buf();
        let kind = fs::symlink_metadata(&path)
            .expect("read metadata")
            .file_type();
        if kind.is_symlink() {
            let target = fs::read_link(&path).expect("read symlink target");
            tree.insert(relative, target.into_os_string().into_encoded_bytes());
        } else if kind.is_dir() {
            collect_tree(root, &path, tree);
        } else {
            tree.insert(relative, fs::read(&path).expect("read file"));
        }
    }
}

fn changed_paths(
    before: &BTreeMap<PathBuf, Vec<u8>>,
    after: &BTreeMap<PathBuf, Vec<u8>>,
) -> Vec<PathBuf> {
    before
        .keys()
        .chain(after.keys())
        .filter(|path| before.get(*path) != after.get(*path))
        .cloned()
        .collect()
}

fn changelog_headers_newer_than(version: &str) -> Vec<String> {
    CHANGELOG
        .lines()
        .filter(|line| line.starts_with("## ["))
        .take_while(|line| !line.starts_with(&format!("## [{version}]")))
        .map(str::to_string)
        .collect()
}

fn printed_headers(stdout: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(stdout)
        .lines()
        .filter(|line| line.starts_with("## ["))
        .map(str::to_string)
        .collect()
}

#[test]
fn update_upgrades_project_written_by_older_cli() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");
    copy_project(&dummy_project_fixture(), &project);
    let spex = project.join("dummy-project-spex");

    let stale_check = run_spex(&project, &["check"], "");
    let stale_warning = String::from_utf8_lossy(&stale_check.stderr).into_owned();
    assert!(
        stale_warning.contains("was written by spex 0.8.8")
            && stale_warning.contains("`spex update`"),
        "check should warn about the outdated config\nstderr:\n{stale_warning}",
    );

    let config_before = fs::read_to_string(spex.join("spex.yaml")).expect("read spex.yaml");
    let local_rule_before =
        fs::read(spex.join("rules/no-todo-comments.yaml")).expect("read local rule");
    fs::write(spex.join("schemas/rule.schema.json"), "{}").expect("tamper schema");

    let update = run_spex(&project, &["update"], "");
    assert!(
        update.status.success(),
        "update failed (exit {:?})\nstderr:\n{}",
        update.status.code(),
        String::from_utf8_lossy(&update.stderr),
    );
    let stdout = String::from_utf8_lossy(&update.stdout);
    assert!(
        stdout.starts_with(&format!(
            "spex skills, schemas and rules updated, cli_version set to {CLI_VERSION}\n"
        )),
        "unexpected update summary\n{stdout}",
    );
    assert!(stdout.contains("Changes since 0.8.8:"), "{stdout}");
    assert_eq!(
        printed_headers(&update.stdout),
        changelog_headers_newer_than("0.8.8"),
        "update should print every changelog entry newer than 0.8.8, newest first",
    );

    let config_after = fs::read_to_string(spex.join("spex.yaml")).expect("reread spex.yaml");
    assert_eq!(
        config_after,
        config_before.replace("cli_version: 0.8.8", &format!("cli_version: {CLI_VERSION}")),
        "update should change only cli_version in spex.yaml",
    );
    assert_eq!(
        fs::read(spex.join("rules/no-todo-comments.yaml")).expect("reread local rule"),
        local_rule_before,
        "update must leave a local rule untouched",
    );
    assert_eq!(
        fs::read(spex.join("schemas/rule.schema.json")).expect("reread schema"),
        fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("templates/schemas/rule.schema.json"))
            .expect("read bundled schema"),
        "update should restore the bundled schema",
    );

    for deployed in [
        SYNCED_RULE,
        SYNCED_RULE_SCRIPT,
        SYNCED_SKILL,
        "docs/glossary.md",
    ] {
        assert!(
            spex.join(deployed).is_file(),
            "update did not deploy {deployed}"
        );
    }
    let script_mode = fs::metadata(spex.join(SYNCED_RULE_SCRIPT))
        .expect("read script metadata")
        .permissions()
        .mode();
    assert_ne!(
        script_mode & 0o111,
        0,
        "deployed script should be executable"
    );
    assert_eq!(
        fs::read_link(spex.join(".claude")).expect("read .claude symlink"),
        Path::new(".agents"),
        ".claude should link to .agents",
    );
    assert!(
        !spex.join("AGENTS.md").exists(),
        "update must not seed AGENTS.md"
    );

    let check = run_spex(&project, &["check"], "");
    assert!(
        check.status.success(),
        "check should pass after update\nstdout:\n{}",
        String::from_utf8_lossy(&check.stdout),
    );
    assert!(
        check.stderr.is_empty(),
        "check should no longer warn after update\nstderr:\n{}",
        String::from_utf8_lossy(&check.stderr),
    );
}

#[test]
fn update_changes_nothing_when_project_is_current() {
    let workspace = tempdir().expect("create temp workspace");
    let project = Project::init(workspace.path());
    let before = read_tree(&project.spex);

    let update = project.update();

    assert!(
        !String::from_utf8_lossy(&update.stdout).contains("Changes since"),
        "a current project has no changelog entries to print",
    );
    assert_eq!(
        changed_paths(&before, &read_tree(&project.spex)),
        Vec::<PathBuf>::new(),
        "update rewrote an already current project",
    );
}

#[test]
fn update_prints_every_changelog_entry_when_previous_version_is_unknown() {
    let workspace = tempdir().expect("create temp workspace");
    let project = Project::init(workspace.path());
    let config = project.read("spex.yaml");
    let patched = config.replace(&format!("cli_version: {CLI_VERSION}"), "cli_version: 0.0.1");
    assert_ne!(patched, config, "cli_version not found in spex.yaml");
    project.write("spex.yaml", &patched);

    let update = project.update();

    assert!(String::from_utf8_lossy(&update.stdout).contains("Changes since 0.0.1:"));
    let printed = printed_headers(&update.stdout);
    assert_eq!(printed, changelog_headers_newer_than("0.0.1"));
    assert!(printed.len() > 1, "expected the whole changelog");
}

#[test]
fn update_restores_synced_rule_and_script_and_redeploys_deleted_rule() {
    let workspace = tempdir().expect("create temp workspace");
    let project = Project::init(workspace.path());
    let deleted_rule = "rules/dependencies-rules.yaml";
    let pristine_rule = project.read(SYNCED_RULE);
    let pristine_script = project.read(SYNCED_RULE_SCRIPT);
    let pristine_deleted = project.read(deleted_rule);

    let tampered_rule = pristine_rule.replace("title: Writing style", "title: Tampered");
    assert_ne!(tampered_rule, pristine_rule, "rule title not found");
    project.write(SYNCED_RULE, &tampered_rule);
    project.write(SYNCED_RULE_SCRIPT, &format!("{pristine_script}\nexit 0\n"));
    project.remove(deleted_rule);

    project.update();

    assert_eq!(project.read(SYNCED_RULE), pristine_rule);
    assert_eq!(project.read(SYNCED_RULE_SCRIPT), pristine_script);
    assert_eq!(project.read(deleted_rule), pristine_deleted);
}

#[test]
fn update_keeps_rule_and_script_that_opted_out_of_upstream_sync() {
    let workspace = tempdir().expect("create temp workspace");
    let project = Project::init(workspace.path());
    let pristine_rule = project.read(SYNCED_RULE);
    let pristine_script = project.read(SYNCED_RULE_SCRIPT);

    let customized_rule = pristine_rule
        .replace("sync_with_upstream: true\n", "")
        .replace("title: Writing style", "title: Local writing style");
    assert_ne!(customized_rule, pristine_rule, "sync flag not found");
    let customized_script = format!("{pristine_script}\nexit 0\n");
    project.write(SYNCED_RULE, &customized_rule);
    project.write(SYNCED_RULE_SCRIPT, &customized_script);

    project.update();

    assert_eq!(project.read(SYNCED_RULE), customized_rule);
    assert_eq!(project.read(SYNCED_RULE_SCRIPT), customized_script);
}

#[test]
fn update_does_not_redeploy_rule_listed_in_rules_unused() {
    let workspace = tempdir().expect("create temp workspace");
    let project = Project::init(workspace.path());
    let config = project.read("spex.yaml");
    project.write(
        "spex.yaml",
        &format!("{}\nrules_unused:\n  - testing-rules\n", config.trim_end()),
    );
    project.remove("rules/testing-rules.yaml");

    project.update();

    assert!(
        !project.path("rules/testing-rules.yaml").exists(),
        "a rule listed in rules_unused was redeployed",
    );
    assert!(
        project.read("spex.yaml").contains("- testing-rules"),
        "update dropped rules_unused from spex.yaml",
    );
}

#[test]
fn update_leaves_files_added_to_the_spex_repository_untouched() {
    let workspace = tempdir().expect("create temp workspace");
    let project = Project::init(workspace.path());
    let added = [
        ("rules/local-rule.yaml", "id: local-rule\n"),
        ("rules/local-rule/run.sh", "#!/bin/sh\nexit 0\n"),
        ("schemas/local.schema.json", "{}\n"),
        ("docs/local.md", "# Local\n"),
        ("plans/local-plan.md", "# Plan\n"),
        (
            ".agents/skills/local-skill/SKILL.md",
            "---\nname: local-skill\n---\n",
        ),
    ];
    for (relative, contents) in added {
        project.write(relative, contents);
    }

    project.update();

    for (relative, contents) in added {
        assert_eq!(
            project.read(relative),
            contents,
            "update changed {relative}"
        );
    }
}

#[test]
fn update_rewrites_skill_above_project_section_and_keeps_the_rest() {
    let workspace = tempdir().expect("create temp workspace");
    let project = Project::init(workspace.path());
    let (pristine, tampered) = project.with_tampered_managed_section(SYNCED_SKILL);
    project.write(SYNCED_SKILL, &format!("{tampered}\nLocal note.\n"));
    project.write(
        ".agents/skills/spex-spec/local.md",
        "Added by the project.\n",
    );

    project.update();

    assert_eq!(
        project.read(SYNCED_SKILL),
        format!("{pristine}\nLocal note.\n")
    );
    assert_eq!(
        project.read(".agents/skills/spex-spec/local.md"),
        "Added by the project.\n",
    );
}

#[test]
fn update_keeps_skill_that_opted_out_of_upstream_sync() {
    let workspace = tempdir().expect("create temp workspace");
    let project = Project::init(workspace.path());
    let (_, tampered) = project.with_tampered_managed_section(SYNCED_SKILL);
    let opted_out = tampered.replacen(
        "\n---\n",
        "\nmetadata:\n  sync-with-upstream: false\n---\n",
        1,
    );
    project.write(SYNCED_SKILL, &opted_out);

    project.update();

    assert_eq!(project.read(SYNCED_SKILL), opted_out);
}

#[test]
fn update_rewrites_agents_md_above_project_section_and_keeps_the_rest() {
    let workspace = tempdir().expect("create temp workspace");
    let project = Project::init(workspace.path());
    let (pristine, tampered) = project.with_tampered_managed_section("AGENTS.md");
    project.write("AGENTS.md", &format!("{tampered}\nProject note.\n"));

    project.update();

    assert_eq!(
        project.read("AGENTS.md"),
        format!("{pristine}\nProject note.\n")
    );
}

#[test]
fn update_fully_overwrites_agents_md_that_predates_the_project_section() {
    let workspace = tempdir().expect("create temp workspace");
    let project = Project::init(workspace.path());
    let pristine = project.read("AGENTS.md");
    project.write(
        "AGENTS.md",
        "Legacy agents file without a project section.\n",
    );

    project.update();

    assert_eq!(project.read("AGENTS.md"), pristine);
}

#[test]
fn update_fails_outside_a_spex_workspace() {
    let workspace = tempdir().expect("create temp workspace");

    let update = run_spex(workspace.path(), &["update"], "");

    assert!(
        !update.status.success(),
        "update should fail when no spex.yaml is found",
    );
    assert!(
        !update.stderr.is_empty(),
        "update should explain why it failed",
    );
}
