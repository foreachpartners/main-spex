#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::support::testing;

    #[test]
    fn scan_collects_bare_layers() {
        let index =
            AnnotationIndex::scan("// @spec contract a\n// @spec impl a\n// @spec test a\n");
        assert!(index.has_bare(&testing::spec_id("a"), CoverageLayer::Contract));
        assert!(index.has_bare(&testing::spec_id("a"), CoverageLayer::Impl));
        assert!(index.has_bare(&testing::spec_id("a"), CoverageLayer::Test));
    }

    #[test]
    fn scan_splits_multiple_ids_in_one_annotation() {
        let index = AnnotationIndex::scan("// @spec test a, b, rate-limit\n");
        assert!(index.has_bare(&testing::spec_id("a"), CoverageLayer::Test));
        assert!(index.has_bare(&testing::spec_id("b"), CoverageLayer::Test));
        assert!(index.has_bare(&testing::spec_id("rate-limit"), CoverageLayer::Test));
    }

    #[test]
    fn scan_treats_vendor_extension_value_as_contract() {
        let index = AnnotationIndex::scan("      x-spec: \"@spec contract a\"\n");
        assert!(index.has_bare(&testing::spec_id("a"), CoverageLayer::Contract));
        assert!(!index.has_bare(&testing::spec_id("a"), CoverageLayer::Impl));
    }

    #[test]
    fn scan_collects_ac_qualified_annotations() {
        let index =
            AnnotationIndex::scan("// @spec impl a#empty-string\n// @spec test a#empty-string\n");
        assert!(index.has_ac(
            &testing::spec_id("a"),
            &testing::ac_id("empty-string"),
            CoverageLayer::Impl
        ));
        assert!(index.has_ac(
            &testing::spec_id("a"),
            &testing::ac_id("empty-string"),
            CoverageLayer::Test
        ));
        assert!(!index.has_ac(
            &testing::spec_id("a"),
            &testing::ac_id("other-ac"),
            CoverageLayer::Impl
        ));
    }

    #[test]
    fn scan_supports_ac_qualified_ids_in_a_list() {
        let index = AnnotationIndex::scan("// @spec impl a#ac-one, b#ac-two\n");
        assert!(index.has_ac(
            &testing::spec_id("a"),
            &testing::ac_id("ac-one"),
            CoverageLayer::Impl
        ));
        assert!(index.has_ac(
            &testing::spec_id("b"),
            &testing::ac_id("ac-two"),
            CoverageLayer::Impl
        ));
    }

    #[test]
    fn scan_treats_ac_qualified_contract_annotation_as_bare() {
        let index = AnnotationIndex::scan("// @spec contract a#empty-string\n");
        assert!(index.has_bare(&testing::spec_id("a"), CoverageLayer::Contract));
    }

    #[test]
    fn scan_collects_ac_qualified_contract_annotation_per_ac() {
        let index = AnnotationIndex::scan("// @spec contract a#empty-string\n");
        assert!(index.has_ac(
            &testing::spec_id("a"),
            &testing::ac_id("empty-string"),
            CoverageLayer::Contract
        ));
        assert!(!index.has_ac(
            &testing::spec_id("a"),
            &testing::ac_id("other-ac"),
            CoverageLayer::Contract
        ));
        assert!(!index.has_ac(
            &testing::spec_id("a"),
            &testing::ac_id("empty-string"),
            CoverageLayer::Impl
        ));
    }

    #[test]
    fn bare_annotation_does_not_satisfy_ac_level_coverage() {
        let index = AnnotationIndex::scan("// @spec impl a\n// @spec test a\n");
        assert!(!index.has_ac(
            &testing::spec_id("a"),
            &testing::ac_id("any-ac"),
            CoverageLayer::Impl
        ));
        assert!(!index.has_ac(
            &testing::spec_id("a"),
            &testing::ac_id("any-ac"),
            CoverageLayer::Test
        ));
    }

    #[test]
    fn scan_project_reads_contract_files_inside_spex_dir() {
        let dir = tempfile::tempdir().expect("tempdir");
        let spex_dir = dir.path();
        std::fs::create_dir_all(spex_dir.join("contracts")).expect("contracts dir");
        std::fs::write(
            spex_dir.join("contracts/api.openapi.yaml"),
            "      x-spec: \"@spec contract a\"\n",
        )
        .expect("write contract");

        let config: crate::config::SpexConfig = serde_yaml_ng::from_str(
            "project_name: demo\ncli_version: 0.0.0\nlayout: multi-repo\nagent_root: .\npackages:\n  - package_name: demo-api\n    package_short_name: api\n    path: ../demo-api\n    contracts:\n      - path: contracts/api.openapi.yaml\n        description: REST API\n",
        )
        .expect("parse config");

        let spex = crate::config::spex::Spex::from_config(spex_dir.to_path_buf(), config);
        let index = AnnotationIndex::scan_project(&spex).expect("scan");
        assert!(index.has_bare(&testing::spec_id("a"), CoverageLayer::Contract));
    }

    #[test]
    fn scan_project_skips_the_spex_dir_nested_in_a_package() {
        let dir = tempfile::tempdir().expect("tempdir");
        let spex_dir = dir.path().join("spex");
        std::fs::create_dir_all(spex_dir.join("specs")).expect("specs dir");
        std::fs::create_dir_all(dir.path().join("src")).expect("src dir");
        std::fs::write(spex_dir.join("specs/a.yaml"), "# @spec impl a#some-ac\n")
            .expect("write spec");
        std::fs::write(dir.path().join("src/lib.rs"), "// @spec test b#other-ac\n")
            .expect("write source");

        let config: crate::config::SpexConfig = serde_yaml_ng::from_str(
            "project_name: demo\ncli_version: 0.0.0\nlayout: single-repo\nagent_root: ..\npackages:\n  - package_name: demo\n    package_short_name: demo\n    path: ..\n",
        )
        .expect("parse config");

        let spex = crate::config::spex::Spex::from_config(spex_dir, config);
        let index = AnnotationIndex::scan_project(&spex).expect("scan");

        assert!(!index.has_ac(
            &testing::spec_id("a"),
            &testing::ac_id("some-ac"),
            CoverageLayer::Impl
        ));
        assert!(index.has_ac(
            &testing::spec_id("b"),
            &testing::ac_id("other-ac"),
            CoverageLayer::Test
        ));
    }

    fn single_package_spex(
        spex_dir: std::path::PathBuf,
        package_entry: &str,
    ) -> crate::config::spex::Spex {
        let config: crate::config::SpexConfig = serde_yaml_ng::from_str(&format!(
            "project_name: demo\ncli_version: 0.0.0\nlayout: single-repo\nagent_root: ..\npackages:\n{package_entry}"
        ))
        .expect("parse config");
        crate::config::spex::Spex::from_config(spex_dir, config)
    }

    #[test]
    fn scan_project_skips_files_and_folders_a_package_excludes() {
        let dir = tempfile::tempdir().expect("tempdir");
        let spex_dir = dir.path().join("spex");
        std::fs::create_dir_all(&spex_dir).expect("spex dir");
        for (file, annotation) in [
            ("src/lib.rs", "// @spec impl kept#kept-ac\n"),
            ("vendor/dep.rs", "// @spec impl folder#folder-ac\n"),
            ("src/api.generated.ts", "// @spec impl glob#glob-ac\n"),
        ] {
            let path = dir.path().join(file);
            std::fs::create_dir_all(path.parent().expect("parent")).expect("parent dir");
            std::fs::write(path, annotation).expect("write source");
        }

        let spex = single_package_spex(
            spex_dir,
            "  - package_name: demo\n    package_short_name: demo\n    path: ..\n    exclude:\n      - vendor\n      - \"**/*.generated.ts\"\n",
        );
        let index = AnnotationIndex::scan_project(&spex).expect("scan");

        for (spec, ac, expected) in [
            ("kept", "kept-ac", true),
            ("folder", "folder-ac", false),
            ("glob", "glob-ac", false),
        ] {
            assert_eq!(
                index.has_ac(
                    &testing::spec_id(spec),
                    &testing::ac_id(ac),
                    CoverageLayer::Impl
                ),
                expected,
                "{spec}"
            );
        }
    }

    #[test]
    fn scan_project_still_reads_a_contract_file_the_package_excludes() {
        let dir = tempfile::tempdir().expect("tempdir");
        let spex_dir = dir.path().join("spex");
        std::fs::create_dir_all(&spex_dir).expect("spex dir");
        std::fs::create_dir_all(dir.path().join("contracts")).expect("contracts dir");
        std::fs::write(
            dir.path().join("contracts/api.openapi.yaml"),
            "      x-spec: \"@spec contract a\"\n",
        )
        .expect("write contract");

        let spex = single_package_spex(
            spex_dir,
            "  - package_name: demo\n    package_short_name: demo\n    path: ..\n    contracts:\n      - path: ../contracts/api.openapi.yaml\n        description: REST API\n    exclude:\n      - contracts\n",
        );
        let index = AnnotationIndex::scan_project(&spex).expect("scan");

        assert!(index.has_bare(&testing::spec_id("a"), CoverageLayer::Contract));
    }

    #[test]
    fn scan_project_does_not_apply_the_exclude_of_one_package_to_another() {
        let dir = tempfile::tempdir().expect("tempdir");
        let spex_dir = dir.path().join("spex");
        std::fs::create_dir_all(&spex_dir).expect("spex dir");
        for package in ["api", "web"] {
            let path = dir.path().join(package).join("vendor/dep.rs");
            std::fs::create_dir_all(path.parent().expect("parent")).expect("parent dir");
            std::fs::write(path, format!("// @spec impl {package}#vendored-ac\n"))
                .expect("write source");
        }

        let spex = single_package_spex(
            spex_dir,
            "  - package_name: demo-api\n    package_short_name: api\n    path: ../api\n    exclude: [vendor]\n  - package_name: demo-web\n    package_short_name: web\n    path: ../web\n",
        );
        let index = AnnotationIndex::scan_project(&spex).expect("scan");

        assert!(!index.has_ac(
            &testing::spec_id("api"),
            &testing::ac_id("vendored-ac"),
            CoverageLayer::Impl
        ));
        assert!(index.has_ac(
            &testing::spec_id("web"),
            &testing::ac_id("vendored-ac"),
            CoverageLayer::Impl
        ));
    }

    #[test]
    fn scan_project_fails_naming_the_package_when_an_exclude_is_not_a_valid_glob() {
        let dir = tempfile::tempdir().expect("tempdir");
        let spex_dir = dir.path().join("spex");
        std::fs::create_dir_all(&spex_dir).expect("spex dir");

        let spex = single_package_spex(
            spex_dir,
            "  - package_name: demo\n    package_short_name: demo\n    path: ..\n    exclude: [\"src/{a,b\"]\n",
        );
        let error = AnnotationIndex::scan_project(&spex).expect_err("invalid exclude");

        let message = crate::error::describe(&error);
        assert!(message.contains("`demo`"), "{message}");
        assert!(message.contains("src/{a,b"), "{message}");
    }
}
