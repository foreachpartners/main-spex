use globset::{Glob, GlobSet, GlobSetBuilder};

#[cfg(test)]
#[path = "glob_test.rs"]
mod glob_test;

pub fn build_globset(patterns: &[String]) -> Result<Option<GlobSet>, globset::Error> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    Ok(Some(builder.build()?))
}
