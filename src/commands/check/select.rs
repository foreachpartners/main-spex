use eros::bail;

use crate::config::Package;
use crate::config::spex::Spex;
use crate::error::Result;

use super::scoped_file::ScopedFile;

#[cfg(test)]
#[path = "select_test.rs"]
mod select_test;

pub(super) fn select_packages<'a>(
    spex: &'a Spex,
    names: &[String],
    files: &[String],
) -> Result<Vec<&'a Package>> {
    if files.is_empty() {
        return spex.select_packages(names);
    }

    spex.validate_package_names(names)?;

    for file in files {
        let found = spex.packages.iter().any(|package| {
            !ScopedFile::provided(&package.path, std::slice::from_ref(file)).is_empty()
        });
        if !found {
            bail!("unknown file `{file}` — not found in any package");
        }
    }

    Ok(spex
        .packages
        .iter()
        .filter(|package| {
            names
                .iter()
                .any(|name| package.matches_any_package_name(name))
                || !ScopedFile::provided(&package.path, files).is_empty()
        })
        .collect())
}
