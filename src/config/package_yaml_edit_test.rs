#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::config::PreexistingError;
    use indoc::indoc;

    fn entry(rule_id: &str, check: &str, files: &[&str]) -> PreexistingError {
        PreexistingError {
            rule_id: rule_id.to_string(),
            check: check.to_string(),
            files: files.iter().map(|file| file.to_string()).collect(),
        }
    }

    #[test]
    fn set_check_ignore_creates_the_block_in_an_empty_file() {
        let out = set_check_ignore("", &[entry("X", "no-cyrillic", &["README.md"])]);

        assert_eq!(
            out,
            indoc! {"
                check_ignore:
                  - rule_id: X
                    check: no-cyrillic
                    files:
                      - README.md
            "}
        );
    }

    #[test]
    fn set_check_ignore_omits_files_for_a_whole_check_entry() {
        let out = set_check_ignore("", &[entry("X", "no-git-history-slop", &[])]);

        assert_eq!(
            out,
            indoc! {"
                check_ignore:
                  - rule_id: X
                    check: no-git-history-slop
            "}
        );
    }

    #[test]
    fn set_check_ignore_inserts_the_block_before_coverage_ignore() {
        let raw = indoc! {"
            coverage_ignore:
              - spec_id: place-order
                layer: impl
        "};

        let out = set_check_ignore(raw, &[entry("X", "no-cyrillic", &["README.md"])]);

        assert_eq!(
            out,
            indoc! {"
                check_ignore:
                  - rule_id: X
                    check: no-cyrillic
                    files:
                      - README.md
                coverage_ignore:
                  - spec_id: place-order
                    layer: impl
            "}
        );
    }

    #[test]
    fn set_check_ignore_replaces_an_existing_block_in_place_and_keeps_the_rest() {
        let raw = indoc! {"
            # owned by the api team
            check_ignore:
              - rule_id: X
                check: old-check
                files: [old.md]
              - rule_id: X
                check: other-old-check

            coverage_ignore:
              - spec_id: place-order
                layer: impl
        "};

        let out = set_check_ignore(raw, &[entry("X", "no-cyrillic", &["README.md"])]);

        assert_eq!(
            out,
            indoc! {"
                # owned by the api team
                check_ignore:
                  - rule_id: X
                    check: no-cyrillic
                    files:
                      - README.md

                coverage_ignore:
                  - spec_id: place-order
                    layer: impl
            "}
        );
    }

    #[test]
    fn set_check_ignore_replaces_a_block_whose_sequence_is_not_indented() {
        let raw = indoc! {"
            check_ignore:
            - rule_id: X
              check: old-check

              files: [old.md]
            coverage_ignore: []
        "};

        let out = set_check_ignore(raw, &[entry("X", "no-cyrillic", &[])]);

        assert_eq!(
            out,
            indoc! {"
                check_ignore:
                  - rule_id: X
                    check: no-cyrillic
                coverage_ignore: []
            "}
        );
    }

    #[test]
    fn set_check_ignore_drops_the_block_when_no_errors_are_left() {
        let raw = indoc! {"
            check_ignore:
              - rule_id: X
                check: old-check
            coverage_ignore:
              - spec_id: place-order
                layer: impl
        "};

        let out = set_check_ignore(raw, &[]);

        assert_eq!(
            out,
            indoc! {"
                coverage_ignore:
                  - spec_id: place-order
                    layer: impl
            "}
        );
    }

    #[test]
    fn set_check_ignore_empties_a_file_that_held_only_the_block() {
        let raw = indoc! {"
            check_ignore:
              - rule_id: X
                check: old-check
        "};

        assert_eq!(set_check_ignore(raw, &[]), "");
    }

    #[test]
    fn set_check_ignore_returns_the_file_as_is_when_there_is_nothing_to_write() {
        for raw in ["", "coverage_ignore: []", "coverage_ignore: []\n"] {
            assert_eq!(set_check_ignore(raw, &[]), raw);
        }
    }

    #[test]
    fn set_check_ignore_is_stable_when_run_again_on_its_own_output() {
        let entries = [entry("X", "no-cyrillic", &["README.md", "docs/intro.md"])];
        let once = set_check_ignore("coverage_ignore: []\n", &entries);

        assert_eq!(set_check_ignore(&once, &entries), once);
    }
}
