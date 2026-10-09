#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::config::Repository;
    use crate::support::testing;
    use indoc::{formatdoc, indoc};
    use tempfile::tempdir;

    fn repo(name: &str, rules: &[&str]) -> Repository {
        Repository {
            rules: rules.iter().map(|r| testing::rule_id(r)).collect(),
            ..testing::repository(name, &format!("../{name}"))
        }
    }

    fn rule(id: &str, extra: &str) -> serde_json::Value {
        serde_json::from_str(&format!(r#"{{"id": "{id}"{extra}}}"#)).expect("valid json")
    }

    #[test]
    fn repo_has_rule_matches_assigned_ids_only() {
        let api = repo("demo-api", &["secret-handling"]);
        assert!(repo_has_rule(&api, &rule("secret-handling", "")));
        assert!(!repo_has_rule(&api, &rule("writing-style", "")));
    }

    #[test]
    fn id_selected_defaults_to_true_when_empty() {
        assert!(id_selected(&rule("a", ""), &[]));
    }

    #[test]
    fn id_selected_filters_to_given_ids() {
        let ids = vec!["a".to_string(), "b".to_string()];
        assert!(id_selected(&rule("a", ""), &ids));
        assert!(!id_selected(&rule("c", ""), &ids));
    }

    #[test]
    fn repo_has_rule_and_id_selected_together_filter_assigned_and_selected_rules() {
        let api = repo("demo-api", &["a", "b"]);
        let rules = [rule("a", ""), rule("b", ""), rule("c", "")];

        let ids: Vec<&str> = rules
            .iter()
            .filter(|rule| repo_has_rule(&api, rule))
            .filter(|rule| id_selected(rule, &[]))
            .map(|r| r["id"].as_str().unwrap())
            .collect();
        assert_eq!(ids, vec!["a", "b"]);

        let filtered_ids: Vec<&str> = rules
            .iter()
            .filter(|rule| repo_has_rule(&api, rule))
            .filter(|rule| id_selected(rule, &["a".to_string()]))
            .map(|r| r["id"].as_str().unwrap())
            .collect();
        assert_eq!(filtered_ids, vec!["a"]);
    }

    fn write_workspace(dir: &std::path::Path) {
        std::fs::write(
            dir.join("spex.yaml"),
            indoc! {"
                project_name: demo
                cli_version: 0.0.0
                repositories:
                  - repository_name: demo-api
                    repository_short_name: api
                    path: ../demo-api
                    rules: [secret-handling]
                  - repository_name: demo-frontend
                    repository_short_name: frontend
                    path: ../demo-frontend
                    rules: [writing-style]
            "},
        )
        .expect("write spex.yaml");
        std::fs::create_dir_all(dir.join("rules")).expect("rules dir");
        std::fs::write(
            dir.join("rules/secret-handling.yaml"),
            indoc! {r#"
                id: secret-handling
                title: Secret protection
                description: no secrets
                sync_with_upstream: true
                acceptance_criteria:
                  - "MUST not commit secrets"
                checks:
                  - name: sample
                    type: glob
                    pattern: "**/*.bak"
            "#},
        )
        .expect("write rule");
        std::fs::write(
            dir.join("rules/writing-style.yaml"),
            indoc! {r#"
                id: writing-style
                title: Writing style
                description: consistent prose
                acceptance_criteria:
                  - "MUST use plain language"
            "#},
        )
        .expect("write rule");
    }

    fn workspace(dir: &std::path::Path) -> Spex {
        let config = crate::config::SpexConfig::load(&dir.join(crate::config::SPEX_CONFIG_FILE))
            .expect("load config");
        Spex::from_config(dir.to_path_buf(), config)
    }

    fn args(ai: bool, id: &[&str], repo: &[&str]) -> RulesArgs {
        RulesArgs {
            ai,
            id: id.iter().map(|s| s.to_string()).collect(),
            repo: repo.iter().map(|s| s.to_string()).collect(),
            output: None,
        }
    }

    #[test]
    fn render_with_no_flags_prints_every_assigned_rule() {
        let dir = tempdir().expect("temp dir");
        write_workspace(dir.path());
        let workspace = workspace(dir.path());

        let yaml = render(&workspace, &args(false, &[], &[])).expect("render");

        assert_eq!(
            yaml,
            indoc! {"
                - id: secret-handling
                  title: Secret protection
                  description: no secrets
                  sync_with_upstream: true
                  acceptance_criteria:
                  - MUST not commit secrets
                  checks:
                  - name: sample
                    type: glob
                    pattern: '**/*.bak'

                - id: writing-style
                  title: Writing style
                  description: consistent prose
                  acceptance_criteria:
                  - MUST use plain language
            "}
        );
    }

    #[test]
    fn render_with_ai_flag_strips_sync_with_upstream_and_checks() {
        let dir = tempdir().expect("temp dir");
        write_workspace(dir.path());
        let workspace = workspace(dir.path());

        let yaml = render(&workspace, &args(true, &["secret-handling"], &[])).expect("render");

        assert_eq!(
            yaml,
            formatdoc! {"
                - id: secret-handling
                  path: {dir}/rules/secret-handling.yaml
                  title: Secret protection
                  description: no secrets
                  acceptance_criteria:
                  - MUST not commit secrets
            ", dir = dir.path().display()}
        );
    }

    #[test]
    fn render_with_ai_flag_preserves_field_order_after_dropping_checks() {
        let dir = tempdir().expect("temp dir");
        std::fs::write(
            dir.path().join("spex.yaml"),
            indoc! {"
                project_name: demo
                cli_version: 0.0.0
                repositories:
                  - repository_name: demo-api
                    repository_short_name: api
                    path: ../demo-api
                    rules: [spec-id-tracing]
            "},
        )
        .expect("write spex.yaml");
        std::fs::create_dir_all(dir.path().join("rules")).expect("rules dir");
        std::fs::write(
            dir.path().join("rules/spec-id-tracing.yaml"),
            indoc! {r#"
                id: spec-id-tracing
                title: Spec ID tracing
                description: Traces artifacts back to their spec.
                sync_with_upstream: true
                acceptance_criteria:
                  - "MUST annotate the boundary"
                checks:
                  - name: sample
                    type: glob
                    pattern: "**/*.bak"
                examples:
                  contract: 'x-spec: "@spec contract list-pizza-menu"'
            "#},
        )
        .expect("write rule");
        let workspace = workspace(dir.path());

        let yaml = render(&workspace, &args(true, &[], &[])).expect("render");

        assert_eq!(
            yaml,
            formatdoc! {"
                - id: spec-id-tracing
                  path: {dir}/rules/spec-id-tracing.yaml
                  title: Spec ID tracing
                  description: Traces artifacts back to their spec.
                  acceptance_criteria:
                  - MUST annotate the boundary
                  examples:
                    contract: 'x-spec: \"@spec contract list-pizza-menu\"'
            ", dir = dir.path().display()}
        );
    }

    #[test]
    fn render_with_single_id_returns_only_that_rule() {
        let dir = tempdir().expect("temp dir");
        write_workspace(dir.path());
        let workspace = workspace(dir.path());

        let yaml = render(&workspace, &args(false, &["writing-style"], &[])).expect("render");

        assert_eq!(
            yaml,
            indoc! {"
                - id: writing-style
                  title: Writing style
                  description: consistent prose
                  acceptance_criteria:
                  - MUST use plain language
            "}
        );
    }

    #[test]
    fn render_with_multiple_ids_returns_matching_rules_only() {
        let dir = tempdir().expect("temp dir");
        write_workspace(dir.path());
        let workspace = workspace(dir.path());

        let yaml = render(
            &workspace,
            &args(true, &["secret-handling", "writing-style"], &[]),
        )
        .expect("render");

        assert_eq!(
            yaml,
            formatdoc! {"
                - id: secret-handling
                  path: {dir}/rules/secret-handling.yaml
                  title: Secret protection
                  description: no secrets
                  acceptance_criteria:
                  - MUST not commit secrets

                - id: writing-style
                  path: {dir}/rules/writing-style.yaml
                  title: Writing style
                  description: consistent prose
                  acceptance_criteria:
                  - MUST use plain language
            ", dir = dir.path().display()}
        );
    }

    #[test]
    fn render_with_single_repo_returns_only_that_repos_rule() {
        let dir = tempdir().expect("temp dir");
        write_workspace(dir.path());
        let workspace = workspace(dir.path());

        let yaml = render(&workspace, &args(true, &[], &["demo-frontend"])).expect("render");

        assert_eq!(
            yaml,
            formatdoc! {"
                - id: writing-style
                  path: {dir}/rules/writing-style.yaml
                  title: Writing style
                  description: consistent prose
                  acceptance_criteria:
                  - MUST use plain language
            ", dir = dir.path().display()}
        );
    }

    #[test]
    fn render_with_multiple_repos_returns_union_of_their_rules() {
        let dir = tempdir().expect("temp dir");
        write_workspace(dir.path());
        let workspace = workspace(dir.path());

        let yaml =
            render(&workspace, &args(true, &[], &["demo-api", "demo-frontend"])).expect("render");

        assert_eq!(
            yaml,
            formatdoc! {"
                - id: secret-handling
                  path: {dir}/rules/secret-handling.yaml
                  title: Secret protection
                  description: no secrets
                  acceptance_criteria:
                  - MUST not commit secrets

                - id: writing-style
                  path: {dir}/rules/writing-style.yaml
                  title: Writing style
                  description: consistent prose
                  acceptance_criteria:
                  - MUST use plain language
            ", dir = dir.path().display()}
        );
    }

    #[test]
    fn render_with_repo_short_name_matches_same_as_full_name() {
        let dir = tempdir().expect("temp dir");
        write_workspace(dir.path());
        let workspace = workspace(dir.path());

        let yaml = render(&workspace, &args(true, &[], &["api"])).expect("render");

        assert_eq!(
            yaml,
            formatdoc! {"
                - id: secret-handling
                  path: {dir}/rules/secret-handling.yaml
                  title: Secret protection
                  description: no secrets
                  acceptance_criteria:
                  - MUST not commit secrets
            ", dir = dir.path().display()}
        );
    }

    #[test]
    fn render_errors_on_unknown_repo() {
        let dir = tempdir().expect("temp dir");
        write_workspace(dir.path());
        let workspace = workspace(dir.path());

        let result = render(&workspace, &args(false, &[], &["missing"]));

        assert!(result.is_err());
    }

    #[test]
    fn render_errors_on_unknown_id() {
        let dir = tempdir().expect("temp dir");
        write_workspace(dir.path());
        let workspace = workspace(dir.path());

        let result = render(&workspace, &args(false, &["missing"], &[]));

        assert!(result.is_err());
    }
}
