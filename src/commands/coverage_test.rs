#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::biz_spec::{BizSpec, Status};

    fn spec(id: &str, status: Status) -> BizSpec {
        BizSpec {
            id: id.to_string(),
            status,
        }
    }

    fn scan(content: &str) -> Found {
        let mut found = Found::new();
        Scanner::new().scan(content, &mut found);
        found
    }

    #[test]
    fn scan_collects_each_layer_from_spec_annotations() {
        let found = scan("// @spec contract BIZ-A\n// @spec impl BIZ-A\n// @spec test BIZ-A\n");
        let layers = found.get("BIZ-A").expect("BIZ-A present");
        assert!(layers.contains(&Layer::Contract));
        assert!(layers.contains(&Layer::Impl));
        assert!(layers.contains(&Layer::Test));
    }

    #[test]
    fn scan_splits_multiple_ids_in_one_annotation() {
        let found = scan("// @spec test BIZ-A, BIZ-B, TECH-RATE-LIMIT\n");
        assert!(found.get("BIZ-A").unwrap().contains(&Layer::Test));
        assert!(found.get("BIZ-B").unwrap().contains(&Layer::Test));
        assert!(found.get("TECH-RATE-LIMIT").unwrap().contains(&Layer::Test));
    }

    #[test]
    fn scan_treats_xspec_as_contract() {
        let found = scan("        x-spec: BIZ-A\n");
        assert!(found.get("BIZ-A").unwrap().contains(&Layer::Contract));
        assert!(!found.get("BIZ-A").unwrap().contains(&Layer::Impl));
    }

    #[test]
    fn scan_treats_quoted_xspec_as_contract() {
        let found = scan("x-spec: \"BIZ-A\"\n");
        assert!(found.get("BIZ-A").unwrap().contains(&Layer::Contract));
    }

    #[test]
    fn active_rows_ignores_non_active_specs() {
        let specs = vec![
            spec("BIZ-A", Status::Active),
            spec("BIZ-B", Status::Planned),
            spec("BIZ-C", Status::Deprecated),
        ];
        let found = Found::new();
        let rows = active_rows(&specs, &found);
        let ids: Vec<&str> = rows.iter().map(|row| row.id.as_str()).collect();
        assert_eq!(ids, vec!["BIZ-A"]);
    }

    #[test]
    fn omitting_status_defaults_to_planned_and_is_ignored() {
        let spec: BizSpec = serde_yaml_ng::from_str("id: BIZ-A\n").expect("parse spec");
        assert_eq!(spec.status, Status::Planned);
        let rows = active_rows(&[spec], &Found::new());
        assert!(rows.is_empty());
    }

    #[test]
    fn missing_summary_lists_required_then_optional() {
        let row = SpecRow {
            id: "BIZ-A".to_string(),
            has_contract: false,
            has_impl: false,
            has_test: false,
        };
        assert_eq!(missing_summary(&row), "impl, test, contract (optional)");
    }

    #[test]
    fn missing_only_contract_is_not_a_gap() {
        let row = SpecRow {
            id: "BIZ-A".to_string(),
            has_contract: false,
            has_impl: true,
            has_test: true,
        };
        assert!(!row.has_gap());
        assert!(!row.fully_covered());
        assert_eq!(missing_summary(&row), "contract (optional)");
    }

    #[test]
    fn missing_test_is_a_gap() {
        let row = SpecRow {
            id: "BIZ-A".to_string(),
            has_contract: true,
            has_impl: true,
            has_test: false,
        };
        assert!(row.has_gap());
        assert_eq!(missing_summary(&row), "test");
    }

    #[test]
    fn full_coverage_has_no_gap() {
        let row = SpecRow {
            id: "BIZ-A".to_string(),
            has_contract: true,
            has_impl: true,
            has_test: true,
        };
        assert!(!row.has_gap());
        assert!(row.fully_covered());
        assert_eq!(missing_summary(&row), "");
    }
}
