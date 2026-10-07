use std::fs;
use std::path::{Path, PathBuf};

use error_stack::ResultExt;
use include_dir::{Dir, DirEntry};

use crate::error::{CliError, Result};

#[cfg(test)]
#[path = "fs_test.rs"]
mod fs_test;

pub fn file_url(path: &Path) -> String {
    format!(
        "file://{}",
        canonical(path).to_string_lossy().replace(' ', "%20")
    )
}

pub fn write_file(path: &Path, contents: impl AsRef<[u8]>) -> Result<()> {
    fs::write(path, contents).change_context_lazy(|| CliError::WriteFile(path.to_path_buf()))
}

pub fn create_dir_all(path: &Path) -> Result<()> {
    fs::create_dir_all(path).change_context_lazy(|| CliError::CreateDir(path.to_path_buf()))
}

pub fn dir_name(path: &Path) -> Option<&str> {
    path.file_name().and_then(|name| name.to_str())
}

pub fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

pub fn write_embedded_dir(dir: &Dir, dest_root: &Path) -> Result<()> {
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(sub) => write_embedded_dir(sub, dest_root)?,
            DirEntry::File(file) => {
                let dest = dest_root.join(file.path());
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
        .change_context(CliError::Io)
        .attach_with(|| format!("could not link `{}` to `{target}`", link.display()))
}

#[cfg(not(unix))]
pub fn symlink_if_absent(_link: &Path, _target: &str) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
pub fn make_files_executable(dir: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    if !dir.is_dir() {
        return Ok(());
    }

    let entries = fs::read_dir(dir)
        .change_context(CliError::Io)
        .attach_with(|| format!("could not read `{}`", dir.display()))?;

    for entry in entries {
        let path = entry.change_context(CliError::Io)?.path();
        if path.is_dir() {
            make_files_executable(&path)?;
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
pub fn make_files_executable(_dir: &Path) -> Result<()> {
    Ok(())
}
