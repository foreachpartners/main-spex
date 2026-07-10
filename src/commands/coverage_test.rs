#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::capability_spec::{CapabilitySpec, CoverageLayer, Status};

    fn spec(id: &str, status: Status) -> CapabilitySpec {
        CapabilitySpec {
            id: id.to_string(),
            acceptance_criteria: Vec::new(),
            blocked_by: Vec::new(),
            status,
            coverage_ignore: Vec::new(),
        }
    }

    fn row(id: &str, present: &[CoverageLayer], ignored: &[CoverageLayer]) -> SpecRow {
        SpecRow {
            id: id.to_string(),
            present: present.iter().copied().collect(),
            ignored: ignored.iter().copied().collect(),
            ac_done: 0,
            ac_total: 0,
        }
    }

    fn scan(content: &str) -> Found {
        let mut found = Found::new();
        Scanner::new().scan(content, &mut found);
        found
    }

    #[test]
    fn scan_collects_each_layer_from_spec_annotations() {
        let found = scan("// @spec contract CS-A\n// @spec impl CS-A\n// @spec test CS-A\n");
        let layers = found.get("CS-A").expect("CS-A present");
        assert!(layers.contains(&CoverageLayer::Contract));
        assert!(layers.contains(&CoverageLayer::Impl));
        assert!(layers.contains(&CoverageLayer::Test));
    }

    #[test]
    fn scan_splits_multiple_ids_in_one_annotation() {
        let found = scan("// @spec test CS-A, CS-B, RS-RATE-LIMIT\n");
        assert!(found.get("CS-A").unwrap().contains(&CoverageLayer::Test));
        assert!(found.get("CS-B").unwrap().contains(&CoverageLayer::Test));
        assert!(
            found
                .get("RS-RATE-LIMIT")
                .unwrap()
                .contains(&CoverageLayer::Test)
        );
    }

    #[test]
    fn scan_treats_xspec_as_contract() {
        let found = scan("        x-spec: CS-A\n");
        assert!(
            found
                .get("CS-A")
                .unwrap()
                .contains(&CoverageLayer::Contract)
        );
        assert!(!found.get("CS-A").unwrap().contains(&CoverageLayer::Impl));
    }

    #[test]
    fn scan_treats_quoted_xspec_as_contract() {
        let found = scan("x-spec: \"CS-A\"\n");
        assert!(
            found
                .get("CS-A")
                .unwrap()
                .contains(&CoverageLayer::Contract)
        );
    }

    #[test]
    fn scan_workspace_reads_contract_files_inside_spex_dir() {
        let dir = tempfile::tempdir().expect("tempdir");
        let spex_dir = dir.path();
        std::fs::create_dir_all(spex_dir.join("contracts")).expect("contracts dir");
        std::fs::write(
            spex_dir.join("contracts/api.openapi.yaml"),
            "      x-spec: CS-A\n",
        )
        .expect("write contract");

        let config: crate::config::Config = serde_yaml_ng::from_str(
            "project_name: demo\ncli_version: 0.0.0\nrepositories:\n  - repository_name: demo-api\n    repository_short_name: api\n    path: ../demo-api\n    contracts:\n      - path: contracts/api.openapi.yaml\n        description: REST API\n",
        )
        .expect("parse config");

        let found = scan_workspace(spex_dir, &config).expect("scan");
        assert!(
            found
                .get("CS-A")
                .unwrap()
                .contains(&CoverageLayer::Contract)
        );
    }

    #[test]
    fn active_rows_ignores_non_active_specs() {
        let specs = vec![
            spec("CS-A", Status::Active),
            spec("CS-B", Status::Planned),
            spec("CS-C", Status::Deprecated),
        ];
        let found = Found::new();
        let rows = active_rows(&specs, &found);
        let ids: Vec<&str> = rows.iter().map(|row| row.id.as_str()).collect();
        assert_eq!(ids, vec!["CS-A"]);
    }

    #[test]
    fn omitting_status_defaults_to_planned_and_is_ignored() {
        let spec: CapabilitySpec = serde_yaml_ng::from_str("id: CS-A\n").expect("parse spec");
        assert_eq!(spec.status, Status::Planned);
        assert!(spec.coverage_ignore.is_empty());
        let rows = active_rows(&[spec], &Found::new());
        assert!(rows.is_empty());
    }

    #[test]
    fn active_rows_carries_coverage_ignore() {
        let spec: CapabilitySpec = serde_yaml_ng::from_str(
            "id: CS-A\nstatus: active\ncoverage_ignore: [contract, test]\n",
        )
        .expect("parse spec");
        let rows = active_rows(&[spec], &Found::new());
        let row = &rows[0];
        assert!(!row.is_required(CoverageLayer::Contract));
        assert!(!row.is_required(CoverageLayer::Test));
        assert!(row.is_required(CoverageLayer::Impl));
    }

    #[test]
    fn missing_summary_lists_layers_in_order() {
        let row = row("CS-A", &[], &[]);
        assert_eq!(missing_summary(&row), "impl, test, contract");
    }

    #[test]
    fn missing_contract_is_a_gap_by_default() {
        let row = row("CS-A", &[CoverageLayer::Impl, CoverageLayer::Test], &[]);
        assert!(row.has_gap());
        assert!(!row.fully_covered());
        assert_eq!(missing_summary(&row), "contract");
    }

    #[test]
    fn ignored_layer_is_not_a_gap() {
        let row = row(
            "CS-A",
            &[CoverageLayer::Impl, CoverageLayer::Test],
            &[CoverageLayer::Contract],
        );
        assert!(!row.has_gap());
        assert!(!row.fully_covered());
        assert_eq!(missing_summary(&row), "contract (ignored)");
    }

    #[test]
    fn missing_ignored_impl_is_not_a_gap() {
        let row = row(
            "CS-A",
            &[CoverageLayer::Test, CoverageLayer::Contract],
            &[CoverageLayer::Impl],
        );
        assert!(!row.has_gap());
        assert_eq!(missing_summary(&row), "impl (ignored)");
    }

    #[test]
    fn missing_test_is_a_gap() {
        let row = row("CS-A", &[CoverageLayer::Impl, CoverageLayer::Contract], &[]);
        assert!(row.has_gap());
        assert_eq!(missing_summary(&row), "test");
    }

    #[test]
    fn unimplemented_ac_is_a_gap() {
        let mut row = row("CS-A", &CoverageLayer::ALL, &[]);
        row.ac_done = 1;
        row.ac_total = 2;
        assert!(row.has_gap());
        assert!(!row.fully_covered());
    }

    #[test]
    fn all_layers_and_ac_complete_is_fully_covered() {
        let mut row = row("CS-A", &CoverageLayer::ALL, &[]);
        row.ac_done = 3;
        row.ac_total = 3;
        assert!(!row.has_gap());
        assert!(row.fully_covered());
    }

    #[test]
    fn active_rows_carries_ac_counts() {
        let spec: CapabilitySpec = serde_yaml_ng::from_str(
            "id: CS-A\nstatus: active\nacceptance_criteria:\n  - \"[x] MUST do a\"\n  - \"[ ] MUST do b\"\n",
        )
        .expect("parse spec");
        let rows = active_rows(&[spec], &Found::new());
        assert_eq!(rows[0].ac_done, 1);
        assert_eq!(rows[0].ac_total, 2);
    }

    #[test]
    fn full_coverage_has_no_gap() {
        let row = row(
            "CS-A",
            &[
                CoverageLayer::Impl,
                CoverageLayer::Test,
                CoverageLayer::Contract,
            ],
            &[],
        );
        assert!(!row.has_gap());
        assert!(row.fully_covered());
        assert_eq!(missing_summary(&row), "");
    }
}
