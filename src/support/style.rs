use owo_colors::OwoColorize;
use supports_hyperlinks::Stream;

#[cfg(test)]
#[path = "style_test.rs"]
mod style_test;

pub fn visible_width(text: &str) -> usize {
    anstream::adapter::strip_str(text)
        .map(|part| part.chars().count())
        .sum()
}

pub fn version_line() -> String {
    format!("spex {}", env!("CARGO_PKG_VERSION"))
        .dimmed()
        .to_string()
}

pub fn hyperlink(url: &str, text: &str) -> String {
    if supports_hyperlinks::on(Stream::Stdout) {
        osc8_link(url, text)
    } else {
        text.to_string()
    }
}

fn osc8_link(url: &str, text: &str) -> String {
    format!("\u{1b}]8;;{url}\u{1b}\\{text}\u{1b}]8;;\u{1b}\\")
}
