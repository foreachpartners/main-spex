#[cfg(test)]
mod tests {
    use super::super::*;
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;
    use tempfile::tempdir;

    use crate::support::testing;

    fn write_candidate(package_path: &std::path::Path, rel: &str, content: &str) -> ScopedFile {
        let abs = package_path.join(rel);
        if let Some(parent) = abs.parent() {
            std::fs::create_dir_all(parent).expect("parent dirs");
        }
        std::fs::write(&abs, content).expect("write candidate");
        ScopedFile {
            abs,
            rel: PathBuf::from(rel),
        }
    }

    fn known(ids: &[&str]) -> HashMap<SpecId, HashSet<AcId>> {
        ids.iter()
            .map(|id| (testing::spec_id(id), HashSet::new()))
            .collect()
    }

    fn known_with_acs(id: &str, ac_ids: &[&str]) -> HashMap<SpecId, HashSet<AcId>> {
        HashMap::from([(
            testing::spec_id(id),
            ac_ids.iter().map(|ac| testing::ac_id(ac)).collect(),
        )])
    }

    #[test]
    fn flags_malformed_annotation() {
        let dir = tempdir().expect("temp dir");
        let candidates = vec![write_candidate(
            dir.path(),
            "src/lib.rs",
            "// @spec impl list-pizza-menu\n// @spec bogus list-pizza-menu\n",
        )];
        let ctx = PackageCtx {
            package_path: dir.path(),
            spex_dir: dir.path(),
            candidates: &candidates,
        };
        let findings = validate_annotations(&ctx, &known(&["list-pizza-menu"]));

        assert_eq!(findings.len(), 1);
        assert_eq!(check_of(&findings[0]), FORMAT_CHECK);
        assert_eq!(
            findings[0].location,
            Some(Location::at_line("src/lib.rs", 2))
        );
    }

    #[test]
    fn flags_orphan_reference_and_ignores_known_ids() {
        let dir = tempdir().expect("temp dir");
        let candidates = vec![write_candidate(
            dir.path(),
            "src/lib.rs",
            "// @spec impl list-pizza-menu#in-stock-only, rate-limit\n// @spec test unknown-spec#some-ac\n",
        )];
        let ctx = PackageCtx {
            package_path: dir.path(),
            spex_dir: dir.path(),
            candidates: &candidates,
        };
        let mut known_ids = known(&["list-pizza-menu", "rate-limit"]);
        known_ids.insert(
            testing::spec_id("list-pizza-menu"),
            HashSet::from([testing::ac_id("in-stock-only")]),
        );
        let findings = validate_annotations(&ctx, &known_ids);

        assert_eq!(findings.len(), 1);
        assert_eq!(check_of(&findings[0]), ORPHAN_CHECK);
        assert!(findings[0].message.contains("unknown-spec"));
    }

    #[test]
    fn skips_markdown_and_spex_project_dirs() {
        let dir = tempdir().expect("temp dir");
        let candidates = vec![
            write_candidate(dir.path(), "docs/notes.md", "@spec bogus\n"),
            write_candidate(dir.path(), "specs/example.yaml", "@spec bogus\n"),
        ];
        let ctx = PackageCtx {
            package_path: dir.path(),
            spex_dir: dir.path(),
            candidates: &candidates,
        };
        let findings = validate_annotations(&ctx, &HashMap::new());

        assert!(findings.is_empty());
    }

