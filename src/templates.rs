use std::path::{Path, PathBuf};

use include_dir::{Dir, include_dir};

use crate::error::Result;
use crate::rule::{RULES_DIR, SCRIPTS_DIR};
use crate::support::fs::{make_files_executable, symlink_if_absent, write_embedded_dir};
use crate::support::util;

#[cfg(test)]
#[path = "templates_test.rs"]
mod templates_test;

pub static TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

pub const SCHEMAS_DIR: &str = "schemas";
pub const DOCS_DIR: &str = "docs";
pub const SKILLS_DIR: &str = "skills";
pub const AGENTS_DIR_NAME: &str = ".agents";
pub const CLAUDE_DIR_NAME: &str = ".claude";
pub const AGENTS_FILE: &str = "AGENTS.md";
pub const CLAUDE_FILE: &str = "CLAUDE.md";

// Copied as-is, overriding existing files, unlike rules and skills whose
// overwrite is conditional.
const PLAIN_DIRS: [&str; 2] = [DOCS_DIR, SCHEMAS_DIR];

pub fn scaffold(spex_dir: &Path) -> Result<()> {
    copy_plain_dirs(spex_dir)?;
    copy_rules(spex_dir)?;
    copy_skills(spex_dir)?;
    copy_agents_md(spex_dir)?;
    link_claude_dir(spex_dir)?;
    make_files_executable(&spex_dir.join(SCRIPTS_DIR))
}

pub fn agents_dir(spex_dir: &Path) -> PathBuf {
    spex_dir.join(AGENTS_DIR_NAME)
}

pub fn installed_skills_dir(spex_dir: &Path) -> PathBuf {
    agents_dir(spex_dir).join(SKILLS_DIR)
}

pub fn copy_plain_dirs(spex_dir: &Path) -> Result<()> {
    for name in PLAIN_DIRS {
        let dir = TEMPLATES
            .get_dir(name)
            .expect("bundled template directory is missing");
        write_embedded_dir(dir, spex_dir)?;
    }
    Ok(())
}

pub fn copy_rule_scripts(rule_id: &str, spex_dir: &Path) -> Result<()> {
    let Some(scripts) = TEMPLATES.get_dir(format!("{SCRIPTS_DIR}/{rule_id}")) else {
        return Ok(());
    };
    write_embedded_dir(scripts, spex_dir)
}

fn copy_rules(spex_dir: &Path) -> Result<()> {
    let Some(rules) = TEMPLATES.get_dir(RULES_DIR) else {
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
    let Some(skills) = TEMPLATES.get_dir(SKILLS_DIR) else {
        return Ok(());
    };

    write_embedded_dir(skills, &agents_dir(spex_dir))
}

fn copy_agents_md(spex_dir: &Path) -> Result<()> {
    let agents = TEMPLATES
        .get_file(AGENTS_FILE)
        .expect("bundled AGENTS.md is missing");
    util::write_file(&spex_dir.join(AGENTS_FILE), agents.contents())?;
    symlink_if_absent(&spex_dir.join(CLAUDE_FILE), AGENTS_FILE)
}

pub fn link_claude_dir(spex_dir: &Path) -> Result<()> {
    symlink_if_absent(&spex_dir.join(CLAUDE_DIR_NAME), AGENTS_DIR_NAME)
}
