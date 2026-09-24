use std::fs;
use std::path::Path;

use error_stack::{Report, ResultExt};
use include_dir::{Dir, DirEntry, include_dir};
use thiserror::Error;

use crate::rule::{RULES_DIR, SCRIPTS_DIR};
use crate::util;
use serde::Deserialize;

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

type Result<T> = core::result::Result<T, Report<TemplateError>>;

#[derive(Debug, Error)]
pub enum TemplateError {
    #[error("an I/O operation failed")]
    Io,
}

// Seeds the project's root `AGENTS.md` (and its `CLAUDE.md` symlink) once, during
// `init`. `update` never seeds it, but does keep the managed section of an
// existing one in sync; see `sync_agents_md`.
pub fn ensure_agents_md(spex_dir: &Path) -> Result<()> {
    let agents = spex_dir.join(AGENTS_FILE);
    if !agents.exists()
        && let Some(file) = TEMPLATES.get_file(AGENTS_FILE)
    {
        fs::write(&agents, file.contents())
            .change_context(TemplateError::Io)
            .attach_with(|| format!("could not write `{}`", agents.display()))?;
    }
    ensure_claude_md_link(spex_dir)
}

// Marks where the user-owned section of `AGENTS.md` begins. Content before
// this marker is bundled and kept in sync by `update`; content from the
// marker onward (the "Project" section and anything the user adds under it)
// is preserved untouched.
const AGENTS_MD_SPLIT_MARKER: &str = "\n---\n\n# Project";

fn split_agents_md(contents: &str) -> Option<(&str, &str)> {
    let idx = contents.find(AGENTS_MD_SPLIT_MARKER)?;
    Some(contents.split_at(idx))
}

// Keeps `AGENTS.md` in sync with the bundled template. If the local file has
// `AGENTS_MD_SPLIT_MARKER`, only the managed section above it is replaced,
// preserving the user's "Project" section untouched; a file that lacks the
// marker (e.g. one written before this split existed) is fully overwritten,
// since there is no safe way to tell managed from user content in it.
fn sync_agents_md(spex_dir: &Path) -> Result<()> {
    let Some(template) = TEMPLATES.get_file(AGENTS_FILE) else {
        return Ok(());
    };

    let agents = spex_dir.join(AGENTS_FILE);
    let Ok(local_contents) = fs::read_to_string(&agents) else {
        return Ok(());
    };

    let updated = match split_agents_md(&local_contents) {
        Some((_, preserved)) => {
            let Some(managed) = std::str::from_utf8(template.contents())
                .ok()
                .and_then(split_agents_md)
                .map(|(managed, _)| managed)
            else {
                return Ok(());
            };
            format!("{managed}{preserved}").into_bytes()
        }
        None => template.contents().to_vec(),
    };

    if updated != local_contents.as_bytes() {
        fs::write(&agents, updated)
            .change_context(TemplateError::Io)
            .attach_with(|| format!("could not write `{}`", agents.display()))?;
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
    ensure_claude_link(spex_dir)?;
    mark_scripts_executable(&spex_dir.join(SCRIPTS_DIR))
}

#[derive(Deserialize, Default)]
struct SyncWithUpstreamFlag {
    #[serde(default)]
    sync_with_upstream: bool,
}

// A spec opts into `spex update` overwriting it (and its scripts) by setting
// `sync_with_upstream: true`. If the local file is missing or fails to parse,
// it is treated as not synced, so a brand-new local spec never gets silently
// overwritten by a same-named bundled one.
fn is_synced_with_upstream(spec_path: &Path) -> bool {
    fs::read_to_string(spec_path)
        .ok()
        .and_then(|raw| serde_yaml_ng::from_str::<SyncWithUpstreamFlag>(&raw).ok())
        .is_some_and(|flag| flag.sync_with_upstream)
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
struct SkillMetadata {
    #[serde(rename = "sync-with-upstream", default = "default_true")]
    sync_with_upstream: bool,
}

impl Default for SkillMetadata {
    fn default() -> Self {
        Self {
            sync_with_upstream: true,
        }
    }
}

#[derive(Deserialize, Default)]
struct SkillFrontmatter {
    #[serde(default)]
    metadata: SkillMetadata,
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
        .and_then(|raw| {
            let frontmatter = frontmatter(&raw)?;
            serde_yaml_ng::from_str::<SkillFrontmatter>(frontmatter).ok()
        })
        .is_none_or(|parsed| parsed.metadata.sync_with_upstream)
}

fn copy_rules(spex_dir: &Path, removed_rules: &[String]) -> Result<()> {
    let Some(dir) = TEMPLATES.get_dir(RULES_DIR) else {
        return Ok(());
    };

    let rules_dest = spex_dir.join(RULES_DIR);
    fs::create_dir_all(&rules_dest)
        .change_context(TemplateError::Io)
        .attach_with(|| format!("could not create `{}`", rules_dest.display()))?;

    let scripts_dir = TEMPLATES.get_dir(SCRIPTS_DIR);
    let scripts_dest = spex_dir.join(SCRIPTS_DIR);

    // Rules may be grouped into nested folders, so a bundled rule already
    // present under a nested folder must be matched there rather than
    // re-created flat at `rules_dest/<name>`.
    let existing_rules = util::yaml_files(&rules_dest);

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

        fs::write(&dest, file.contents())
            .change_context(TemplateError::Io)
            .attach_with(|| format!("could not write `{}`", dest.display()))?;

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
    fs::create_dir_all(&dest_root)
        .change_context(TemplateError::Io)
        .attach_with(|| format!("could not create `{}`", dest_root.display()))?;

    for skill in skills.dirs() {
        let name = skill.path().file_name().unwrap_or_default();
        let dest = dest_root.join(name);

        if dest.exists() {
            if !skill_synced_with_upstream(&dest.join("SKILL.md")) {
                continue;
            }
            // Replace each managed skill wholesale so renamed or removed files don't linger.
            fs::remove_dir_all(&dest)
                .change_context(TemplateError::Io)
                .attach_with(|| format!("could not remove `{}`", dest.display()))?;
        }
        copy_tree(skill, Path::new(SKILLS_DIR), &dest_root)?;
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
                    fs::create_dir_all(parent)
                        .change_context(TemplateError::Io)
                        .attach_with(|| format!("could not create `{}`", parent.display()))?;
                }
                fs::write(&dest, file.contents())
                    .change_context(TemplateError::Io)
                    .attach_with(|| format!("could not write `{}`", dest.display()))?;
            }
        }
    }
    Ok(())
}

