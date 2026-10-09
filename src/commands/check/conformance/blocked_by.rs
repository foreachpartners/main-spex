use crate::spec::{Spec, SpecId, Status};
use std::collections::HashMap;

use super::super::finding::{Finding, Location, Source};

#[cfg(test)]
#[path = "blocked_by_test.rs"]
mod blocked_by_test;

pub(super) fn status_by_id(specs: &[Spec]) -> HashMap<&SpecId, Status> {
    specs.iter().map(|spec| (&spec.id, spec.status)).collect()
}

pub(super) fn validate_blocked_by(
    spec: &Spec,
    status_by_id: &HashMap<&SpecId, Status>,
) -> Vec<Finding> {
    spec.blocked_by
        .iter()
        .filter_map(|blocker| {
            let location = Some(Location::in_file(&spec.path));
            match status_by_id.get(blocker) {
                None => Some(Finding::error(
                    Source::spec(spec.id.as_str(), "blocked-by"),
                    location,
                    format!("`blocked_by` references `{blocker}`, a non-existing spec"),
                )),
                Some(Status::Active) => Some(Finding::warn(
                    Source::spec(spec.id.as_str(), "blocked-by"),
                    location,
                    format!(
                        "`blocked_by` references `{blocker}`, which is already active — remove satisfied blockers"
                    ),
                )),
                Some(_) => None,
            }
        })
        .collect()
}
