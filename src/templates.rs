use std::fs;
use std::path::Path;

use error_stack::ResultExt;
use include_dir::{Dir, DirEntry, include_dir};
use serde_yaml_ng::Value;

use crate::error::{CliError, Result};
use crate::rule::{RULES_DIR, SCRIPTS_DIR};
use crate::support::util;

#[cfg(test)]
#[path = "templates_test.rs"]
mod templates_test;

static TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

pub const SCHEMAS_DIR: &str = "schemas";
pub const DOCS_DIR: &str = "docs";
pub const SKILLS_DIR: &str = "skills";
pub const AGENTS_DIR_NAME: &str = ".agents";
pub const CLAUDE_DIR_NAME: &str = ".claude";
pub const AGENTS_FILE: &str = "AGENTS.md";
const SKILL_FILE: &str = "SKILL.md";
pub const CLAUDE_FILE: &str = "CLAUDE.md";

// Top-level template categories scaffolded into a spex repo. Shared by `init`
// and `update` so both manage the exact same set of bundled files. Framework
// docs ship from `templates/docs/`, whose files are symlinks into the
// repository's own `docs/` folder — the single source of truth — so the docs
// scaffolded are exactly the ones linked there.
// `rules/` and `scripts/` are handled separately: they are only overwritten
// when the local spec opts in via `sync_with_upstream`. Skills are also
// handled separately: a local skill is left untouched when its
// `metadata.sync-with-upstream` is explicitly `false`.
const MANAGED_FLAT_DIRS: [&str; 2] = [DOCS_DIR, SCHEMAS_DIR];

// Seeds the project's root `AGENTS.md` (and its `CLAUDE.md` symlink) once, during
// `init`. `update` never seeds it, but does keep the managed section of an
// existing one in sync; see `sync_agents_md`.
pub fn ensure_agents_md(spex_dir: &Path) -> Result<()> {
    let agents = spex_dir.join(AGENTS_FILE);
    if !agents.exists()
        && let Some(file) = TEMPLATES.get_file(AGENTS_FILE)
    {
        util::write_file(&agents, file.contents())?;
    }
    ensure_symlink(spex_dir, CLAUDE_FILE, AGENTS_FILE)
}

// Marks where the user-owned section of `AGENTS.md` and of every bundled
// `SKILL.md` begins. Content before this marker is bundled and kept in sync by
// `update`; content from the marker onward (the "Project" section and anything
// the user adds under it) is preserved untouched.
const PROJECT_SECTION_MARKER: &str = "\n---\n\n# Project";

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
    let Some(template) = TEMPLATES.get_file(AGENTS_FILE) else {
        return Ok(());
    };

    let agents = spex_dir.join(AGENTS_FILE);
    let Ok(local_contents) = fs::read_to_string(&agents) else {
        return Ok(());
    };

    let updated = merge_project_section(template.contents(), &local_contents);
    if updated != local_contents.as_bytes() {
        util::write_file(&agents, updated)?;
    }
    Ok(())
}

pub fn scaffold(spex_dir: &Path, removed_rules: &[String]) -> Result<()> {
    for name in MANAGED_FLAT_DIRS {
        copy_flat_dir(name, spex_dir)?;
    }
    copy_rules(spex_dir, removed_rules)?;
    copy_skills(spex_dir)?;
    sync_agents_md(spex_dir)?;
    ensure_symlink(spex_dir, CLAUDE_DIR_NAME, AGENTS_DIR_NAME)?;
    mark_scripts_executable(&spex_dir.join(SCRIPTS_DIR))
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

fn copy_rules(spex_dir: &Path, removed_rules: &[String]) -> Result<()> {
    let Some(dir) = TEMPLATES.get_dir(RULES_DIR) else {
        return Ok(());
    };

    let rules_dest = spex_dir.join(RULES_DIR);
    util::create_dir_all(&rules_dest)?;

    let scripts_dir = TEMPLATES.get_dir(SCRIPTS_DIR);
    let scripts_dest = spex_dir.join(SCRIPTS_DIR);

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

        util::write_file(&dest, file.contents())?;

        if let Some(scripts) = scripts_dir
            && let Some(rule_scripts) = scripts.get_dir(format!("{SCRIPTS_DIR}/{rule_id}"))
        {
            copy_tree(rule_scripts, Path::new(SCRIPTS_DIR), &scripts_dest)?;
        }
    }

    Ok(())
}

