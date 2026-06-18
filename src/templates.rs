use std::fs;
use std::path::Path;

use error_stack::{Report, ResultExt};
use include_dir::{Dir, DirEntry, include_dir};
use thiserror::Error;

use crate::tech_spec::{SCRIPTS_DIR, TECH_SPECS_DIR};

#[cfg(test)]
#[path = "templates_test.rs"]
mod templates_test;

static TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

pub const SCHEMAS_DIR: &str = "schemas";
pub const SKILLS_DIR: &str = "skills";
pub const AGENTS_DIR_NAME: &str = ".agents";
pub const CLAUDE_DIR_NAME: &str = ".claude";

// Top-level template categories scaffolded into a spex repo. Shared by `init`
// and `update` so both manage the exact same set of bundled files.
const MANAGED_FLAT_DIRS: [&str; 3] = [SCHEMAS_DIR, SCRIPTS_DIR, TECH_SPECS_DIR];

type Result<T> = core::result::Result<T, Report<TemplateError>>;

#[derive(Debug, Error)]
pub enum TemplateError {
    #[error("an I/O operation failed")]
    Io,
}

pub fn tech_specs_dir() -> Option<&'static Dir<'static>> {
    TEMPLATES.get_dir(TECH_SPECS_DIR)
}

// Writes every bundled template into the spex repo, overwriting managed files
// while leaving any additional files the user added in place.
pub fn scaffold(spex_dir: &Path) -> Result<()> {
    for name in MANAGED_FLAT_DIRS {
        copy_flat_dir(name, spex_dir)?;
    }
    copy_skills(spex_dir)?;
    ensure_claude_link(spex_dir)?;
    mark_scripts_executable(&spex_dir.join(SCRIPTS_DIR))
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
        // Replace each managed skill wholesale so renamed or removed files don't linger.
        if dest.exists() {
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
