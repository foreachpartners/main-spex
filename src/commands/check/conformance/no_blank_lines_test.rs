#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::commands::check::conformance::conformance::RULE_ID;
    use tempfile::tempdir;

    #[test]
    fn blank_lines_passes_a_file_without_blank_lines() {
        let text = indoc::indoc! {r#"
            id: tidy
            title: Tidy spec
            acceptance_criteria:
              - id: first
                must: "do the thing"
        "#};

        assert!(blank_lines_between_keys(text).is_empty());
    }

    #[test]
    fn blank_lines_flags_a_blank_line_between_top_level_keys() {
        let text = "id: gappy\n\ntitle: Gappy spec\n";

        assert_eq!(blank_lines_between_keys(text), vec![2]);
    }

    #[test]
    fn blank_lines_flags_a_blank_line_between_list_items() {
        let text = indoc::indoc! {r#"
            id: gappy
            acceptance_criteria:
              - id: first
                must: "do the thing"

              - id: second
                must: "do the other thing"
        "#};

        assert_eq!(blank_lines_between_keys(text), vec![5]);
    }

    #[test]
    fn blank_lines_flags_a_whitespace_only_line() {
        let text = "id: gappy\n   \ntitle: Gappy spec\n";

        assert_eq!(blank_lines_between_keys(text), vec![2]);
    }

    #[test]
    fn blank_lines_allows_a_blank_line_inside_a_block_value() {
        let text = indoc::indoc! {r#"
            id: example
            examples:
              good: |
                first paragraph

                second paragraph
              bad: |
                one

                two
        "#};

        assert!(blank_lines_between_keys(text).is_empty());
    }

    #[test]
    fn blank_lines_flags_a_blank_line_after_a_block_value() {
        let text = indoc::indoc! {r#"
            id: example
            good: |
              first paragraph

              second paragraph

            title: After the block
        "#};

        assert_eq!(blank_lines_between_keys(text), vec![6]);
    }

    #[test]
    fn blank_lines_allows_a_blank_line_inside_a_multi_line_quoted_value() {
        let text = "id: quoted\ndescription: \"first line\n\n  second line\"\ntitle: Quoted\n";

        assert!(blank_lines_between_keys(text).is_empty());
    }

    #[test]
    fn no_blank_lines_flags_a_spec_and_a_rule_at_the_blank_line() {
        let dir = tempdir().expect("temp dir");
        for (folder, name, text) in [
            (
                SPECS_DIR,
                "gappy-spec",
                "id: gappy-spec\n\ntitle: Gappy spec\n",
            ),
            (
                RULES_DIR,
                "gappy-rule",
                "id: gappy-rule\ntitle: Gappy rule\n\n",
            ),
        ] {
            let folder = dir.path().join(folder);
            std::fs::create_dir_all(&folder).expect("folder");
            std::fs::write(folder.join(format!("{name}.yaml")), text).expect("write file");
        }

        let findings = validate_no_blank_lines(dir.path()).expect("validate");

        let reported: Vec<String> = findings
            .iter()
            .map(|finding| {
                let location = finding.location.as_ref().expect("location");
                format!("{}:{}", location.file.display(), location.line.unwrap_or(0))
            })
            .collect();
        assert_eq!(reported.len(), 2, "{reported:?}");
        assert!(
            reported[0].ends_with("specs/gappy-spec.yaml:2"),
            "{reported:?}"
        );
        assert!(
            reported[1].ends_with("rules/gappy-rule.yaml:3"),
            "{reported:?}"
        );
        assert!(findings.iter().all(|finding| {
            finding.source.baseline_key() == Some((RULE_ID, NO_BLANK_LINES_CHECK))
        }));
    }
}
