#[cfg(test)]
mod tests {
    use super::super::*;
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn write_candidate(repo_path: &std::path::Path, rel: &str, content: &str) -> ScopedFile {
        let abs = repo_path.join(rel);
        if let Some(parent) = abs.parent() {
            std::fs::create_dir_all(parent).expect("parent dirs");
        }
        std::fs::write(&abs, content).expect("write candidate");
        ScopedFile {
            abs,
            rel: PathBuf::from(rel),
        }
    }

    fn known(ids: &[&str]) -> HashMap<String, HashSet<String>> {
        ids.iter()
            .map(|id| (id.to_string(), HashSet::new()))
            .collect()
    }

    fn known_with_acs(id: &str, ac_ids: &[&str]) -> HashMap<String, HashSet<String>> {
        HashMap::from([(
            id.to_string(),
            ac_ids.iter().map(|ac| ac.to_string()).collect(),
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
        let ctx = RepoCtx {
            repo_path: dir.path(),
            spex_dir: dir.path(),
            candidates: &candidates,
        };
        let findings = validate_annotations(&ctx, false, &known(&["list-pizza-menu"]));

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].check, FORMAT_CHECK);
        assert_eq!(findings[0].location.as_deref(), Some("src/lib.rs:2"));
    }

    #[test]
    fn flags_orphan_reference_and_ignores_known_ids() {
        let dir = tempdir().expect("temp dir");
        let candidates = vec![write_candidate(
            dir.path(),
            "src/lib.rs",
            "// @spec impl list-pizza-menu#in-stock-only, rate-limit\n// @spec test unknown-spec#some-ac\n",
        )];
        let ctx = RepoCtx {
            repo_path: dir.path(),
            spex_dir: dir.path(),
            candidates: &candidates,
        };
        let mut known_ids = known(&["list-pizza-menu", "rate-limit"]);
        known_ids.insert(
            "list-pizza-menu".to_string(),
            HashSet::from(["in-stock-only".to_string()]),
        );
        let findings = validate_annotations(&ctx, false, &known_ids);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].check, ORPHAN_CHECK);
        assert!(findings[0].message.contains("unknown-spec"));
    }

    #[test]
    fn skips_markdown_and_spex_workspace_dirs() {
        let dir = tempdir().expect("temp dir");
        let candidates = vec![
            write_candidate(dir.path(), "docs/notes.md", "@spec bogus\n"),
            write_candidate(dir.path(), "specs/example.yaml", "@spec bogus\n"),
        ];
        let ctx = RepoCtx {
            repo_path: dir.path(),
            spex_dir: dir.path(),
            candidates: &candidates,
        };
        let findings = validate_annotations(&ctx, true, &HashMap::new());

        assert!(findings.is_empty());
    }

    #[test]
    fn skips_schemas_dir_in_any_repo() {
        let dir = tempdir().expect("temp dir");
        let candidates = vec![write_candidate(
            dir.path(),
            "schemas/spec.schema.json",
            "@spec bogus\n",
        )];
        let ctx = RepoCtx {
            repo_path: dir.path(),
            spex_dir: dir.path(),
            candidates: &candidates,
        };
        let findings = validate_annotations(&ctx, false, &HashMap::new());

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
        let ctx = RepoCtx {
            repo_path: dir.path(),
            spex_dir: dir.path(),
            candidates: &candidates,
        };
        let findings = validate_annotations(
            &ctx,
            false,
            &known_with_acs("delivery-time", &["estimate-on-delivery"]),
        );

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].check, ORPHAN_CHECK);
        assert!(findings[0].message.contains("xxx"));
        assert!(findings[0].message.contains("delivery-time"));
    }

    #[test]
    fn does_not_truncate_an_id_with_a_digit_into_a_known_prefix() {
        let dir = tempdir().expect("temp dir");
        let candidates = vec![write_candidate(
            dir.path(),
            "src/lib.rs",
            "// @spec impl delivery-time2#estimate-on-delivery\n",
        )];
        let ctx = RepoCtx {
            repo_path: dir.path(),
            spex_dir: dir.path(),
            candidates: &candidates,
        };
        let findings = validate_annotations(
            &ctx,
            false,
            &known_with_acs("delivery-time", &["estimate-on-delivery"]),
        );

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].check, ORPHAN_CHECK);
        assert!(findings[0].message.contains("delivery-time2"));
    }
}
