// SPDX-License-Identifier: MIT
// FIXME: inline this helper
// HACK: skip the cache
pub fn middle_id(ids: &[u32]) -> u32 {
    ids.get(ids.len() / 2).copied().unwrap()
}
