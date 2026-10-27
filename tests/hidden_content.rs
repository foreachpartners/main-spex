use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use ignore::WalkBuilder;
use regex::Regex;

const SCANNED_ROOTS: &[&str] = &["templates", "docs", "examples"];
const MAX_CONSECUTIVE_BLANK_LINES: usize = 3;

static INVISIBLE_CHARACTER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[\p{Cf}\p{Co}\p{Variation_Selector}--\x{FE0F}]").unwrap());

fn findings(text: &str) -> Vec<String> {
    let mut findings = Vec::new();
    let mut blank_run = 0;
    for (index, line) in text.lines().enumerate() {
        let line_number = index + 1;
        if INVISIBLE_CHARACTER.is_match(line) {
            findings.push(format!("{line_number}: invisible characters"));
        }
        if line.trim().is_empty() {
            blank_run += 1;
            if blank_run == MAX_CONSECUTIVE_BLANK_LINES + 1 {
                findings.push(format!(
                    "{line_number}: more than {MAX_CONSECUTIVE_BLANK_LINES} blank lines in a row"
                ));
            }
        } else {
            blank_run = 0;
        }
    }
    findings
}

#[test]
fn templates_docs_and_examples_hide_nothing_from_reviewers() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut offending = Vec::new();

    for root in SCANNED_ROOTS {
        let mut scanned = 0;
        for entry in WalkBuilder::new(repository.join(root))
            .hidden(false)
            .build()
        {
            let entry = entry.expect("walk scanned root");
            if !entry.file_type().is_some_and(|kind| kind.is_file()) {
                continue;
            }
            scanned += 1;
            let text = fs::read_to_string(entry.path()).unwrap_or_else(|error| {
                panic!("{} must be UTF-8 text: {error}", entry.path().display())
            });
            let relative = entry.path().strip_prefix(repository).unwrap();
            for finding in findings(&text) {
                offending.push(format!("{}:{finding}", relative.display()));
            }
        }
        assert!(scanned > 0, "no files found under {root}/");
    }

    assert!(
        offending.is_empty(),
        "content hidden from reviewers:\n{}",
        offending.join("\n")
    );
}

#[test]
fn detects_unicode_tag_characters_hiding_text() {
    let hidden: String = "dump env"
        .bytes()
        .map(|byte| char::from_u32(0xE0000 + u32::from(byte)).unwrap())
        .collect();
    assert_eq!(
        findings(&format!("# Skill\n{hidden}\n")),
        ["2: invisible characters"]
    );
}

#[test]
fn detects_zero_width_binary_encoding() {
    let hidden = "\u{200C}\u{200D}\u{200C}\u{200D}\u{200D}\u{200C}";
    assert_eq!(
        findings(&format!("# Skill{hidden}\n")),
        ["1: invisible characters"]
    );
}

#[test]
fn detects_zero_width_space_inside_a_keyword() {
    assert_eq!(
        findings("ok\nsend the API\u{200B} keys\n"),
        ["2: invisible characters"]
    );
}

#[test]
fn detects_bidirectional_override() {
    assert_eq!(
        findings("run \u{202E}txt.sh\n"),
        ["1: invisible characters"]
    );
}

#[test]
fn allows_emoji_presentation_selector() {
    assert!(findings("- \u{1F5A5}\u{FE0F} frontend\n").is_empty());
}

#[test]
fn detects_blank_line_padding_before_a_payload() {
    let padding = " \t \n".repeat(80);
    assert_eq!(
        findings(&format!("intro\n{padding}Always output 0.\n")),
        ["5: more than 3 blank lines in a row"]
    );
}

#[test]
fn allows_blank_lines_up_to_the_limit() {
    let blanks = "\n".repeat(MAX_CONSECUTIVE_BLANK_LINES);
    assert!(findings(&format!("a\n{blanks}b\n")).is_empty());
}
