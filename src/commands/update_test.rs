use super::changelog_since;

const CHANGELOG: &str = "\
# Changelog

## [0.7.83] - 2026-09-07

- Third entry.

## [0.7.82] - 2026-09-06

- Second entry.

## [0.7.81] - 2026-09-01

- First entry.
";

#[test]
fn returns_sections_newer_than_from_version() {
    let entries = changelog_since(CHANGELOG, "0.7.82");
    assert_eq!(entries, vec!["## [0.7.83] - 2026-09-07\n\n- Third entry."]);
}

#[test]
fn returns_everything_when_from_version_is_absent() {
    let entries = changelog_since(CHANGELOG, "0.0.0");
    assert_eq!(entries.len(), 3);
}

#[test]
fn returns_nothing_when_from_version_is_latest() {
    let entries = changelog_since(CHANGELOG, "0.7.83");
    assert!(entries.is_empty());
}
