use crate::spec::{Spec, Status};

use super::super::finding::{Finding, Location, Source};

#[cfg(test)]
#[path = "status_test.rs"]
mod status_test;

pub(super) fn status_finding(spec: &Spec) -> Option<Finding> {
    let (done, total) = spec.ac_done_total();
    if total == 0 || done != total || spec.status.is_implemented() {
        return None;
    }
    Some(Finding::warn(
        Source::spec(spec.id.as_str(), "status"),
        Some(Location::in_file(&spec.path)),
        format!(
            "every acceptance criterion is done but status is `{}` — set status to active or deprecated",
            spec.status.label()
        ),
    ))
}

pub(super) fn active_incomplete_finding(spec: &Spec) -> Option<Finding> {
    let (done, total) = spec.ac_done_total();
    if spec.status != Status::Active || done == total {
        return None;
    }
    Some(Finding::error(
        Source::spec(spec.id.as_str(), "status"),
        Some(Location::in_file(&spec.path)),
        format!(
            "status is `active` but only {done}/{total} acceptance criteria are done — use `in-progress` until every `is_done: false` is cleared"
        ),
    ))
}
