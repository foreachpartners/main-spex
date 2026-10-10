use globset::Glob;

use crate::rule::Check;

use super::super::finding::Location;
use super::super::scoped_file::ScopedFile;
use super::{Outcome, Wording, presence_hits};

#[cfg(test)]
#[path = "glob_test.rs"]
mod glob_test;

pub(in crate::commands::check) fn run_glob(
    check: &Check,
    pattern: &str,
    negate: bool,
    files: &[&ScopedFile],
) -> Outcome {
    let matcher = Glob::new(pattern)
        .map_err(|err| format!("invalid glob: {err}"))?
        .compile_matcher();

    let found = files
        .iter()
        .filter(|file| matcher.is_match(&file.rel))
        .map(|file| Location::in_file(&file.rel))
        .collect();
    let required = format!("required path not found: {pattern}");
    let wording = Wording {
        forbidden: "forbidden path",
        required: &required,
    };
    Ok(presence_hits(check, negate, found, None, &wording))
}
