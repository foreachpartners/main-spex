use super::*;

#[test]
fn hyperlink_wraps_text_in_osc8_when_enabled() {
    let style = Style { enabled: true };

    assert_eq!(
        style.hyperlink("file:///tmp/a.md", "a.md"),
        "\u{1b}]8;;file:///tmp/a.md\u{1b}\\a.md\u{1b}]8;;\u{1b}\\"
    );
}

#[test]
fn hyperlink_returns_plain_text_when_disabled() {
    let style = Style { enabled: false };

    assert_eq!(style.hyperlink("file:///tmp/a.md", "a.md"), "a.md");
}
