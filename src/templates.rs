use std::fs;
use std::path::{Path, PathBuf};

use error_stack::ResultExt;
use include_dir::{Dir, DirEntry, include_dir};

use crate::error::{CliError, Result};
use crate::rule::{RULES_DIR, SCRIPTS_DIR};
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
    mark_scripts_executable(&spex_dir.join(SCRIPTS_DIR))
}

pub fn installed_skills_dir(spex_dir: &Path) -> PathBuf {
    spex_dir.join(AGENTS_DIR_NAME).join(SKILLS_DIR)
}

pub fn copy_plain_dirs(spex_dir: &Path) -> Result<()> {
    for name in PLAIN_DIRS {
        let dir = TEMPLATES
            .get_dir(name)
            .expect("bundled template directory is missing");
        copy_tree(dir, Path::new(""), spex_dir)?;
    }
    Ok(())
}

pub fn copy_rule_scripts(rule_id: &str, spex_dir: &Path) -> Result<()> {
    let Some(scripts) = TEMPLATES.get_dir(format!("{SCRIPTS_DIR}/{rule_id}")) else {
        return Ok(());
    };
    copy_tree(scripts, Path::new(""), spex_dir)
}

fn copy_rules(spex_dir: &Path) -> Result<()> {
    let Some(rules) = TEMPLATES.get_dir(RULES_DIR) else {
        return Ok(());
    };

    copy_tree(rules, Path::new(""), spex_dir)?;
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

    copy_tree(
        skills,
        Path::new(SKILLS_DIR),
        &installed_skills_dir(spex_dir),
    )
}

fn copy_agents_md(spex_dir: &Path) -> Result<()> {
    let agents = TEMPLATES
        .get_file(AGENTS_FILE)
        .expect("bundled AGENTS.md is missing");
    util::write_file(&spex_dir.join(AGENTS_FILE), agents.contents())?;
    ensure_symlink(spex_dir, CLAUDE_FILE, AGENTS_FILE)
}

pub fn copy_tree(dir: &Dir, strip: &Path, dest_root: &Path) -> Result<()> {
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

pub fn link_claude_dir(spex_dir: &Path) -> Result<()> {
    ensure_symlink(spex_dir, CLAUDE_DIR_NAME, AGENTS_DIR_NAME)
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
pub fn mark_scripts_executable(scripts_dir: &Path) -> Result<()> {
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
pub fn mark_scripts_executable(_scripts_dir: &Path) -> Result<()> {
    Ok(())
}
