use std::fs;
use std::path::Path;

use regex::Regex;

fn cargo_version() -> String {
    let manifest = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("failed to read Cargo.toml");
    let re = Regex::new(r#"(?m)^version = "([^"]+)""#).unwrap();
    re.captures(&manifest)
        .expect("Cargo.toml missing version field")
        .get(1)
        .unwrap()
        .as_str()
        .to_string()
}

fn parse_semver(version: &str) -> (u64, u64, u64) {
    let parts: Vec<u64> = version
        .split('.')
        .map(|part| part.parse().unwrap_or_else(|_| panic!("invalid version segment: {version}")))
        .collect();
    assert_eq!(parts.len(), 3, "version must be MAJOR.MINOR.PATCH: {version}");
    (parts[0], parts[1], parts[2])
}

#[test]
fn changelog_starts_with_expected_header() {
    let changelog = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("CHANGELOG.md"))
        .expect("failed to read CHANGELOG.md");
    let first_line = changelog.lines().next().unwrap_or_default();
    assert_eq!(first_line, "# Changelog");
    assert!(
        changelog.contains("keepachangelog.com"),
        "CHANGELOG.md should reference the Keep a Changelog convention"
    );
}

#[test]
fn changelog_entries_are_well_formed_and_descending() {
    let changelog = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("CHANGELOG.md"))
        .expect("failed to read CHANGELOG.md");
    let header_re = Regex::new(r"^## \[(\d+\.\d+\.\d+)\] - (\d{4}-\d{2}-\d{2})$").unwrap();

    let mut entries = Vec::new();
    for line in changelog.lines() {
        if let Some(captures) = header_re.captures(line) {
            entries.push((captures[1].to_string(), captures[2].to_string()));
        } else if line.starts_with("## ") {
            panic!("malformed changelog header, expected `## [X.Y.Z] - YYYY-MM-DD`: {line}");
        }
    }

    assert!(!entries.is_empty(), "CHANGELOG.md has no version entries");

    for window in entries.windows(2) {
        let (prev_version, prev_date) = &window[0];
        let (next_version, next_date) = &window[1];
        assert!(
            parse_semver(prev_version) > parse_semver(next_version),
            "changelog versions must be in descending order: {prev_version} is not after {next_version}"
        );
        assert!(
            prev_date >= next_date,
            "changelog dates must be in descending order: {prev_date} is not after {next_date}"
        );
    }

    let (latest_version, _) = &entries[0];
    assert_eq!(
        *latest_version,
        cargo_version(),
        "latest CHANGELOG.md entry must match the `version` in Cargo.toml"
    );
}

#[test]
fn changelog_entries_each_have_at_least_one_bullet() {
    let changelog = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("CHANGELOG.md"))
        .expect("failed to read CHANGELOG.md");
    let header_re = Regex::new(r"^## \[\d+\.\d+\.\d+\] - \d{4}-\d{2}-\d{2}$").unwrap();

    let mut current_header: Option<String> = None;
    let mut has_bullet = false;
    for line in changelog.lines() {
        if header_re.is_match(line) {
            if let Some(header) = &current_header {
                assert!(has_bullet, "changelog entry has no bullet points: {header}");
            }
            current_header = Some(line.to_string());
            has_bullet = false;
        } else if line.trim_start().starts_with("- ") {
            has_bullet = true;
        }
    }
    if let Some(header) = &current_header {
        assert!(has_bullet, "changelog entry has no bullet points: {header}");
    }
}
