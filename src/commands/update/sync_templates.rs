use std::fs;
use std::path::Path;

use serde_yaml_ng::Value;

use crate::commands::init::scaffold::copy_docs_and_schemas;
use crate::error::Result;
use crate::rule::RULES_DIR;
use crate::support::bundled_agents::bundled_agents_md;
use crate::support::bundled_rule::write_bundled_rule;
use crate::support::fs::{create_dir_all, write_embedded_dir, write_file};
use crate::support::managed_section::replace_managed_section;
use crate::support::yaml_dir::yaml_files;
use crate::templates::{
    AGENTS_DIRNAME, AGENTS_FILENAME, BUNDLED_TEMPLATES_DIR, RULES_LIBRARY_DIRNAME, SKILLS_DIRNAME,
};

#[cfg(test)]
#[path = "sync_templates_test.rs"]
mod sync_templates_test;

const SKILL_FILE: &str = "SKILL.md";

pub fn sync_templates(spex_dir: &Path, agent_root: &Path) -> Result<()> {
    copy_docs_and_schemas(spex_dir)?;
    sync_rules(spex_dir)?;
    sync_skills(agent_root)?;
    sync_agents_md(agent_root)
}

fn sync_agents_md(agent_root: &Path) -> Result<()> {
    let Some(template) = bundled_agents_md() else {
        return Ok(());
    };

    let agents = agent_root.join(AGENTS_FILENAME);
    let Ok(local_contents) = fs::read_to_string(&agents) else {
        return Ok(());
    };

    let updated = replace_managed_section(template.as_bytes(), &local_contents);
    if updated != local_contents.as_bytes() {
        write_file(&agents, updated)?;
    }
    Ok(())
}

fn is_synced_with_upstream(spec_path: &Path) -> bool {
    fs::read_to_string(spec_path)
        .ok()
        .and_then(|raw| serde_yaml_ng::from_str::<Value>(&raw).ok())
        .and_then(|spec| spec.get("sync_with_upstream")?.as_bool())
        .unwrap_or(true)
}

fn frontmatter(contents: &str) -> Option<&str> {
    let rest = contents.strip_prefix("---")?;
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}

fn skill_synced_with_upstream(skill_md_path: &Path) -> bool {
    fs::read_to_string(skill_md_path)
        .ok()
        .and_then(|raw| serde_yaml_ng::from_str::<Value>(frontmatter(&raw)?).ok())
        .and_then(|frontmatter| {
            frontmatter
                .get("metadata")?
                .get("sync-with-upstream")?
                .as_bool()
        })
        .unwrap_or(true)
}

fn sync_rules(spex_dir: &Path) -> Result<()> {
    let Some(dir) = BUNDLED_TEMPLATES_DIR.get_dir(RULES_LIBRARY_DIRNAME) else {
        return Ok(());
    };

    let local_rules = yaml_files(&spex_dir.join(RULES_DIR))?;

    for file in dir.files() {
        let Some(name) = file.path().file_name() else {
            continue;
        };
        let Some(local) = local_rules
            .iter()
            .find(|path| path.file_name() == Some(name))
        else {
            continue;
        };

        if is_synced_with_upstream(local) {
            write_bundled_rule(file, local)?;
        }
    }

    Ok(())
}

fn sync_skills(agent_root: &Path) -> Result<()> {
    let Some(skills) = BUNDLED_TEMPLATES_DIR.get_dir(SKILLS_DIRNAME) else {
        return Ok(());
    };

    let agents_dir = agent_root.join(AGENTS_DIRNAME);
    let dest_root = agents_dir.join(SKILLS_DIRNAME);
    create_dir_all(&dest_root)?;

    for skill in skills.dirs() {
        let name = skill.path().file_name().unwrap_or_default();
        let dest = dest_root.join(name);
        let skill_md = dest.join(SKILL_FILE);

        if dest.exists() && !skill_synced_with_upstream(&skill_md) {
            continue;
        }

        let local_skill_md = fs::read_to_string(&skill_md).ok();
        write_embedded_dir(skill, &agents_dir)?;

        if let Some(local) = local_skill_md
            && let Some(template) = skill.get_file(skill.path().join(SKILL_FILE))
        {
            write_file(
                &skill_md,
                replace_managed_section(template.contents(), &local),
            )?;
        }
    }

    Ok(())
}
