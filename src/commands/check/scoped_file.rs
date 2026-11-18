use std::collections::HashSet;
use std::path::{Path, PathBuf};

use eros::Context;

use crate::config::Package;
use crate::error::Result;
use crate::support::fs::normalize;
use crate::support::package_files::{Exclude, PackageFiles};

#[cfg(test)]
#[path = "scoped_file_test.rs"]
mod scoped_file_test;

pub(super) struct ScopedFile {
    pub(super) abs: PathBuf,
    pub(super) rel: PathBuf,
}

impl ScopedFile {
    pub(super) fn collect(package: &Package) -> Result<Vec<Self>> {
        let exclude = package.compile_exclude()?;
        let mut files = Vec::new();
        for result in PackageFiles::new(&package.path, &exclude) {
            let abs = result.context("could not walk package files")?;
            files.push(Self::from_absolute(&package.path, abs));
        }
        let mut seen: HashSet<_> = files.iter().map(|file| file.abs.clone()).collect();
        let mut declared: Vec<_> = package
            .contracts
            .iter()
            .filter(|path| path.is_file())
            .filter(|path| seen.insert((*path).clone()))
            .map(|path| Self::from_absolute(&package.path, path.clone()))
            .filter(|file| !excluded(&exclude, &file.rel))
            .collect();
        declared.sort_by(|left, right| left.rel.cmp(&right.rel));
        files.extend(declared);
        Ok(files)
    }

    pub(super) fn provided_not_excluded(
        package: &Package,
        spex_dir: &Path,
        files: &[String],
    ) -> Result<Vec<Self>> {
        let exclude = package.compile_exclude()?;
        Ok(Self::provided_for_package(package, spex_dir, files)
            .into_iter()
            .filter(|file| !excluded(&exclude, &file.rel))
            .collect())
    }

    pub(super) fn provided_for_package(
        package: &Package,
        spex_dir: &Path,
        files: &[String],
    ) -> Vec<Self> {
        let base = normalize(&package.path);
        let mut selected = Self::provided(&base, files);
        let mut seen: HashSet<_> = selected.iter().map(|file| file.abs.clone()).collect();
        for path in &package.contracts {
            if path.is_file()
                && files.iter().any(|file| {
                    normalize(&base.join(file)) == *path || normalize(&spex_dir.join(file)) == *path
                })
                && seen.insert(path.clone())
            {
                selected.push(Self::from_absolute(&base, path.clone()));
            }
        }
        selected
    }

    fn provided(package_path: &Path, files: &[String]) -> Vec<Self> {
        let mut seen = HashSet::new();
        files
            .iter()
            .map(|file| normalize(&package_path.join(file)))
            .filter(|abs| seen.insert(abs.clone()))
            .filter_map(|abs| {
                (abs.starts_with(package_path) && abs.is_file())
                    .then(|| Self::from_absolute(package_path, abs))
            })
            .collect()
    }

    fn from_absolute(base: &Path, abs: PathBuf) -> Self {
        let rel = package_relative_path(base, &abs);
        Self { abs, rel }
    }
}

fn package_relative_path(base: &Path, path: &Path) -> PathBuf {
    let base = normalize(base);
    for (parents, ancestor) in base.ancestors().enumerate() {
        if let Ok(suffix) = path.strip_prefix(ancestor) {
            let mut relative = PathBuf::new();
            for _ in 0..parents {
                relative.push("..");
            }
            relative.push(suffix);
            return relative;
        }
    }
    path.to_path_buf()
}

fn excluded(exclude: &Exclude, path: &Path) -> bool {
    path.ancestors()
        .take_while(|candidate| !candidate.as_os_str().is_empty())
        .any(|candidate| exclude.matches(candidate))
}
