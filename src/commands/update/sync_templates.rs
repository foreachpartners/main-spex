use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use eros::Context;
use serde_yaml_ng::Value;

use crate::error::{Result, could_not_read};
use crate::rule::RULES_DIR;
use crate::support::bundled_agents::bundled_agents_md;
use crate::support::bundled_rule::bundled_rules;
use crate::support::managed_section::replace_managed_section;
use crate::support::yaml_dir::yaml_files;
use crate::templates::{
    AGENTS_DIRNAME, AGENTS_FILENAME, BUNDLED_TEMPLATES_DIR, SCHEMAS_DIRNAME, SKILLS_DIRNAME,
};

use super::managed_files::ManagedFiles;

#[cfg(test)]
#[path = "sync_templates_test.rs"]
mod sync_templates_test;

const SKILL_FILE: &str = "SKILL.md";

pub(super) fn template_updates(spex_dir: &Path, agent_root: &Path) -> Result<ManagedFiles> {
    let mut updates = ManagedFiles::default();
    let schemas = BUNDLED_TEMPLATES_DIR
        .get_dir(SCHEMAS_DIRNAME)
        .expect("bundled schemas");
    updates.add_directory(schemas, &spex_dir.join(SCHEMAS_DIRNAME), false)?;
    sync_rules(&mut updates, spex_dir)?;
    sync_skills(&mut updates, agent_root)?;
    sync_agents_md(&mut updates, agent_root)?;
    Ok(updates)
}

#[cfg(test)]
fn sync_templates(spex_dir: &Path, agent_root: &Path) -> Result<()> {
    template_updates(spex_dir, agent_root)?.apply()
}

fn sync_agents_md(updates: &mut ManagedFiles, agent_root: &Path) -> Result<()> {
    let Some(template) = bundled_agents_md() else {
        return Ok(());
    };
    let agents = agent_root.join(AGENTS_FILENAME);
    let Some(local_contents) = read_if_present(&agents)? else {
        return Ok(());
    };

    let updated = replace_managed_section(template.as_bytes(), &local_contents);
    updates.add(agents, &updated, false)
}

fn is_synced_with_upstream(raw: &str) -> bool {
    serde_yaml_ng::from_str::<Value>(raw)
        .ok()
        .and_then(|spec| spec.get("sync_with_upstream")?.as_bool())
        .unwrap_or(true)
}

fn frontmatter(contents: &str) -> Option<&str> {
    let rest = contents.strip_prefix("---")?;
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}

fn skill_synced_with_upstream(skill: &str) -> bool {
    frontmatter(skill)
        .and_then(|raw| serde_yaml_ng::from_str::<Value>(raw).ok())
        .and_then(|frontmatter| {
            frontmatter
                .get("metadata")?
                .get("sync-with-upstream")?
                .as_bool()
        })
        .unwrap_or(true)
}

fn sync_rules(updates: &mut ManagedFiles, spex_dir: &Path) -> Result<()> {
    let local_rules = yaml_files(&spex_dir.join(RULES_DIR))?;

    for file in bundled_rules() {
        let Some(name) = file.path().file_name() else {
            continue;
        };
        let Some(local) = local_rules
            .iter()
            .find(|path| path.file_name() == Some(name))
        else {
            continue;
        };

        let raw = fs::read_to_string(local).with_context(|| could_not_read(local))?;
        if !is_synced_with_upstream(&raw) {
            continue;
        }
        updates.add(local.clone(), file.contents(), false)?;
        if let Some(scripts) = BUNDLED_TEMPLATES_DIR.get_dir(file.path().with_extension("")) {
            updates.add_directory(scripts, &local.with_extension(""), true)?;
        }
    }

    Ok(())
}

fn sync_skills(updates: &mut ManagedFiles, agent_root: &Path) -> Result<()> {
    let Some(skills) = BUNDLED_TEMPLATES_DIR.get_dir(SKILLS_DIRNAME) else {
        return Ok(());
    };

    let dest_root = agent_root.join(AGENTS_DIRNAME).join(SKILLS_DIRNAME);
    for skill in skills.dirs() {
        let name = skill.path().file_name().expect("bundled skill name");
        let dest = dest_root.join(name);
        let skill_md = dest.join(SKILL_FILE);
        let local = read_if_present(&skill_md)?;
        if local
            .as_deref()
            .is_some_and(|contents| !skill_synced_with_upstream(contents))
        {
            continue;
        }

        for file in skill.files() {
            let basename = file.path().file_name().expect("bundled skill file name");
            let bytes = match &local {
                Some(local) if basename == SKILL_FILE => {
                    replace_managed_section(file.contents(), local)
                }
                _ => file.contents().to_vec(),
            };
            updates.add(dest.join(basename), &bytes, false)?;
        }
        for subdir in skill.dirs() {
            let path = dest.join(subdir.path().file_name().expect("bundled directory name"));
            updates.add_directory(subdir, &path, false)?;
        }
    }

    Ok(())
}

fn read_if_present(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(Some(contents)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(eros::error!("{}: {error}", could_not_read(path))),
    }
}
