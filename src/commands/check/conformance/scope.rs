use crate::config::Repository;
use crate::spec::Spec;

use super::super::finding::{Finding, Location, Source};

#[cfg(test)]
#[path = "scope_test.rs"]
mod scope_test;

pub(super) fn validate_scope(spec: &Spec, repositories: &[Repository]) -> Vec<Finding> {
    spec.scope
        .iter()
        .filter(|short_name| {
            !repositories
                .iter()
                .any(|repo| &repo.repository_short_name == *short_name)
        })
        .map(|short_name| {
            Finding::error(
                Source::spec(spec.id.as_str(), "scope"),
                Some(Location::in_file(&spec.path)),
                format!(
                    "`scope` references `{short_name}`, which does not match any repository_short_name in spex.yaml"
                ),
            )
        })
        .collect()
}
