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
    fn scan_workspace_reads_contract_files_inside_spex_dir() {
        let dir = tempfile::tempdir().expect("tempdir");
        let spex_dir = dir.path();
        std::fs::create_dir_all(spex_dir.join("contracts")).expect("contracts dir");
        std::fs::write(
            spex_dir.join("contracts/api.openapi.yaml"),
            "      x-spec: \"@spec contract a\"\n",
        )
        .expect("write contract");

        let config: crate::config::SpexConfig = serde_yaml_ng::from_str(
            "project_name: demo\ncli_version: 0.0.0\nrepositories:\n  - repository_name: demo-api\n    repository_short_name: api\n    path: ../demo-api\n    contracts:\n      - path: contracts/api.openapi.yaml\n        description: REST API\n",
        )
        .expect("parse config");

        let spex = crate::config::spex::Spex::from_config(spex_dir.to_path_buf(), config);
        let index = AnnotationIndex::scan_workspace(&spex).expect("scan");
        assert!(index.has_bare(&testing::spec_id("a"), CoverageLayer::Contract));
    }
}
