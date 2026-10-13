use std::fs;
use std::path::Path;

use tempfile::tempdir;

use crate::common::{
    add_packages, assert_fails_mentioning, copy_no_spex, report_text, run_spex, with_version_header,
};

const EXPECTED_CHECK_REPORT: &str = r#"proj-spex
  no issues

proj-api
  warn  secret-handling/no-backup-files  (data.backup)
      Backup and scratch files may carry pre-redaction content
  0 errors, 1 warning

proj-frontend
  error  writing-style/no-cyrillic  (README.md:1)
      Requirements and documentation must be in English
  1 error, 0 warnings

proj: 1 error, 1 warning
"#;

const EXPECTED_COVERAGE_REPORT: &str = r#"Specs (0)
  none found

Gaps (0)
  no active or in-progress specs

planned (0), in-progress (0), active (0), gaps (0)
"#;

#[test]
fn init_then_check_no_spex_project() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");

    copy_no_spex(&project);

    let init = run_spex(&project, &["init"], "multi-repo\n\n");
    assert!(
        init.status.success(),
        "init failed (exit {:?})\nstdout:\n{}\nstderr:\n{}",
        init.status.code(),
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr),
    );
    assert!(
        project.join("proj-spex/spex.yaml").is_file(),
        "init did not write proj-spex/spex.yaml"
    );
    assert!(
        project.join("proj-spex/rules").is_dir(),
        "init did not scaffold rules"
    );

    add_packages(&project);

    let check = run_spex(&project, &["check"], "");
    assert_eq!(
        report_text(&check.stdout),
        with_version_header(EXPECTED_CHECK_REPORT),
        "check report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&check.stderr),
    );
    assert_eq!(
        check.status.code(),
        Some(1),
        "check should exit 1 when errors are present"
    );

    let coverage = run_spex(&project, &["coverage"], "");
    assert_eq!(
        report_text(&coverage.stdout),
        with_version_header(EXPECTED_COVERAGE_REPORT),
        "coverage report did not match\nstderr:\n{}",
        String::from_utf8_lossy(&coverage.stderr),
    );
    assert!(
        coverage.status.success(),
        "coverage should exit 0 when there are no gaps (exit {:?})",
        coverage.status.code(),
    );
}

fn package_with_existing_agent_files(workspace: &Path) -> std::path::PathBuf {
    let project = workspace.join("snake");
    fs::create_dir_all(project.join("src")).expect("create src");
    fs::write(project.join("src/main.rs"), "fn main() {}\n").expect("write source");
    fs::write(project.join("AGENTS.md"), "# Snake notes\n\nuse tabs\n").expect("write AGENTS.md");
    fs::create_dir_all(project.join(".claude/skills/my-skill")).expect("create user skill");
    fs::write(project.join(".claude/skills/my-skill/SKILL.md"), "mine\n").expect("write skill");
    fs::write(project.join(".claude/settings.json"), "{}\n").expect("write settings");
    project
}

fn init_with(project: &Path, answers: &str) -> std::process::Output {
    run_spex(project, &["init"], answers)
}

