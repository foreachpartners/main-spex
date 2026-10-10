#[cfg(test)]
mod tests {
    use super::super::wrap_text;

    #[test]
    fn wrap_text_breaks_lines_at_the_width() {
        assert_eq!(
            wrap_text("one two three four", 9),
            vec!["one two", "three", "four"]
        );
    }

    #[test]
    fn wrap_text_keeps_an_overlong_word_on_its_own_line() {
        assert_eq!(
            wrap_text("a verylongword b", 4),
            vec!["a", "verylongword", "b"]
        );
    }

    #[test]
    fn wrap_text_of_blank_text_is_empty() {
        assert!(wrap_text("   ", 10).is_empty());
    }
}
