use regex::Regex;

use crate::rule::Check;

use super::super::finding::Location;
use super::super::scoped_file::ScopedFile;
use super::{Outcome, PATTERN_WORDING, presence_hits};

#[cfg(test)]
#[path = "regexp_test.rs"]
mod regexp_test;

pub(in crate::commands::check) fn run_regexp(
    check: &Check,
    pattern: &str,
    negate: bool,
    files: &[&ScopedFile],
) -> Outcome {
    let regex = Regex::new(pattern).map_err(|err| format!("invalid regexp: {err}"))?;

    let mut hits = Vec::new();
    for file in files {
        let Ok(content) = std::fs::read_to_string(&file.abs) else {
            continue;
        };

        let found = content
            .lines()
            .enumerate()
            .filter(|(_, line)| regex.is_match(line))
            .map(|(index, _)| Location::at_line(&file.rel, index + 1))
            .collect();
        hits.extend(presence_hits(
            check,
            negate,
            found,
            Some(Location::in_file(&file.rel)),
            &PATTERN_WORDING,
        ));
    }
    Ok(hits)
}