#[test]
fn init_single_repo_keeps_the_package_files_and_checks_clean() {
    let workspace = tempdir().expect("create temp workspace");
    let project = package_with_existing_agent_files(workspace.path());

    let init = init_with(&project, "2\n\n");
    assert!(
        init.status.success(),
        "init failed\nstderr:\n{}",
        String::from_utf8_lossy(&init.stderr)
    );

    let config = fs::read_to_string(project.join("spex/spex.yaml")).expect("read spex.yaml");
    assert!(config.contains("project_name: snake\n"), "{config}");
    assert!(config.contains("layout: single-repo\n"), "{config}");
    assert!(config.contains("agent_root: ..\n"), "{config}");
    assert!(config.contains("package_name: snake\n"), "{config}");
    assert!(config.contains("package_short_name: snake\n"), "{config}");
    assert!(config.contains("path: ..\n"), "{config}");

    for folder in ["rules", "schemas", "docs", "specs", "plans", "contracts"] {
        assert!(
            project.join("spex").join(folder).is_dir(),
            "missing spex/{folder}"
        );
    }
    assert!(
        !project.join("spex/.agents").exists(),
        "agent files belong in the root"
    );
    assert!(!project.join("spex/AGENTS.md").exists());
    assert!(!project.join("spex/snake.code-workspace").exists());

    assert!(project.join(".agents/skills/spex-spec/SKILL.md").is_file());
    assert_eq!(
        fs::read_to_string(project.join(".agents/skills/my-skill/SKILL.md")).expect("moved skill"),
        "mine\n",
        "an existing .claude skill moves into .agents"
    );
    assert!(project.join(".claude/settings.json").is_file());
    assert!(
        project.join(".claude/skills/my-skill/SKILL.md").is_file(),
        ".claude/skills should resolve to .agents/skills"
    );

    let agents = fs::read_to_string(project.join("AGENTS.md")).expect("read AGENTS.md");
    assert!(agents.starts_with("# spex"), "managed section is on top");
    assert!(
        agents.ends_with("# Snake notes\n\nuse tabs\n"),
        "existing notes are kept"
    );
    assert_eq!(
        fs::read_link(project.join("CLAUDE.md")).expect("CLAUDE.md symlink"),
        Path::new("AGENTS.md")
    );

    let check = run_spex(&project, &["check"], "");
    assert_eq!(
        report_text(&check.stdout),
        with_version_header(
            "snake\n  no issues\n\nspex\n  no issues\n\nsnake: 0 errors, 0 warnings\n"
        ),
        "stderr:\n{}",
        String::from_utf8_lossy(&check.stderr),
    );
    assert!(check.status.success());

    let coverage = run_spex(&project, &["coverage"], "");
    assert!(
        coverage.status.success(),
        "coverage should pass on a fresh project"
    );
}

#[test]
fn init_monorepo_lists_no_package_and_puts_agent_files_in_the_root() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("shop");
    fs::create_dir_all(project.join("apps/api")).expect("create package");

    let init = init_with(&project, "3\n\n");
    assert!(
        init.status.success(),
        "init failed\nstderr:\n{}",
        String::from_utf8_lossy(&init.stderr)
    );

    let config = fs::read_to_string(project.join("spex/spex.yaml")).expect("read spex.yaml");
    assert!(config.contains("layout: monorepo\n"), "{config}");
    assert!(config.contains("agent_root: ..\n"), "{config}");
    assert!(config.contains("packages: []\n"), "{config}");
    assert!(project.join(".agents/skills/spex-spec/SKILL.md").is_file());
    assert!(project.join("AGENTS.md").is_file());
    assert!(!project.join("spex/.agents").exists());
}

#[test]
fn init_in_a_non_empty_repository_named_spex_uses_it_as_the_workspace_root() {
    for (layout_answer, layout) in [("2", "single-repo"), ("3", "monorepo")] {
        let workspace = tempdir().expect("create temp workspace");
        let project = workspace.path().join("spex");
        fs::create_dir_all(project.join("src")).expect("create src");
        fs::write(project.join("src/main.rs"), "fn main() {}\n").expect("write source");

        let init = init_with(&project, &format!("{layout_answer}\n\n"));
        assert!(
            init.status.success(),
            "{layout} init failed\nstderr:\n{}",
            String::from_utf8_lossy(&init.stderr)
        );

        let config = fs::read_to_string(project.join("spex/spex.yaml")).expect("read spex.yaml");
        assert!(config.contains("project_name: spex\n"), "{config}");
        assert!(config.contains(&format!("layout: {layout}\n")), "{config}");
        assert!(project.join("AGENTS.md").is_file(), "{layout}");
        assert!(project.join(".agents/skills/spex-spec/SKILL.md").is_file());
        assert!(project.join("src/main.rs").is_file());
        assert!(!workspace.path().join("AGENTS.md").exists());
    }
}

#[test]
fn init_inside_an_initialized_spex_directory_fails_without_nesting_a_project() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("snake");
    fs::create_dir_all(&project).expect("create project");
    assert!(init_with(&project, "2\n\n").status.success());

    let spex_dir = project.join("spex");
    let again = init_with(&spex_dir, "2\n\n");

    assert_fails_mentioning(&again, "not empty");
    assert!(!spex_dir.join("spex").exists(), "nothing may be nested");
}

