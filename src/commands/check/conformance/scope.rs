use crate::config::Repository;
use crate::spec::Spec;

use super::Finding;

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
                spec.id.as_str(),
                "scope",
                Some(spec.location()),
                format!(
                    "`scope` references `{short_name}`, which does not match any repository_short_name in spex.yaml"
                ),
            )
        })
        .collect()
}
