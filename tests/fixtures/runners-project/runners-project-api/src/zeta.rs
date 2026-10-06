// FIXME: drop the legacy path
// HACK: fixed ordering assumption
pub fn last_id(ids: &[u32]) -> u32 {
    ids.last().copied().unwrap()
}