    #[test]
    fn skips_the_spex_dir_nested_in_the_package_but_not_its_code() {
        let dir = tempdir().expect("temp dir");
        let candidates = vec![
            write_candidate(dir.path(), "spex/specs/example.yaml", "@spec bogus\n"),
            write_candidate(dir.path(), "spex/rules/example/check.sh", "@spec bogus\n"),
            write_candidate(dir.path(), "spex/schemas/spec.schema.json", "@spec bogus\n"),
            write_candidate(dir.path(), "src/lib.rs", "// @spec impl bogus\n"),
        ];
        let spex_dir = dir.path().join("spex");
        let ctx = PackageCtx {
            package_path: dir.path(),
            spex_dir: &spex_dir,
            candidates: &candidates,
        };
        let findings = validate_annotations(&ctx, &HashMap::new());

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].location, Some(Location::in_file("src/lib.rs")));
    }

    #[test]
    fn skips_schemas_dir_in_any_package() {
        let dir = tempdir().expect("temp dir");
        let candidates = vec![write_candidate(
            dir.path(),
            "schemas/spec.schema.json",
            "@spec bogus\n",
        )];
        let ctx = PackageCtx {
            package_path: dir.path(),
            spex_dir: dir.path(),
            candidates: &candidates,
        };
        let findings = validate_annotations(&ctx, &HashMap::new());

        assert!(findings.is_empty());
    }

    #[test]
    fn flags_unknown_ac_id_but_allows_known_ones() {
        let dir = tempdir().expect("temp dir");
        let candidates = vec![write_candidate(
            dir.path(),
            "src/lib.rs",
            "// @spec impl delivery-time#estimate-on-delivery\n// @spec impl delivery-time#xxx\n",
        )];
        let ctx = PackageCtx {
            package_path: dir.path(),
            spex_dir: dir.path(),
            candidates: &candidates,
        };
        let findings = validate_annotations(
            &ctx,
            &known_with_acs("delivery-time", &["estimate-on-delivery"]),
        );

        assert_eq!(findings.len(), 1);
        assert_eq!(check_of(&findings[0]), ORPHAN_CHECK);
        assert!(findings[0].message.contains("xxx"));
        assert!(findings[0].message.contains("delivery-time"));
    }

    #[test]
    fn does_not_truncate_an_id_with_an_uppercase_letter_into_a_known_prefix() {
        let dir = tempdir().expect("temp dir");
        let candidates = vec![write_candidate(
            dir.path(),
            "src/lib.rs",
            "// @spec impl delivery-timeX#estimate-on-delivery\n",
        )];
        let ctx = PackageCtx {
            package_path: dir.path(),
            spex_dir: dir.path(),
            candidates: &candidates,
        };
        let findings = validate_annotations(
            &ctx,
            &known_with_acs("delivery-time", &["estimate-on-delivery"]),
        );

        assert_eq!(findings.len(), 1);
        assert_eq!(check_of(&findings[0]), FORMAT_CHECK);
        assert_eq!(
            findings[0].location,
            Some(Location::at_line("src/lib.rs", 1))
        );
    }

    fn validate_content(content: &str, known_ids: &HashMap<SpecId, HashSet<AcId>>) -> Vec<Finding> {
        let dir = tempdir().expect("temp dir");
        let candidates = vec![write_candidate(dir.path(), "src/lib.rs", content)];
        let ctx = PackageCtx {
            package_path: dir.path(),
            spex_dir: dir.path(),
            candidates: &candidates,
        };
        validate_annotations(&ctx, known_ids)
    }

    fn locations(findings: &[Finding]) -> Vec<String> {
        findings
            .iter()
            .map(|finding| {
                finding
                    .location
                    .as_ref()
                    .expect("finding location")
                    .to_string()
            })
            .collect()
    }

    fn check_of(finding: &Finding) -> &str {
        finding.source.baseline_key().expect("check source").1
    }

    #[test]
    fn flags_an_annotation_without_a_known_layer_or_ids() {
        let findings = validate_content(
            "// @spec xx\n// @spec ggg valid_id\n// @spec test\n",
            &known(&["a"]),
        );

        assert!(
            findings
                .iter()
                .all(|finding| check_of(finding) == FORMAT_CHECK)
        );
        assert_eq!(
            locations(&findings),
            vec!["src/lib.rs:1", "src/lib.rs:2", "src/lib.rs:3"]
        );
    }

    #[test]
    fn flags_an_id_that_continues_past_its_valid_part_without_an_orphan_finding() {
        let findings = validate_content(
            "// @spec impl a_b\n\
             // @spec impl a, B\n\
             // @spec impl a#b#c\n\
             // @spec impl aB\n\
             // @spec impl a--b\n\
             // @spec impl a\u{430}\n",
            &known(&["a"]),
        );

        assert!(
            findings
                .iter()
                .all(|finding| check_of(finding) == FORMAT_CHECK)
        );
        assert_eq!(
            locations(&findings),
            vec![
                "src/lib.rs:1",
                "src/lib.rs:2",
                "src/lib.rs:3",
                "src/lib.rs:4",
                "src/lib.rs:5",
                "src/lib.rs:6",
            ]
        );
    }

    #[test]
    fn accepts_annotations_followed_by_a_comment_or_string_closer() {
        let findings = validate_content(
            "/* @spec impl a */\n\
             x-spec: \"@spec contract a\"\n\
             <!-- @spec impl a -->\n\
             // @spec impl a.\n\
             // @spec test a, b\n",
            &known(&["a", "b"]),
        );

        assert!(findings.is_empty());
    }
}
