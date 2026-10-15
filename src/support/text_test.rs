#[cfg(test)]
mod tests {
    use super::super::{level_two_sections, wrap_text};

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

    #[test]
    fn level_two_sections_start_at_each_level_two_heading() {
        assert_eq!(
            level_two_sections("# Title\n\nIntro.\n\n## One\n\nBody.\n\n### Sub\n\n## Two\n"),
            vec!["## One\n\nBody.\n\n### Sub\n\n", "## Two\n"]
        );
    }

    #[test]
    fn level_two_sections_of_text_without_one_is_empty() {
        assert!(level_two_sections("# Title\n\nIntro.\n").is_empty());
    }
}
