use std::ops::RangeInclusive;
use std::path::Path;

use ast_grep_language::{LanguageExt, SupportLang};

use crate::error::Result;
use crate::rule::RULES_DIR;
use crate::spec::SPECS_DIR;
use crate::support::yaml_dir::yaml_files;

use super::super::finding::{Finding, Location};
use super::conformance_error;

#[cfg(test)]
#[path = "no_blank_lines_test.rs"]
mod no_blank_lines_test;

pub(super) const NO_BLANK_LINES_CHECK: &str = "no-blank-lines";

const SCALAR_KINDS: [&str; 4] = [
    "block_scalar",
    "double_quote_scalar",
    "single_quote_scalar",
    "plain_scalar",
];

pub(super) fn validate_no_blank_lines(spex_dir: &Path) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();
    for dir in [SPECS_DIR, RULES_DIR] {
        for file in yaml_files(&spex_dir.join(dir))? {
            let Ok(text) = std::fs::read_to_string(&file) else {
                continue;
            };
            findings.extend(blank_lines_between_keys(&text).into_iter().map(|line| {
                conformance_error(
                    NO_BLANK_LINES_CHECK,
                    Some(Location::at_line(&file, line)),
                    "blank line between keys: remove it, because a blank line separates one spec or rule from the next in the output of `spex specs`, `spex rules` and `spex lib`".to_string(),
                )
            }));
        }
    }
    Ok(findings)
}

fn blank_lines_between_keys(text: &str) -> Vec<usize> {
    let inside_values = multi_line_value_lines(text);
    text.lines()
        .enumerate()
        .filter(|(index, line)| {
            line.trim().is_empty() && !inside_values.iter().any(|lines| lines.contains(index))
        })
        .map(|(index, _)| index + 1)
        .collect()
}

fn multi_line_value_lines(text: &str) -> Vec<RangeInclusive<usize>> {
    let tree = SupportLang::Yaml.ast_grep(text);
    tree.root()
        .dfs()
        .filter(|node| SCALAR_KINDS.contains(&node.kind().as_ref()))
        .map(|node| {
            let first = node.start_pos().line();
            let last = first + node.text().trim_end().lines().count().saturating_sub(1);
            first..=last
        })
        .collect()
}
