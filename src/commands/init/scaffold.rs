use std::path::Path;

use crate::error::Result;
use crate::rule::{RULES_DIR, SCRIPTS_DIR};
use crate::support::fs::{
    make_files_executable, symlink_if_absent, write_embedded_dir, write_file,
};
use crate::templates::{
    AGENTS_DIRNAME, AGENTS_FILENAME, CLAUDE_DIRNAME, CLAUDE_FILENAME, DOCS_DIRNAME,
    SCHEMAS_DIRNAME, SKILLS_DIRNAME, TEMPLATES_DIR,
};

#[cfg(test)]
#[path = "scaffold_test.rs"]
mod scaffold_test;

pub fn scaffold(spex_dir: &Path) -> Result<()> {
    copy_docs_and_schemas(spex_dir)?;
    copy_rules(spex_dir)?;
    copy_skills(spex_dir)?;
    copy_agents_md(spex_dir)?;
    link_claude_dir(spex_dir)?;
    make_files_executable(&spex_dir.join(SCRIPTS_DIR))
}

pub fn copy_docs_and_schemas(spex_dir: &Path) -> Result<()> {
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

pub fn link_claude_dir(spex_dir: &Path) -> Result<()> {
    symlink_if_absent(&spex_dir.join(CLAUDE_DIRNAME), AGENTS_DIRNAME)
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

    write_embedded_dir(skills, &spex_dir.join(AGENTS_DIRNAME))
}

fn copy_agents_md(spex_dir: &Path) -> Result<()> {
    let agents = TEMPLATES_DIR
        .get_file(AGENTS_FILENAME)
        .expect("bundled AGENTS.md is missing");
    write_file(&spex_dir.join(AGENTS_FILENAME), agents.contents())?;
    symlink_if_absent(&spex_dir.join(CLAUDE_FILENAME), AGENTS_FILENAME)
}
