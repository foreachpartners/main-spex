// FIXME: regenerate the bindings
// HACK: patched by hand
pub fn first_binding(bindings: &[u32]) -> u32 {
    bindings.first().copied().unwrap()
}
