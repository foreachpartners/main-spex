use crate::config::Package;
use crate::spec::Spec;

use super::super::finding::{Finding, Location, Source};

#[cfg(test)]
#[path = "packages_test.rs"]
mod packages_test;

pub(super) fn validate_packages(spec: &Spec, packages: &[Package]) -> Vec<Finding> {
    spec.packages
        .iter()
        .filter(|short_name| {
            !packages
                .iter()
                .any(|package| &package.package_short_name == *short_name)
        })
        .map(|short_name| {
            Finding::error(
                Source::spec(spec.id.as_str(), "packages"),
                Some(Location::in_file(&spec.path)),
                format!(
                    "`packages` references `{short_name}`, which does not match any package_short_name in spex.yaml"
                ),
            )
        })
        .collect()
}
