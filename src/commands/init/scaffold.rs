use std::path::Path;

use include_dir::File;

use crate::error::Result;
use crate::rule::RULES_DIR;
use crate::support::fs::{
    create_dir_all, make_files_executable, symlink_if_absent, write_embedded_dir,
    write_embedded_dir_at, write_file,
};
use crate::templates::{
    AGENTS_DIRNAME, AGENTS_FILENAME, BUNDLED_TEMPLATES_DIR, CLAUDE_DIRNAME, CLAUDE_FILENAME,
    DOCS_DIRNAME, SCHEMAS_DIRNAME, SKILLS_DIRNAME,
};

#[cfg(test)]
#[path = "scaffold_test.rs"]
mod scaffold_test;

pub fn scaffold(spex_dir: &Path) -> Result<()> {
    copy_docs_and_schemas(spex_dir)?;
    copy_rules(spex_dir)?;
    copy_skills(spex_dir)?;
    copy_agents_md(spex_dir)?;
    link_claude_dir(spex_dir)
}

pub fn copy_docs_and_schemas(spex_dir: &Path) -> Result<()> {
    for name in [DOCS_DIRNAME, SCHEMAS_DIRNAME] {
        let dir = BUNDLED_TEMPLATES_DIR
            .get_dir(name)
            .expect("bundled template directory is missing");
        write_embedded_dir(dir, spex_dir)?;
    }
    Ok(())
}

pub fn write_rule(rule: &File, dest: &Path) -> Result<()> {
    write_file(dest, rule.contents())?;

    let Some(scripts) = BUNDLED_TEMPLATES_DIR.get_dir(rule.path().with_extension("")) else {
        return Ok(());
    };
    let scripts_dest = dest.with_extension("");
    write_embedded_dir_at(scripts, &scripts_dest)?;
    make_files_executable(&scripts_dest)
}

pub fn link_claude_dir(spex_dir: &Path) -> Result<()> {
    symlink_if_absent(&spex_dir.join(CLAUDE_DIRNAME), AGENTS_DIRNAME)
}

fn copy_rules(spex_dir: &Path) -> Result<()> {
    let Some(rules) = BUNDLED_TEMPLATES_DIR.get_dir(RULES_DIR) else {
        return Ok(());
    };

    let rules_dest = spex_dir.join(RULES_DIR);
    create_dir_all(&rules_dest)?;
    for file in rules.files() {
        let name = file.path().file_name().unwrap_or_default();
        write_rule(file, &rules_dest.join(name))?;
    }
    Ok(())
}

fn copy_skills(spex_dir: &Path) -> Result<()> {
    let Some(skills) = BUNDLED_TEMPLATES_DIR.get_dir(SKILLS_DIRNAME) else {
        return Ok(());
    };

    write_embedded_dir(skills, &spex_dir.join(AGENTS_DIRNAME))
}

fn copy_agents_md(spex_dir: &Path) -> Result<()> {
    let agents = BUNDLED_TEMPLATES_DIR
        .get_file(AGENTS_FILENAME)
        .expect("bundled AGENTS.md is missing");
    write_file(&spex_dir.join(AGENTS_FILENAME), agents.contents())?;
    symlink_if_absent(&spex_dir.join(CLAUDE_FILENAME), AGENTS_FILENAME)
}
