#[cfg(test)]
mod tests {
    use super::super::*;

    fn spec(coverage_ignore: Vec<CoverageIgnore>) -> Spec {
        Spec {
            id: "A".to_string(),
            path: "A.yaml".to_string(),
            description: String::new(),
            acceptance_criteria: Vec::new(),
            blocked_by: Vec::new(),
            scope: Vec::new(),
            status: Status::Active,
            coverage_ignore,
        }
    }

    fn ignore(layer: CoverageLayer, ac: Option<&str>) -> CoverageIgnore {
        CoverageIgnore {
            layer,
            ac: ac.map(str::to_string),
        }
    }

    #[test]
    fn layer_ignored_true_for_whole_layer_entry() {
        let spec = spec(vec![ignore(CoverageLayer::Contract, None)]);
        assert!(spec.layer_ignored(CoverageLayer::Contract));
        assert!(!spec.layer_ignored(CoverageLayer::Impl));
    }

    #[test]
    fn ac_ignored_true_when_layer_is_ignored_for_whole_spec() {
        let spec = spec(vec![ignore(CoverageLayer::Test, None)]);
        assert!(spec.ac_ignored("any-ac", CoverageLayer::Test));
    }

    #[test]
    fn ac_ignored_true_when_ac_specifically_ignored() {
        let spec = spec(vec![ignore(
            CoverageLayer::Impl,
            Some("no-null-typed-string"),
        )]);
        assert!(spec.ac_ignored("no-null-typed-string", CoverageLayer::Impl));
        assert!(!spec.ac_ignored("other-ac", CoverageLayer::Impl));
        assert!(!spec.ac_ignored("no-null-typed-string", CoverageLayer::Test));
    }

    #[test]
    fn ac_ignored_false_with_no_coverage_ignore() {
        let spec = spec(Vec::new());
        assert!(!spec.ac_ignored("any-ac", CoverageLayer::Impl));
    }

    #[test]
    fn coverage_ignore_parses_whole_layer_and_per_ac_entries() {
        let spec: Spec = serde_yaml_ng::from_str(
            "id: A\ncoverage_ignore:\n  - layer: contract\n  - layer: impl\n    ac: no-null-typed-string\n",
        )
        .expect("parse spec");
        assert!(spec.layer_ignored(CoverageLayer::Contract));
        assert!(spec.ac_ignored("no-null-typed-string", CoverageLayer::Impl));
        assert!(!spec.ac_ignored("other-ac", CoverageLayer::Impl));
    }

    #[test]
    fn is_done_defaults_true_when_absent() {
        let ac = AcceptanceCriterion {
            id: "a".to_string(),
            commitment: Commitment::Must("a".to_string()),
            is_done: None,
            line: 0,
        };
        assert!(ac.is_done());
    }

    #[test]
    fn is_done_reads_explicit_false() {
        let ac = AcceptanceCriterion {
            id: "a".to_string(),
            commitment: Commitment::Must("a".to_string()),
            is_done: Some(false),
            line: 0,
        };
        assert!(!ac.is_done());
    }
}
