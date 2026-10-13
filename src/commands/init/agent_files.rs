use std::fs;
use std::path::Path;

use eros::{Context, bail};

use crate::error::{Result, could_not_read, could_not_read_dir};
use crate::support::fs::{create_dir_all, symlink_if_absent, write_file};
use crate::support::managed_section::keep_as_project_section;
use crate::templates::{
    AGENTS_DIRNAME, AGENTS_FILENAME, BUNDLED_TEMPLATES_DIR, CLAUDE_DIRNAME, CLAUDE_FILENAME,
    SKILLS_DIRNAME,
};

#[cfg(test)]
#[path = "agent_files_test.rs"]
mod agent_files_test;

pub fn verify_agent_files(agent_root: &Path) -> Result<()> {
    if is_regular_file(&agent_root.join(AGENTS_FILENAME))
        && is_regular_file(&agent_root.join(CLAUDE_FILENAME))
    {
        bail!(
            "`{AGENTS_FILENAME}` and `{CLAUDE_FILENAME}` both exist in `{}` — merge them into `{AGENTS_FILENAME}` and remove `{CLAUDE_FILENAME}`, then run `spex init` again",
            agent_root.display()
        );
    }

    let bundled = bundled_skill_names();
    let agents_skills = agent_root.join(AGENTS_DIRNAME).join(SKILLS_DIRNAME);
    for name in claude_skill_names(agent_root)? {
        if bundled.contains(&name) || agents_skills.join(&name).exists() {
            bail!(
                "skill `{}` in `{CLAUDE_DIRNAME}/{SKILLS_DIRNAME}` would overwrite the skill of the same name in `{AGENTS_DIRNAME}/{SKILLS_DIRNAME}` or a bundled one — rename or remove it, then run `spex init` again",
                name.to_string_lossy()
            );
        }
    }
    Ok(())
}

pub fn move_claude_skills(agent_root: &Path) -> Result<()> {
    let claude_skills = agent_root.join(CLAUDE_DIRNAME).join(SKILLS_DIRNAME);
    if !is_real_dir(&claude_skills) {
        return Ok(());
    }

    let agents_skills = agent_root.join(AGENTS_DIRNAME).join(SKILLS_DIRNAME);
    create_dir_all(&agents_skills)?;
    for name in claude_skill_names(agent_root)? {
        let destination = agents_skills.join(&name);
        fs::rename(claude_skills.join(&name), &destination)
            .with_context(|| format!("could not move skill to `{}`", destination.display()))?;
    }
    fs::remove_dir(&claude_skills)
        .with_context(|| format!("could not remove `{}`", claude_skills.display()))?;
    Ok(())
}

pub fn write_agents_md(agent_root: &Path) -> Result<()> {
    let template = BUNDLED_TEMPLATES_DIR
        .get_file(AGENTS_FILENAME)
        .expect("bundled AGENTS.md is missing")
        .contents();
    let agents = agent_root.join(AGENTS_FILENAME);
    let claude = agent_root.join(CLAUDE_FILENAME);

    let adopted_claude = !agents.exists() && is_regular_file(&claude);
    let existing_path = if agents.exists() {
        Some(&agents)
    } else if adopted_claude {
        Some(&claude)
    } else {
        None
    };
    let contents = match existing_path {
        Some(path) => {
            let existing = fs::read_to_string(path).with_context(|| could_not_read(path))?;
            keep_as_project_section(template, &existing)
        }
        None => template.to_vec(),
    };

    write_file(&agents, contents)?;
    if adopted_claude {
        fs::remove_file(&claude)
            .with_context(|| format!("could not remove `{}`", claude.display()))?;
    }
    symlink_if_absent(&claude, AGENTS_FILENAME)
}

pub fn link_claude_dir(agent_root: &Path) -> Result<()> {
    let claude = agent_root.join(CLAUDE_DIRNAME);
    if is_real_dir(&claude) {
        let skills_target = format!("../{AGENTS_DIRNAME}/{SKILLS_DIRNAME}");
        return symlink_if_absent(&claude.join(SKILLS_DIRNAME), &skills_target);
    }
    symlink_if_absent(&claude, AGENTS_DIRNAME)
}

fn bundled_skill_names() -> Vec<std::ffi::OsString> {
    BUNDLED_TEMPLATES_DIR
        .get_dir(SKILLS_DIRNAME)
        .into_iter()
        .flat_map(|skills| skills.dirs())
        .filter_map(|skill| skill.path().file_name().map(ToOwned::to_owned))
        .collect()
}

fn claude_skill_names(agent_root: &Path) -> Result<Vec<std::ffi::OsString>> {
    let claude_skills = agent_root.join(CLAUDE_DIRNAME).join(SKILLS_DIRNAME);
    if !is_real_dir(&claude_skills) {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in fs::read_dir(&claude_skills).with_context(|| could_not_read_dir(&claude_skills))? {
        let entry = entry.with_context(|| could_not_read_dir(&claude_skills))?;
        names.push(entry.file_name());
    }
    Ok(names)
}

fn is_real_dir(path: &Path) -> bool {
    path.symlink_metadata().is_ok_and(|meta| meta.is_dir())
}

fn is_regular_file(path: &Path) -> bool {
    path.symlink_metadata().is_ok_and(|meta| meta.is_file())
}
