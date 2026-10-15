use std::cmp::Reverse;
use std::process::ExitCode;

use anstream::println;
use eros::bail;

use crate::error::Result;
use crate::support::text::level_two_sections;

#[cfg(test)]
#[path = "migration_guide_test.rs"]
mod migration_guide_test;

const MIGRATION_GUIDE: &str =
    include_str!("../../../templates/skills/spex-update/references/migration_guide.md");

type Version = (u64, u64, u64);

pub fn print_since(from_version: &str) -> Result<ExitCode> {
    let sections = migration_guide_since(MIGRATION_GUIDE, from_version)?;
    if sections.is_empty() {
        println!("no migration needed");
    } else {
        println!("{}", sections.join("\n\n"));
    }
    Ok(ExitCode::SUCCESS)
}

fn migration_guide_since(guide: &str, from_version: &str) -> Result<Vec<String>> {
    let Some(from) = parse_from_version(from_version) else {
        bail!(
            "invalid version `{from_version}` — expected MAJOR, MAJOR.MINOR or MAJOR.MINOR.PATCH"
        );
    };

    let mut newer: Vec<(Version, String)> = level_two_sections(guide)
        .into_iter()
        .filter_map(|section| {
            let version = parse_heading_version(section.lines().next()?.strip_prefix("## ")?)?;
            (version > from).then(|| (version, section.trim_end().to_string()))
        })
        .collect();
    newer.sort_by_key(|(version, _)| Reverse(*version));
    Ok(newer.into_iter().map(|(_, section)| section).collect())
}

fn parse_numbers(text: &str) -> Option<Vec<u64>> {
    text.split('.').map(|part| part.parse().ok()).collect()
}

fn parse_heading_version(text: &str) -> Option<Version> {
    let [major, minor, patch] = parse_numbers(text)?[..] else {
        return None;
    };
    Some((major, minor, patch))
}

fn parse_from_version(text: &str) -> Option<Version> {
    match parse_numbers(text)?[..] {
        [major] => Some((major, 0, 0)),
        [major, minor] => Some((major, minor, 0)),
        [major, minor, patch] => Some((major, minor, patch)),
        _ => None,
    }
}
