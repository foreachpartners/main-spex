use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::rule::{Check, CheckKind, Severity};

use super::scoped_file::ScopedFile;

pub(super) fn scoped_file(dir: &TempDir, rel: &str, contents: &str) -> ScopedFile {
    let abs = dir.path().join(rel);
    write_file(&abs, contents);
    ScopedFile {
        abs,
        rel: PathBuf::from(rel),
    }
}

pub(super) fn write_script(path: &Path, contents: &str) {
    write_file(path, contents);
    let mut permissions = std::fs::metadata(path).expect("metadata").permissions();
    permissions.set_mode(permissions.mode() | 0o111);
    std::fs::set_permissions(path, permissions).expect("set permissions");
}

pub(super) fn check(kind: CheckKind) -> Check {
    Check {
        name: "sample".to_string(),
        description: None,
        kind,
        include: Vec::new(),
        exclude: Vec::new(),
        severity: Severity::Error,
    }
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent");
    }
    std::fs::write(path, contents).expect("write file");
}
