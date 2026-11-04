use std::path::{Path, PathBuf};

use include_dir::File;

use crate::error::Result;
use crate::support::fs::{make_files_executable, write_embedded_dir_at, write_file};
use crate::templates::BUNDLED_TEMPLATES_DIR;

pub fn write_bundled_rule(rule: &File, dest: &Path) -> Result<()> {
    write_file(dest, rule.contents())?;

    let Some(scripts) = BUNDLED_TEMPLATES_DIR.get_dir(rule.path().with_extension("")) else {
        return Ok(());
    };
    let scripts_dest = dest.with_extension("");
    write_embedded_dir_at(scripts, &scripts_dest)?;
    make_files_executable(&scripts_dest)
}

pub fn bundled_rule_paths(rule: &File, dest: &Path) -> Vec<PathBuf> {
    let has_scripts = BUNDLED_TEMPLATES_DIR
        .get_dir(rule.path().with_extension(""))
        .is_some();
    let mut paths = vec![dest.to_path_buf()];
    if has_scripts {
        paths.push(dest.with_extension(""));
    }
    paths
}
