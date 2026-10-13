use std::fs;
use std::path::{Component, Path, PathBuf};

use eros::{Context, IntoAnyUnion};
use include_dir::{Dir, DirEntry};

use crate::error::{Result, could_not_read_dir};

#[cfg(test)]
#[path = "fs_test.rs"]
mod fs_test;

pub fn file_url(path: &Path) -> String {
    anstyle_hyperlink::file_to_url(None, &canonical(path)).unwrap_or_default()
}

pub fn current_dir() -> Result<PathBuf> {
    std::env::current_dir()
        .context("could not determine the current directory")
        .any_union()
}

pub fn write_file(path: &Path, contents: impl AsRef<[u8]>) -> Result<()> {
    fs::write(path, contents)
        .with_context(|| format!("could not write `{}`", path.display()))
        .any_union()
}

pub fn create_dir_all(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .with_context(|| format!("could not create directory `{}`", path.display()))
        .any_union()
}

pub fn dir_name(path: &Path) -> Option<&str> {
    path.file_name().and_then(|name| name.to_str())
}

pub fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

pub fn normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir if normalized.pop() => {}
            other => normalized.push(other),
        }
    }
    normalized
}

pub fn relative_to<'a>(path: &'a Path, base: &Path) -> &'a Path {
    path.strip_prefix(base).unwrap_or(path)
}

pub fn write_embedded_dir(dir: &Dir, dest_root: &Path) -> Result<()> {
    write_embedded_dir_at(dir, &dest_root.join(dir.path()))
}

pub fn write_embedded_dir_at(dir: &Dir, dest_dir: &Path) -> Result<()> {
    write_embedded_entries(dir, dir.path(), dest_dir)
}

fn write_embedded_entries(dir: &Dir, root: &Path, dest_dir: &Path) -> Result<()> {
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(sub) => write_embedded_entries(sub, root, dest_dir)?,
            DirEntry::File(file) => {
                let relative = file.path().strip_prefix(root).unwrap_or(file.path());
                let dest = dest_dir.join(relative);
                if let Some(parent) = dest.parent() {
                    create_dir_all(parent)?;
                }
                write_file(&dest, file.contents())?;
            }
        }
    }
    Ok(())
}

#[cfg(unix)]
pub fn symlink_if_absent(link: &Path, target: &str) -> Result<()> {
    use std::os::unix::fs::symlink;

    if link.symlink_metadata().is_ok() {
        return Ok(());
    }

    symlink(target, link)
        .with_context(|| format!("could not link `{}` to `{target}`", link.display()))
        .any_union()
}

#[cfg(not(unix))]
pub fn symlink_if_absent(_link: &Path, _target: &str) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
pub fn make_files_executable(dir: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    use walkdir::WalkDir;

    if !dir.is_dir() {
        return Ok(());
    }

    for entry in WalkDir::new(dir) {
        let entry = entry.with_context(|| could_not_read_dir(dir))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let mut permissions = entry
            .metadata()
            .with_context(|| format!("could not read metadata of `{}`", path.display()))?
            .permissions();
        permissions.set_mode(permissions.mode() | 0o111);
        fs::set_permissions(path, permissions)
            .with_context(|| format!("could not set permissions on `{}`", path.display()))?;
    }
    Ok(())
}

#[cfg(not(unix))]
pub fn make_files_executable(_dir: &Path) -> Result<()> {
    Ok(())
}
