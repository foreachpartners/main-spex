use std::path::Path;

use crate::config::{Package, SPEX_CONFIG_FILE};
use crate::support::fs::relative_to;

use super::super::finding::{Finding, Location};
use super::conformance_error;

#[cfg(test)]
#[path = "declared_files_test.rs"]
mod declared_files_test;

const DECLARED_FILES_CHECK: &str = "declared-files-exist";

pub(super) fn validate_declared_files(
    spex_dir: &Path,
    prd: &Path,
    packages: &[Package],
) -> Vec<Finding> {
    let shown = |path: &Path| relative_to(path, spex_dir).display().to_string();

    let missing_prd = (!prd.is_file())
        .then(|| format!("`prd` file `{}` does not exist", shown(prd)))
        .into_iter();
    let missing_contracts = packages.iter().flat_map(|package| {
        package
            .contracts
            .iter()
            .filter(|contract| !contract.is_file())
            .map(|contract| {
                format!(
                    "`contracts` file `{}` of package `{}` does not exist",
                    shown(contract),
                    package.package_name
                )
            })
    });

    missing_prd
        .chain(missing_contracts)
        .map(|message| {
            conformance_error(
                DECLARED_FILES_CHECK,
                Some(Location::in_file(spex_dir.join(SPEX_CONFIG_FILE))),
                message,
            )
        })
        .collect()
}
