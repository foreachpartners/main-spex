#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::spec::{AcceptanceCriterion, Commitment, CoverageIgnore, Spec, Status};

    fn spec(id: &str, ac_ids: &[&str], coverage_ignore: Vec<CoverageIgnore>) -> Spec {
        Spec {
            id: id.to_string(),
            path: format!("{id}.yaml"),
            description: String::new(),
            acceptance_criteria: ac_ids
                .iter()
                .map(|id| AcceptanceCriterion {
                    id: id.to_string(),
                    commitment: Commitment::Must(id.to_string()),
                    is_done: None,
                    line: 0,
                })
                .collect(),
            blocked_by: Vec::new(),
            scope: Vec::new(),
            status: Status::Active,
            coverage_ignore,
        }
    }

    #[test]
    fn scan_collects_bare_layers() {
        let found = scan("// @spec contract a\n// @spec impl a\n// @spec test a\n");
        assert!(found.has_bare("a", CoverageLayer::Contract));
        assert!(found.has_bare("a", CoverageLayer::Impl));
        assert!(found.has_bare("a", CoverageLayer::Test));
    }

    #[test]
    fn scan_splits_multiple_ids_in_one_annotation() {
        let found = scan("// @spec test a, b, rate-limit\n");
        assert!(found.has_bare("a", CoverageLayer::Test));
        assert!(found.has_bare("b", CoverageLayer::Test));
        assert!(found.has_bare("rate-limit", CoverageLayer::Test));
    }

    #[test]
    fn scan_treats_vendor_extension_value_as_contract() {
        let found = scan("      x-spec: \"@spec contract a\"\n");
        assert!(found.has_bare("a", CoverageLayer::Contract));
        assert!(!found.has_bare("a", CoverageLayer::Impl));
    }

    #[test]
    fn scan_collects_ac_qualified_annotations() {
        let found = scan("// @spec impl a#empty-string\n// @spec test a#empty-string\n");
        assert!(found.has_ac("a", "empty-string", CoverageLayer::Impl));
        assert!(found.has_ac("a", "empty-string", CoverageLayer::Test));
        assert!(!found.has_ac("a", "other-ac", CoverageLayer::Impl));
    }

    #[test]
    fn scan_supports_ac_qualified_ids_in_a_list() {
        let found = scan("// @spec impl a#ac-one, b#ac-two\n");
        assert!(found.has_ac("a", "ac-one", CoverageLayer::Impl));
        assert!(found.has_ac("b", "ac-two", CoverageLayer::Impl));
    }

    #[test]
    fn scan_treats_ac_qualified_contract_annotation_as_bare() {
        let found = scan("// @spec contract a#empty-string\n");
        assert!(found.has_bare("a", CoverageLayer::Contract));
    }

    #[test]
    fn bare_annotation_does_not_satisfy_ac_level_coverage() {
        let found = scan("// @spec impl a\n// @spec test a\n");
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

        let found = scan_workspace(spex_dir, &config).expect("scan");
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

        let rows = ac_coverage(&spec, &found);
        assert!(rows[0].is_ok());
        assert!(!rows[1].is_ok());
        assert!(rows[1].impl_required && !rows[1].impl_present);
        assert!(rows[1].test_required && !rows[1].test_present);
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
        let rows = ac_coverage(&spec, &Found::new());
        assert!(!rows[0].impl_required);
        assert!(rows[0].test_required);
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
        let rows = ac_coverage(&spec, &Found::new());
        assert!(rows[0].impl_required);
        assert!(!rows[0].test_required);
    }

    #[test]
    fn contract_coverage_reads_bare_annotation() {
        let spec = spec("a", &[], Vec::new());
        let found = scan("// @spec contract a\n");
        assert_eq!(contract_coverage(&spec, &found), (true, true));
    }

    #[test]
    fn contract_coverage_respects_ignore() {
        let spec = spec(
            "a",
            &[],
            vec![CoverageIgnore {
                layer: CoverageLayer::Contract,
                ac: None,
            }],
        );
        assert_eq!(contract_coverage(&spec, &Found::new()), (false, false));
    }
}
