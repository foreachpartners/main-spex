use std::fs;
use std::path::Path;

use serde_yaml_ng::Value;

use crate::commands::init::scaffold::{copy_docs_and_schemas, link_claude_dir, write_rule};
use crate::error::Result;
use crate::rule::RULES_DIR;
use crate::support::fs::{create_dir_all, write_embedded_dir, write_file};
use crate::support::util;
use crate::templates::{AGENTS_DIRNAME, AGENTS_FILENAME, BUNDLED_TEMPLATES_DIR, SKILLS_DIRNAME};

#[cfg(test)]
#[path = "sync_templates_test.rs"]
mod sync_templates_test;

const SKILL_FILE: &str = "SKILL.md";

// Marks where the user-owned section of `AGENTS.md` and of every bundled
// `SKILL.md` begins. Content before this marker is bundled and kept in sync by
// `update`; content from the marker onward (the "Project" section and anything
// the user adds under it) is preserved untouched.
const PROJECT_SECTION_MARKER: &str = "\n---\n\n# Project";

pub fn sync_templates(spex_dir: &Path, removed_rules: &[String]) -> Result<()> {
    copy_docs_and_schemas(spex_dir)?;
    sync_rules(spex_dir, removed_rules)?;
    sync_skills(spex_dir)?;
    sync_agents_md(spex_dir)?;
    link_claude_dir(spex_dir)
}

fn split_project_section(contents: &str) -> Option<(&str, &str)> {
    let idx = contents.find(PROJECT_SECTION_MARKER)?;
    Some(contents.split_at(idx))
}

// Merges a bundled file with its local copy. If the local file has
// `PROJECT_SECTION_MARKER`, only the managed section above it is replaced,
// preserving the user's "Project" section untouched; a file that lacks the
// marker (e.g. one written before this split existed) is fully overwritten,
// since there is no safe way to tell managed from user content in it.
fn merge_project_section(template: &[u8], local: &str) -> Vec<u8> {
    let Some((_, preserved)) = split_project_section(local) else {
        return template.to_vec();
    };
    let Some((managed, _)) = std::str::from_utf8(template)
        .ok()
        .and_then(split_project_section)
    else {
        return local.as_bytes().to_vec();
    };
    format!("{managed}{preserved}").into_bytes()
}

fn sync_agents_md(spex_dir: &Path) -> Result<()> {
    let Some(template) = BUNDLED_TEMPLATES_DIR.get_file(AGENTS_FILENAME) else {
        return Ok(());
    };

    let agents = spex_dir.join(AGENTS_FILENAME);
    let Ok(local_contents) = fs::read_to_string(&agents) else {
        return Ok(());
    };

    let updated = merge_project_section(template.contents(), &local_contents);
    if updated != local_contents.as_bytes() {
        write_file(&agents, updated)?;
    }
    Ok(())
}

// A spec opts into `spex update` overwriting it (and its scripts) by setting
// `sync_with_upstream: true`. If the local file is missing or fails to parse,
// it is treated as not synced, so a brand-new local spec never gets silently
// overwritten by a same-named bundled one.
fn is_synced_with_upstream(spec_path: &Path) -> bool {
    fs::read_to_string(spec_path)
        .ok()
        .and_then(|raw| serde_yaml_ng::from_str::<Value>(&raw).ok())
        .and_then(|spec| spec.get("sync_with_upstream")?.as_bool())
        .unwrap_or(false)
}

// Extracts the YAML frontmatter (the block between the leading `---` lines)
// from a `SKILL.md` file's contents.
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

fn sync_rules(spex_dir: &Path, removed_rules: &[String]) -> Result<()> {
    let Some(dir) = BUNDLED_TEMPLATES_DIR.get_dir(RULES_DIR) else {
        return Ok(());
    };

    let rules_dest = spex_dir.join(RULES_DIR);
    create_dir_all(&rules_dest)?;

    // Rules may be grouped into nested folders, so a bundled rule already
    // present under a nested folder must be matched there rather than
    // re-created flat at `rules_dest/<name>`.
    let existing_rules = util::yaml_files(&rules_dest)?;

    for file in dir.files() {
        let Some(name) = file.path().file_name() else {
            continue;
        };
        let dest = existing_rules
            .iter()
            .find(|path| path.file_name() == Some(name))
            .cloned()
            .unwrap_or_else(|| rules_dest.join(name));
        let rule_id = dest.file_stem().unwrap_or_default().to_string_lossy();

        if dest.exists() {
            if !is_synced_with_upstream(&dest) {
                continue;
            }
        } else if removed_rules
            .iter()
            .any(|id| id.eq_ignore_ascii_case(rule_id.as_ref()))
        {
            continue;
        }

        write_rule(file, &dest)?;
    }

    Ok(())
}

fn sync_skills(spex_dir: &Path) -> Result<()> {
    let Some(skills) = BUNDLED_TEMPLATES_DIR.get_dir(SKILLS_DIRNAME) else {
        return Ok(());
    };

    let agents_dir = spex_dir.join(AGENTS_DIRNAME);
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
                merge_project_section(template.contents(), &local),
            )?;
        }
    }

    Ok(())
}
