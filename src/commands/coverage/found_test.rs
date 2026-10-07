#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::spec::{ContractLevel, CoverageIgnore, Spec, Status};
    use crate::support::testing;

    fn spec(id: &str, ac_ids: &[&str], coverage_ignore: Vec<CoverageIgnore>) -> Spec {
        Spec {
            acceptance_criteria: ac_ids.iter().map(|id| testing::criterion(id)).collect(),
            status: Status::Active,
            coverage_ignore,
            ..testing::spec(id)
        }
    }

    #[test]
    fn scan_collects_bare_layers() {
        let found = Found::scan("// @spec contract a\n// @spec impl a\n// @spec test a\n");
        assert!(found.has_bare("a", CoverageLayer::Contract));
        assert!(found.has_bare("a", CoverageLayer::Impl));
        assert!(found.has_bare("a", CoverageLayer::Test));
    }

    #[test]
    fn scan_splits_multiple_ids_in_one_annotation() {
        let found = Found::scan("// @spec test a, b, rate-limit\n");
        assert!(found.has_bare("a", CoverageLayer::Test));
        assert!(found.has_bare("b", CoverageLayer::Test));
        assert!(found.has_bare("rate-limit", CoverageLayer::Test));
    }

    #[test]
    fn scan_treats_vendor_extension_value_as_contract() {
        let found = Found::scan("      x-spec: \"@spec contract a\"\n");
        assert!(found.has_bare("a", CoverageLayer::Contract));
        assert!(!found.has_bare("a", CoverageLayer::Impl));
    }

    #[test]
    fn scan_collects_ac_qualified_annotations() {
        let found = Found::scan("// @spec impl a#empty-string\n// @spec test a#empty-string\n");
        assert!(found.has_ac("a", "empty-string", CoverageLayer::Impl));
        assert!(found.has_ac("a", "empty-string", CoverageLayer::Test));
        assert!(!found.has_ac("a", "other-ac", CoverageLayer::Impl));
    }

    #[test]
    fn scan_supports_ac_qualified_ids_in_a_list() {
        let found = Found::scan("// @spec impl a#ac-one, b#ac-two\n");
        assert!(found.has_ac("a", "ac-one", CoverageLayer::Impl));
        assert!(found.has_ac("b", "ac-two", CoverageLayer::Impl));
    }

    #[test]
    fn scan_treats_ac_qualified_contract_annotation_as_bare() {
        let found = Found::scan("// @spec contract a#empty-string\n");
        assert!(found.has_bare("a", CoverageLayer::Contract));
    }

    #[test]
    fn scan_collects_ac_qualified_contract_annotation_per_ac() {
        let found = Found::scan("// @spec contract a#empty-string\n");
        assert!(found.has_ac("a", "empty-string", CoverageLayer::Contract));
        assert!(!found.has_ac("a", "other-ac", CoverageLayer::Contract));
        assert!(!found.has_ac("a", "empty-string", CoverageLayer::Impl));
    }

    #[test]
    fn bare_annotation_does_not_satisfy_ac_level_coverage() {
        let found = Found::scan("// @spec impl a\n// @spec test a\n");
        assert!(!found.has_ac("a", "any-ac", CoverageLayer::Impl));
        assert!(!found.has_ac("a", "any-ac", CoverageLayer::Test));
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

        let config: crate::config::Config = serde_yaml_ng::from_str(
            "project_name: demo\ncli_version: 0.0.0\nrepositories:\n  - repository_name: demo-api\n    repository_short_name: api\n    path: ../demo-api\n    contracts:\n      - path: contracts/api.openapi.yaml\n        description: REST API\n",
        )
        .expect("parse config");

        let found = Found::scan_workspace(spex_dir, &config).expect("scan");
        assert!(found.has_bare("a", CoverageLayer::Contract));
    }

    #[test]
    fn ac_coverage_reports_missing_impl_and_test() {
        let spec = spec("a", &["ac-1", "ac-2"], Vec::new());
        let mut found = Found::new();
        found.by_ac.insert(("a".to_string(), "ac-1".to_string()), {
            let mut set = HashSet::new();
            set.insert(CoverageLayer::Impl);
            set.insert(CoverageLayer::Test);
            set
        });

        let rows = found.ac_coverage(&spec);
        assert!(rows[0].is_ok());
        assert!(!rows[1].is_ok());
        assert_eq!(
            rows[1].missing,
            vec![CoverageLayer::Impl, CoverageLayer::Test]
        );
    }

    #[test]
    fn ac_coverage_respects_per_ac_ignore() {
        let spec = spec(
            "a",
            &["ac-1"],
            vec![CoverageIgnore {
                layer: CoverageLayer::Impl,
                ac: Some("ac-1".to_string()),
            }],
        );
        let rows = Found::new().ac_coverage(&spec);
        assert_eq!(rows[0].missing, vec![CoverageLayer::Test]);
        assert!(!rows[0].is_ok());
    }

    #[test]
    fn ac_coverage_respects_whole_layer_ignore() {
        let spec = spec(
            "a",
            &["ac-1"],
            vec![CoverageIgnore {
                layer: CoverageLayer::Test,
                ac: None,
            }],
        );
        let rows = Found::new().ac_coverage(&spec);
        assert_eq!(rows[0].missing, vec![CoverageLayer::Impl]);
    }

    #[test]
    fn contract_missing_is_cleared_by_a_bare_annotation() {
        let spec = spec("a", &[], Vec::new());
        assert!(Found::new().contract_missing(&spec));
        let found = Found::scan("// @spec contract a\n");
        assert!(!found.contract_missing(&spec));
    }

    #[test]
    fn contract_missing_respects_ignore() {
        let spec = spec(
            "a",
            &[],
            vec![CoverageIgnore {
                layer: CoverageLayer::Contract,
                ac: None,
            }],
        );
        assert!(!Found::new().contract_missing(&spec));
    }

    #[test]
    fn contract_missing_never_applies_under_none() {
        let mut spec = spec("a", &[], Vec::new());
        spec.contract = ContractLevel::None;
        assert!(!Found::new().contract_missing(&spec));
    }

    #[test]
    fn contract_missing_is_spec_level_under_single() {
        let mut spec = spec("a", &[], Vec::new());
        spec.contract = ContractLevel::Single;
        let found = Found::scan("// @spec contract a#ac-1\n");
        assert!(!found.contract_missing(&spec));
    }

    #[test]
    fn contract_missing_is_per_ac_under_for_each_ac() {
        let mut spec = spec("a", &["ac-one", "ac-two"], Vec::new());
        spec.contract = ContractLevel::ForEachAc;
        assert!(!Found::new().contract_missing(&spec));

        let found = Found::scan("// @spec contract a#ac-one\n");
        let rows = found.ac_coverage(&spec);
        assert_eq!(
            rows[0].missing,
            vec![CoverageLayer::Impl, CoverageLayer::Test]
        );
        assert_eq!(
            rows[1].missing,
            vec![
                CoverageLayer::Contract,
                CoverageLayer::Impl,
                CoverageLayer::Test
            ]
        );
    }

    #[test]
    fn ac_coverage_under_for_each_ac_respects_contract_ignore() {
        let mut spec = spec(
            "a",
            &["ac-one"],
            vec![CoverageIgnore {
                layer: CoverageLayer::Contract,
                ac: Some("ac-one".to_string()),
            }],
        );
        spec.contract = ContractLevel::ForEachAc;
        let rows = Found::new().ac_coverage(&spec);
        assert_eq!(
            rows[0].missing,
            vec![CoverageLayer::Impl, CoverageLayer::Test]
        );
    }
}
