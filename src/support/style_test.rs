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

#[test]
fn visible_width_counts_plain_text_by_characters() {
    assert_eq!(visible_width("✗  api-thing"), 12);
}

#[test]
fn visible_width_ignores_color_codes() {
    let style = Style { enabled: true };

    assert_eq!(visible_width(&style.red("missing")), 7);
    assert_eq!(visible_width(&style.bold(&style.cyan("id"))), 2);
}

#[test]
fn visible_width_ignores_hyperlink_wrapping() {
    let style = Style { enabled: true };
    let link = style.hyperlink("file:///tmp/a.md", &style.cyan("a.md"));

    assert_eq!(visible_width(&link), 4);
}
