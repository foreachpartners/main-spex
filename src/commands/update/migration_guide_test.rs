use super::{MIGRATION_GUIDE, migration_guide_since};

const GUIDE: &str = "\
# Migration guide

Intro that belongs to no version.

## 0.21.0

Third steps.

## 0.20.0

Second steps.

## 0.9.0

First steps.
";

#[test]
fn returns_sections_after_from_version_newest_first() {
    let sections = migration_guide_since(GUIDE, "0.9.0").unwrap();
    assert_eq!(
        sections,
        vec!["## 0.21.0\n\nThird steps.", "## 0.20.0\n\nSecond steps."]
    );
}

#[test]
fn compares_versions_numerically() {
    let sections = migration_guide_since(GUIDE, "0.10.0").unwrap();
    assert_eq!(sections.len(), 2);
}

#[test]
fn returns_sections_after_a_from_version_the_guide_does_not_list() {
    let sections = migration_guide_since(GUIDE, "0.20.5").unwrap();
    assert_eq!(sections, vec!["## 0.21.0\n\nThird steps."]);
}

#[test]
fn returns_nothing_when_from_version_is_latest() {
    assert!(migration_guide_since(GUIDE, "0.21.0").unwrap().is_empty());
}

#[test]
fn reads_a_short_from_version_as_zero_padded() {
    assert_eq!(
        migration_guide_since(GUIDE, "0.20").unwrap(),
        migration_guide_since(GUIDE, "0.20.0").unwrap()
    );
    assert_eq!(
        migration_guide_since(GUIDE, "0").unwrap(),
        migration_guide_since(GUIDE, "0.0.0").unwrap()
    );
    assert!(migration_guide_since(GUIDE, "1").unwrap().is_empty());
}

#[test]
fn rejects_a_from_version_that_is_not_major_minor_or_patch() {
    for invalid in [
        "",
        "0.",
        "0..1",
        "0.21.0.1",
        "v0.21.0",
        "0.21.x",
        "0.21.0-beta",
    ] {
        assert!(
            migration_guide_since(GUIDE, invalid).is_err(),
            "{invalid:?} should be rejected"
        );
    }
}

#[test]
fn every_section_of_the_bundled_guide_is_headed_by_a_version() {
    let headings = MIGRATION_GUIDE
        .lines()
        .filter(|line| line.starts_with("## "))
        .count();
    assert!(headings > 0);
    assert_eq!(
        migration_guide_since(MIGRATION_GUIDE, "0.0.0")
            .unwrap()
            .len(),
        headings
    );
}
