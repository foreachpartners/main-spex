use std::path::Path;

use crate::error::Result;
use crate::support::fs::write_embedded_dir;
use crate::templates::{AGENTS_DIRNAME, BUNDLED_TEMPLATES_DIR, SCHEMAS_DIRNAME, SKILLS_DIRNAME};

use super::agent_files::{link_claude_skills, move_claude_skills, write_agents_md};

#[cfg(test)]
#[path = "scaffold_test.rs"]
mod scaffold_test;

pub fn scaffold(spex_dir: &Path, agent_root: &Path) -> Result<()> {
    copy_schemas(spex_dir)?;
    move_claude_skills(agent_root)?;
    copy_skills(agent_root)?;
    write_agents_md(agent_root)?;
    link_claude_skills(agent_root)
}

pub fn copy_schemas(spex_dir: &Path) -> Result<()> {
    let dir = BUNDLED_TEMPLATES_DIR
        .get_dir(SCHEMAS_DIRNAME)
        .expect("bundled template directory is missing");
    write_embedded_dir(dir, spex_dir)
}

fn copy_skills(agent_root: &Path) -> Result<()> {
    let Some(skills) = BUNDLED_TEMPLATES_DIR.get_dir(SKILLS_DIRNAME) else {
        return Ok(());
    };

    write_embedded_dir(skills, &agent_root.join(AGENTS_DIRNAME))
}
