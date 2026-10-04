use globset::Glob;

use crate::rule::Check;

use super::super::ScopedFile;
use super::{Hit, Outcome};

pub(in crate::commands::check) fn run_glob(
    check: &Check,
    pattern: &str,
    negate: bool,
    files: &[&ScopedFile],
) -> Outcome {
    let matcher = Glob::new(pattern)
        .map_err(|err| format!("invalid glob: {err}"))?
        .compile_matcher();

    let matched: Vec<&&ScopedFile> = files
        .iter()
        .filter(|file| matcher.is_match(&file.rel))
        .collect();

    if negate {
        return Ok(if matched.is_empty() {
            vec![Hit::new(
                None,
                check.message(&format!("required path not found: {pattern}")),
            )]
        } else {
            Vec::new()
        });
    }

    Ok(matched
        .into_iter()
        .map(|file| Hit::new(Some(file.rel_str()), check.message("forbidden path")))
        .collect())
}
