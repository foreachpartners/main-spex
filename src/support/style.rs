use anstyle_hyperlink::Hyperlink;
use owo_colors::{OwoColorize, Style};
use supports_hyperlinks::Stream;

#[cfg(test)]
#[path = "style_test.rs"]
mod style_test;

pub const DIM: Style = Style::new().dimmed();
pub const RED: Style = Style::new().red();

pub fn terminal_width() -> usize {
    terminal_size::terminal_size().map_or(100, |(terminal_size::Width(width), _)| width as usize)
}

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
    let link = if supports_hyperlinks::on(Stream::Stdout) {
        Hyperlink::with_url(url)
    } else {
        Hyperlink::default()
    };
    format!("{link}{text}{link:#}")
}
