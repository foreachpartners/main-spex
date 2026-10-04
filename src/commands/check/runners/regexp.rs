use regex::Regex;

use crate::rule::Check;

use super::super::ScopedFile;
use super::{Hit, Outcome};

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

        if negate {
            if !content.lines().any(|line| regex.is_match(line)) {
                hits.push(Hit::new(
                    Some(file.rel_str()),
                    check.message("required pattern not found"),
                ));
            }
        } else {
            for (index, line) in content.lines().enumerate() {
                if regex.is_match(line) {
                    hits.push(Hit::new(
                        Some(format!("{}:{}", file.rel_str(), index + 1)),
                        check.message("matches forbidden pattern"),
                    ));
                }
            }
        }
    }
    Ok(hits)
}
