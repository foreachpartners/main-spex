#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn from_config_keeps_the_path_and_numbers_criteria_by_source_line() {
        let raw = "id: a\nacceptance_criteria:\n  - \"MUST x\"\n  - \"SHOULD y\"\nchecks: []\n";
        let config = serde_yaml_ng::from_str(raw).expect("parse rule config");

        let rule = Rule::from_config(config, PathBuf::from("/spex/rules/a.yaml"), raw);

        assert_eq!(rule.path, PathBuf::from("/spex/rules/a.yaml"));
        let criteria: Vec<(&str, usize)> = rule
            .acceptance_criteria
            .iter()
            .map(|ac| (ac.text.as_str(), ac.line))
            .collect();
        assert_eq!(criteria, vec![("MUST x", 3), ("SHOULD y", 4)]);
    }
}
