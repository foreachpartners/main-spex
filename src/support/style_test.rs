use super::*;

#[test]
fn osc8_link_wraps_text_in_a_terminal_hyperlink() {
    assert_eq!(
        osc8_link("file:///tmp/a.md", "a.md"),
        "\u{1b}]8;;file:///tmp/a.md\u{1b}\\a.md\u{1b}]8;;\u{1b}\\"
    );
}

#[test]
fn visible_width_counts_plain_text_by_characters() {
    assert_eq!(visible_width("✗  api-thing"), 12);
}

#[test]
fn visible_width_ignores_color_codes() {
    assert_eq!(visible_width(&"missing".red().to_string()), 7);
    assert_eq!(visible_width(&"id".cyan().bold().to_string()), 2);
}

#[test]
fn visible_width_ignores_hyperlink_wrapping() {
    let link = osc8_link("file:///tmp/a.md", &"a.md".cyan().to_string());

    assert_eq!(visible_width(&link), 4);
}