#[test]
fn init_multi_repo_keeps_everything_in_the_spex_repository() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("proj");
    fs::create_dir_all(&project).expect("create project");

    let init = init_with(&project, "\n\n");
    assert!(init.status.success());

    let spex = project.join("proj-spex");
    let config = fs::read_to_string(spex.join("spex.yaml")).expect("read spex.yaml");
    assert!(config.contains("layout: multi-repo\n"), "{config}");
    assert!(config.contains("agent_root: .\n"), "{config}");
    assert!(spex.join("proj.code-workspace").is_file());
    assert!(spex.join(".agents/skills/spex-spec/SKILL.md").is_file());
    assert!(spex.join("AGENTS.md").is_file());
    assert!(!project.join(".agents").exists());
}

#[test]
fn init_fails_without_writing_when_agents_md_and_claude_md_both_exist() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("snake");
    fs::create_dir_all(&project).expect("create project");
    fs::write(project.join("AGENTS.md"), "agents\n").expect("write AGENTS.md");
    fs::write(project.join("CLAUDE.md"), "claude\n").expect("write CLAUDE.md");

    let init = init_with(&project, "2\n\n");

    assert_fails_mentioning(&init, "both exist");
    assert!(!project.join("spex").exists(), "nothing may be written");
    assert!(!project.join(".agents").exists());
    assert_eq!(
        fs::read_to_string(project.join("AGENTS.md")).expect("read"),
        "agents\n"
    );
}

#[test]
fn init_fails_without_writing_when_a_claude_skill_clashes_with_a_bundled_skill() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("snake");
    fs::create_dir_all(project.join(".claude/skills/spex-spec")).expect("create skill");
    fs::write(project.join(".claude/skills/spex-spec/SKILL.md"), "mine\n").expect("write skill");

    let init = init_with(&project, "2\n\n");

    assert_fails_mentioning(&init, "spex-spec");
    assert!(!project.join("spex").exists(), "nothing may be written");
    assert!(project.join(".claude/skills/spex-spec/SKILL.md").is_file());
}

#[test]
fn init_rejects_an_unknown_layout() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("snake");
    fs::create_dir_all(&project).expect("create project");

    let init = init_with(&project, "polyrepo\n");

    assert_fails_mentioning(&init, "unknown layout `polyrepo`");
    assert!(!project.join("spex").exists());
}

#[test]
fn check_validates_the_spex_dir_in_single_repo_even_for_a_selected_package() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("snake");
    fs::create_dir_all(&project).expect("create project");
    assert!(init_with(&project, "2\n\n").status.success());
    fs::write(
        project.join("spex/specs/bad.yaml"),
        "id: other\ntitle: Other\ndescription: A spec in a file named after another id.\nacceptance_criteria:\n  - id: works\n    must: work\ncontract: none\n",
    )
    .expect("write mismatched spec");

    let check = run_spex(&project, &["check", "--package", "snake"], "");

    let report = report_text(&check.stdout);
    assert!(report.contains("\nspex\n"), "no spex heading in:\n{report}");
    assert!(
        report.contains("filename stem `bad` does not match id `other`"),
        "{report}"
    );
    assert_eq!(check.status.code(), Some(1));
}

#[test]
fn init_accepts_the_layout_name_as_well_as_its_number() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("snake");
    fs::create_dir_all(&project).expect("create project");

    let init = init_with(&project, "single-repo\n\n");

    assert!(init.status.success());
    let config = fs::read_to_string(project.join("spex/spex.yaml")).expect("read spex.yaml");
    assert!(config.contains("layout: single-repo\n"), "{config}");
}

#[test]
fn init_lists_the_layouts_with_numbers() {
    let workspace = tempdir().expect("create temp workspace");
    let project = workspace.path().join("snake");
    fs::create_dir_all(&project).expect("create project");

    let init = init_with(&project, "\n\n");

    let stdout = String::from_utf8_lossy(&init.stdout);
    assert!(
        stdout.contains("Layout:\n  1) multi-repo\n  2) single-repo\n  3) monorepo\nChoose [1]: "),
        "{stdout}"
    );
}
