use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use eros::{Context, bail};
use include_dir::Dir;
use tempfile::{Builder, NamedTempFile};

use crate::error::{Result, could_not_read};
use crate::support::fs::create_dir_all;

#[derive(Default)]
pub(super) struct ManagedFiles(Vec<Edit>);

pub(super) struct Edit {
    pub(super) path: PathBuf,
    bytes: Vec<u8>,
    previous: Option<FileState>,
    executable: bool,
}

#[derive(PartialEq, Eq)]
struct FileState {
    bytes: Vec<u8>,
    permissions: fs::Permissions,
}

impl Edit {
    pub(super) fn action(&self) -> &'static str {
        if self.previous.is_some() {
            "Update"
        } else {
            "Create"
        }
    }
}

impl ManagedFiles {
    pub(super) fn list(&self) -> &[Edit] {
        &self.0
    }

    pub(super) fn add(&mut self, path: PathBuf, bytes: &[u8], executable: bool) -> Result<()> {
        let previous = read_optional(&path)?;
        if previous.as_ref().is_some_and(|existing| {
            existing.bytes == bytes && has_required_mode(&existing.permissions, executable)
        }) {
            return Ok(());
        }
        self.0.push(Edit {
            path,
            bytes: bytes.to_vec(),
            previous,
            executable,
        });
        Ok(())
    }

    pub(super) fn add_directory(
        &mut self,
        template: &Dir,
        dest: &Path,
        executable: bool,
    ) -> Result<()> {
        for file in template.files() {
            let path = dest.join(file.path().file_name().expect("bundled file name"));
            self.add(path, file.contents(), executable)?;
        }
        for subdir in template.dirs() {
            let path = dest.join(subdir.path().file_name().expect("bundled directory name"));
            self.add_directory(subdir, &path, executable)?;
        }
        Ok(())
    }

    pub(super) fn apply(self) -> Result<()> {
        let mut staged = Vec::with_capacity(self.0.len());
        for edit in self.0 {
            let parent = edit.path.parent().expect("managed file has a parent");
            create_dir_all(parent)?;
            let mut temp = staging_file(parent)
                .with_context(|| format!("could not stage {}", edit.path.display()))?;
            temp.write_all(&edit.bytes)
                .with_context(|| format!("could not stage {}", edit.path.display()))?;
            let permissions = edit.previous.as_ref().map(|file| file.permissions.clone());
            prepare_permissions(temp.as_file(), permissions, edit.executable).with_context(
                || format!("could not set permissions for {}", edit.path.display()),
            )?;
            staged.push((edit, temp));
        }
        for (edit, _) in &staged {
            if read_optional(&edit.path)? != edit.previous {
                bail!("{} changed while preparing the update", edit.path.display());
            }
        }
        for (edit, temp) in staged {
            temp.persist(&edit.path)
                .map_err(|error| error.error)
                .with_context(|| format!("could not update {}", edit.path.display()))?;
        }
        Ok(())
    }
}

fn read_optional(path: &Path) -> Result<Option<FileState>> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(eros::error!(
                "could not inspect {}: {error}",
                path.display()
            ));
        }
    };
    if metadata.is_symlink() {
        bail!(
            "managed update target is a symbolic link: {}",
            path.display()
        );
    }
    if !metadata.is_file() {
        bail!(
            "managed update target is not a regular file: {}",
            path.display()
        );
    }
    let bytes = fs::read(path).with_context(|| could_not_read(path))?;
    Ok(Some(FileState {
        bytes,
        permissions: metadata.permissions(),
    }))
}

fn staging_file(parent: &Path) -> std::io::Result<NamedTempFile> {
    let mut builder = Builder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        builder.permissions(fs::Permissions::from_mode(0o666));
    }
    builder.tempfile_in(parent)
}

#[cfg(unix)]
fn has_required_mode(permissions: &fs::Permissions, executable: bool) -> bool {
    use std::os::unix::fs::PermissionsExt;

    !executable || permissions.mode() & 0o111 == 0o111
}

#[cfg(not(unix))]
fn has_required_mode(_permissions: &fs::Permissions, _executable: bool) -> bool {
    true
}

#[cfg(unix)]
fn prepare_permissions(
    file: &fs::File,
    permissions: Option<fs::Permissions>,
    executable: bool,
) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    if permissions.is_none() && !executable {
        return Ok(());
    }
    let mut permissions = match permissions {
        Some(mode) => mode,
        None => file.metadata()?.permissions(),
    };
    if executable {
        permissions.set_mode(permissions.mode() | 0o111);
    }
    file.set_permissions(permissions)
}

#[cfg(not(unix))]
fn prepare_permissions(
    file: &fs::File,
    permissions: Option<fs::Permissions>,
    _executable: bool,
) -> std::io::Result<()> {
    if let Some(permissions) = permissions {
        file.set_permissions(permissions)?;
    }
    Ok(())
}
