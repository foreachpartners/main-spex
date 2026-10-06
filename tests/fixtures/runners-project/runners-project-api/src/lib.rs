// SPDX-License-Identifier: MIT
// FIXME: remove the temporary cache
// HACK: skip validation for now
pub fn first_id(ids: &[u32]) -> u32 {
    ids.first().copied().unwrap()
}
