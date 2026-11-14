use std::path::{Path, PathBuf};

use include_dir::{Dir, File};

use crate::error::Result;
use crate::support::fs::{make_files_executable, write_embedded_dir_at, write_file};
use crate::templates::{BUNDLED_TEMPLATES_DIR, RULES_LIBRARY_DIRNAME};

pub fn bundled_rules() -> Vec<&'static File<'static>> {
    let mut rules = Vec::new();
    if let Some(dir) = BUNDLED_TEMPLATES_DIR.get_dir(RULES_LIBRARY_DIRNAME) {
        collect_rules(dir, &mut rules);
    }
    rules.sort_by_key(|rule| rule.path().file_name());
    rules
}

// A subfolder groups rules unless a rule file of its name sits beside it, making it that rule's script folder.
fn collect_rules(dir: &'static Dir<'static>, rules: &mut Vec<&'static File<'static>>) {
    rules.extend(dir.files().filter(|file| {
        file.path()
            .extension()
            .is_some_and(|extension| extension == "yaml")
    }));
    for subdir in dir.dirs() {
        let is_script_folder = BUNDLED_TEMPLATES_DIR
            .get_file(subdir.path().with_extension("yaml"))
            .is_some();
        if !is_script_folder {
            collect_rules(subdir, rules);
        }
    }
}

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
