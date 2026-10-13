#[cfg(test)]
mod tests {
    use super::super::*;

    const TEMPLATE: &str = "managed\n\n---\n\n# Project\n\nnotes for the project\n";

    fn text(bytes: Vec<u8>) -> String {
        String::from_utf8(bytes).expect("utf8")
    }

    #[test]
    fn replace_managed_section_keeps_everything_from_the_local_marker() {
        let local = "stale\n\n---\n\n# Project\n\nmine\n";

        let updated = text(replace_managed_section(TEMPLATE.as_bytes(), local));

        assert_eq!(updated, "managed\n\n---\n\n# Project\n\nmine\n");
    }

    #[test]
    fn replace_managed_section_overwrites_a_local_file_without_the_marker() {
        let updated = text(replace_managed_section(TEMPLATE.as_bytes(), "no marker"));

        assert_eq!(updated, TEMPLATE);
    }

    #[test]
    fn keep_as_project_section_puts_existing_content_below_the_marker() {
        let kept = text(keep_as_project_section(
            TEMPLATE.as_bytes(),
            "# My package\n\nrules\n",
        ));

        assert_eq!(
            kept,
            "managed\n\n---\n\n# Project\n\nnotes for the project\n\n# My package\n\nrules\n"
        );
    }

    #[test]
    fn keep_as_project_section_syncs_content_that_already_has_the_marker() {
        let existing = "stale\n\n---\n\n# Project\n\nmine\n";

        let kept = text(keep_as_project_section(TEMPLATE.as_bytes(), existing));

        assert_eq!(kept, "managed\n\n---\n\n# Project\n\nmine\n");
    }
}