fn copy_flat_dir(name: &str, spex_dir: &Path) -> Result<()> {
    let Some(dir) = TEMPLATES.get_dir(name) else {
        return Ok(());
    };
    copy_tree(dir, Path::new(""), spex_dir)
}

fn copy_skills(spex_dir: &Path) -> Result<()> {
    let Some(skills) = TEMPLATES.get_dir(SKILLS_DIR) else {
        return Ok(());
    };

    let dest_root = spex_dir.join(AGENTS_DIR_NAME).join(SKILLS_DIR);
    util::create_dir_all(&dest_root)?;

    for skill in skills.dirs() {
        let name = skill.path().file_name().unwrap_or_default();
        let dest = dest_root.join(name);
        let skill_md = dest.join(SKILL_FILE);

        if dest.exists() && !skill_synced_with_upstream(&skill_md) {
            continue;
        }

        let local_skill_md = fs::read_to_string(&skill_md).ok();
        copy_tree(skill, Path::new(SKILLS_DIR), &dest_root)?;

        if let Some(local) = local_skill_md
            && let Some(template) = skill.get_file(skill.path().join(SKILL_FILE))
        {
            util::write_file(
                &skill_md,
                merge_project_section(template.contents(), &local),
            )?;
        }
    }

    Ok(())
}

fn copy_tree(dir: &Dir, strip: &Path, dest_root: &Path) -> Result<()> {
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(sub) => copy_tree(sub, strip, dest_root)?,
            DirEntry::File(file) => {
                let rel = file.path().strip_prefix(strip).unwrap_or(file.path());
                let dest = dest_root.join(rel);
                if let Some(parent) = dest.parent() {
                    util::create_dir_all(parent)?;
                }
                util::write_file(&dest, file.contents())?;
            }
        }
    }
    Ok(())
}

#[cfg(unix)]
fn ensure_symlink(spex_dir: &Path, link_name: &str, target: &str) -> Result<()> {
    use std::os::unix::fs::symlink;

    let link = spex_dir.join(link_name);
    if link.symlink_metadata().is_ok() {
        return Ok(());
    }

    symlink(target, &link)
        .change_context(CliError::Io)
        .attach_with(|| format!("could not link `{}` to `{target}`", link.display()))
}

#[cfg(not(unix))]
fn ensure_symlink(_spex_dir: &Path, _link_name: &str, _target: &str) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn mark_scripts_executable(scripts_dir: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    if !scripts_dir.is_dir() {
        return Ok(());
    }

    let entries = fs::read_dir(scripts_dir)
        .change_context(CliError::Io)
        .attach_with(|| format!("could not read `{}`", scripts_dir.display()))?;

    for entry in entries {
        let path = entry.change_context(CliError::Io)?.path();
        // Scripts are namespaced under `scripts/<spec-id>/`, so recurse into subdirectories.
        if path.is_dir() {
            mark_scripts_executable(&path)?;
            continue;
        }
        let mut permissions = fs::metadata(&path)
            .change_context(CliError::Io)?
            .permissions();
        permissions.set_mode(permissions.mode() | 0o111);
        fs::set_permissions(&path, permissions)
            .change_context(CliError::Io)
            .attach_with(|| format!("could not set permissions on `{}`", path.display()))?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn mark_scripts_executable(_scripts_dir: &Path) -> Result<()> {
    Ok(())
}