#[cfg(unix)]
fn ensure_claude_link(spex_dir: &Path) -> Result<()> {
    use std::os::unix::fs::symlink;

    let link = spex_dir.join(CLAUDE_DIR_NAME);
    if link.symlink_metadata().is_ok() {
        return Ok(());
    }

    symlink(AGENTS_DIR_NAME, &link)
        .change_context(TemplateError::Io)
        .attach_with(|| {
            format!(
                "could not link `{}` to `{}`",
                link.display(),
                AGENTS_DIR_NAME
            )
        })
}

#[cfg(not(unix))]
fn ensure_claude_link(_spex_dir: &Path) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn ensure_claude_md_link(spex_dir: &Path) -> Result<()> {
    use std::os::unix::fs::symlink;

    let link = spex_dir.join(CLAUDE_FILE);
    if link.symlink_metadata().is_ok() {
        return Ok(());
    }

    symlink(AGENTS_FILE, &link)
        .change_context(TemplateError::Io)
        .attach_with(|| format!("could not link `{}` to `{}`", link.display(), AGENTS_FILE))
}

#[cfg(not(unix))]
fn ensure_claude_md_link(_spex_dir: &Path) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn mark_scripts_executable(scripts_dir: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    if !scripts_dir.is_dir() {
        return Ok(());
    }

    let entries = fs::read_dir(scripts_dir)
        .change_context(TemplateError::Io)
        .attach_with(|| format!("could not read `{}`", scripts_dir.display()))?;

    for entry in entries {
        let path = entry.change_context(TemplateError::Io)?.path();
        // Scripts are namespaced under `scripts/<spec-id>/`, so recurse into subdirectories.
        if path.is_dir() {
            mark_scripts_executable(&path)?;
            continue;
        }
        let mut permissions = fs::metadata(&path)
            .change_context(TemplateError::Io)?
            .permissions();
        permissions.set_mode(permissions.mode() | 0o111);
        fs::set_permissions(&path, permissions)
            .change_context(TemplateError::Io)
            .attach_with(|| format!("could not set permissions on `{}`", path.display()))?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn mark_scripts_executable(_scripts_dir: &Path) -> Result<()> {
    Ok(())
}
