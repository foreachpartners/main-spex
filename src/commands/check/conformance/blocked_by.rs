use std::collections::HashMap;

use crate::rule::Rule;
use crate::spec::{Spec, Status};

use super::Finding;

#[cfg(test)]
#[path = "blocked_by_test.rs"]
mod blocked_by_test;

pub(super) fn status_by_id<'a>(specs: &'a [Spec], rules: &'a [Rule]) -> HashMap<&'a str, Status> {
    let mut status_by_id: HashMap<&str, Status> = HashMap::new();
    for spec in specs {
        status_by_id.insert(spec.id.as_str(), spec.status);
    }
    // Rules are global and always apply, so they count as satisfied for
    // `blocked_by` purposes.
    for rule in rules {
        status_by_id.insert(rule.id.as_str(), Status::Active);
    }
    status_by_id
}

pub(super) fn validate_blocked_by(
    spec: &Spec,
    status_by_id: &HashMap<&str, Status>,
) -> Vec<Finding> {
    spec.blocked_by
        .iter()
        .filter_map(|blocker| {
            let location = Some(spec.location());
            match status_by_id.get(blocker.as_str()) {
                None => Some(Finding::error(
                    &spec.id,
                    "blocked-by",
                    location,
                    format!("`blocked_by` references `{blocker}`, a non-existing spec"),
                )),
                Some(Status::Active) => Some(Finding::warn(
                    &spec.id,
                    "blocked-by",
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
