use super::*;

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
    let url = Hyperlink::with_url("file:///tmp/a.md");
    let link = format!("{url}{}{url:#}", "a.md".cyan());

    assert_eq!(visible_width(&link), 4);
}
