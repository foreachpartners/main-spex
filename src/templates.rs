use std::path::{Path, PathBuf};

use include_dir::{Dir, include_dir};

use crate::error::Result;
use crate::rule::{RULES_DIR, SCRIPTS_DIR};
use crate::support::fs::{
    make_files_executable, symlink_if_absent, write_embedded_dir, write_file,
};

#[cfg(test)]
#[path = "templates_test.rs"]
mod templates_test;

pub static TEMPLATES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

pub const SCHEMAS_DIRNAME: &str = "schemas";
pub const DOCS_DIRNAME: &str = "docs";
pub const SKILLS_DIRNAME: &str = "skills";
pub const AGENTS_DIRNAME: &str = ".agents";
pub const CLAUDE_DIRNAME: &str = ".claude";
pub const AGENTS_FILENAME: &str = "AGENTS.md";
pub const CLAUDE_FILENAME: &str = "CLAUDE.md";

pub fn scaffold(spex_dir: &Path) -> Result<()> {
    copy_plain_dirs(spex_dir)?;
    copy_rules(spex_dir)?;
    copy_skills(spex_dir)?;
    copy_agents_md(spex_dir)?;
    link_claude_dir(spex_dir)?;
    make_files_executable(&spex_dir.join(SCRIPTS_DIR))
}

pub fn agents_dir(spex_dir: &Path) -> PathBuf {
    spex_dir.join(AGENTS_DIRNAME)
}

pub fn installed_skills_dir(spex_dir: &Path) -> PathBuf {
    agents_dir(spex_dir).join(SKILLS_DIRNAME)
}

pub fn copy_plain_dirs(spex_dir: &Path) -> Result<()> {
    for name in [DOCS_DIRNAME, SCHEMAS_DIRNAME] {
        let dir = TEMPLATES_DIR
            .get_dir(name)
            .expect("bundled template directory is missing");
        write_embedded_dir(dir, spex_dir)?;
    }
    Ok(())
}

pub fn copy_rule_scripts(rule_id: &str, spex_dir: &Path) -> Result<()> {
    let Some(scripts) = TEMPLATES_DIR.get_dir(format!("{SCRIPTS_DIR}/{rule_id}")) else {
        return Ok(());
    };
    write_embedded_dir(scripts, spex_dir)
}

fn copy_rules(spex_dir: &Path) -> Result<()> {
    let Some(rules) = TEMPLATES_DIR.get_dir(RULES_DIR) else {
        return Ok(());
    };

    write_embedded_dir(rules, spex_dir)?;
    for file in rules.files() {
        let rule_id = file
            .path()
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        copy_rule_scripts(&rule_id, spex_dir)?;
    }
    Ok(())
}

fn copy_skills(spex_dir: &Path) -> Result<()> {
    let Some(skills) = TEMPLATES_DIR.get_dir(SKILLS_DIRNAME) else {
        return Ok(());
    };

    write_embedded_dir(skills, &agents_dir(spex_dir))
}

fn copy_agents_md(spex_dir: &Path) -> Result<()> {
    let agents = TEMPLATES_DIR
        .get_file(AGENTS_FILENAME)
        .expect("bundled AGENTS.md is missing");
    write_file(&spex_dir.join(AGENTS_FILENAME), agents.contents())?;
    symlink_if_absent(&spex_dir.join(CLAUDE_FILENAME), AGENTS_FILENAME)
}

pub fn link_claude_dir(spex_dir: &Path) -> Result<()> {
    symlink_if_absent(&spex_dir.join(CLAUDE_DIRNAME), AGENTS_DIRNAME)
}
