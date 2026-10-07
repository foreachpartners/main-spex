use crate::spec::{Spec, Status};

use super::Finding;

#[cfg(test)]
#[path = "status_test.rs"]
mod status_test;

// A spec with every acceptance criterion marked done is implemented, so its
// status must say so. Specs with no criteria or any not-done criterion are
// exempt. This only reads `is_done`; whether that's actually true (proven by
// impl/test annotations) is `spex coverage`'s job, not `spex check`'s.
pub(super) fn status_finding(spec: &Spec) -> Option<Finding> {
    let (done, total) = spec.ac_done_total();
    if total == 0 || done != total || spec.status.is_implemented() {
        return None;
    }
    Some(Finding::warn(
        spec.id.as_str(),
        "status",
        Some(spec.location()),
        format!(
            "every acceptance criterion is done but status is `{}` — set status to active or deprecated",
            spec.status.label()
        ),
    ))
}

// `active` means fully implemented, so it requires every acceptance
// criterion to be marked done (`is_done` absent or `true`); a spec still
// being worked on belongs in `in-progress` until then.
pub(super) fn active_incomplete_finding(spec: &Spec) -> Option<Finding> {
    let (done, total) = spec.ac_done_total();
    if spec.status != Status::Active || done == total {
        return None;
    }
    Some(Finding::error(
        spec.id.as_str(),
        "status",
        Some(spec.location()),
        format!(
            "status is `active` but only {done}/{total} acceptance criteria are done — use `in-progress` until every `is_done: false` is cleared"
        ),
    ))
}
