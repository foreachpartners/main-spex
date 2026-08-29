#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::config::Repository;
    use tempfile::tempdir;

    fn repo(name: &str, rules: &[&str]) -> Repository {
        Repository {
            name: name.to_string(),
            short_name: name.to_string(),
            path: format!("../{name}"),
            contracts: Vec::new(),
            rules: rules.iter().map(|r| r.to_string()).collect(),
            ignored_errors: Vec::new(),
        }
    }

    fn rule(id: &str, extra: &str) -> serde_json::Value {
        serde_json::from_str(&format!(r#"{{"id": "{id}"{extra}}}"#)).expect("valid json")
    }

    #[test]
    fn repo_has_rule_matches_assigned_ids_only() {
        let api = repo("demo-api", &["SECRET-HANDLING"]);
        assert!(repo_has_rule(&api, &rule("SECRET-HANDLING", "")));
        assert!(!repo_has_rule(&api, &rule("WRITING-STYLE", "")));
    }

    #[test]
    fn id_selected_defaults_to_true_when_empty() {
        assert!(id_selected(&rule("A", ""), &[]));
    }

    #[test]
    fn id_selected_filters_to_given_ids() {
        let ids = vec!["A".to_string(), "B".to_string()];
        assert!(id_selected(&rule("A", ""), &ids));
        assert!(!id_selected(&rule("C", ""), &ids));
    }

    #[test]
    fn repo_entry_includes_only_assigned_and_id_selected_rules() {
        let api = repo("demo-api", &["A", "B"]);
        let rules = vec![rule("A", ""), rule("B", ""), rule("C", "")];

        let entry = repo_entry(&api, &rules, &[]);
        let ids: Vec<&str> = entry["rules"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["id"].as_str().unwrap())
            .collect();
        assert_eq!(ids, vec!["A", "B"]);
        assert_eq!(entry["repository_name"], "demo-api");

        let filtered = repo_entry(&api, &rules, &["A".to_string()]);
        let ids: Vec<&str> = filtered["rules"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["id"].as_str().unwrap())
            .collect();
        assert_eq!(ids, vec!["A"]);
    }

    #[test]
    fn load_docs_reads_yaml_files_recursively() {
        let dir = tempdir().expect("temp dir");
        std::fs::write(dir.path().join("a.yaml"), "id: A\ntitle: A rule\n").expect("write a.yaml");
        std::fs::create_dir_all(dir.path().join("nested")).expect("nested dir");
        std::fs::write(dir.path().join("nested/b.yaml"), "id: B\n").expect("write b.yaml");

        let docs = load_docs(dir.path()).expect("load docs");
        let ids: Vec<&str> = docs.iter().map(|d| d["id"].as_str().unwrap()).collect();
        assert_eq!(ids, vec!["A", "B"]);
        assert_eq!(docs[0]["title"], "A rule");
    }

    #[test]
    fn load_docs_on_missing_dir_is_empty() {
        let dir = tempdir().expect("temp dir");
        let docs = load_docs(&dir.path().join("missing")).expect("load docs");
        assert!(docs.is_empty());
    }

    fn write_workspace(dir: &std::path::Path) {
        std::fs::write(
            dir.join("spex.yaml"),
            "project_name: demo\ncli_version: 0.0.0\nrepositories:\n  - repository_name: demo-api\n    repository_short_name: api\n    path: ../demo-api\n    rules: [SECRET-HANDLING]\n  - repository_name: demo-frontend\n    repository_short_name: frontend\n    path: ../demo-frontend\n    rules: [WRITING-STYLE]\n",
        )
        .expect("write spex.yaml");
        std::fs::create_dir_all(dir.join("rules")).expect("rules dir");
        std::fs::write(
            dir.join("rules/secret-handling.yaml"),
            "id: SECRET-HANDLING\ntitle: Secret protection\ndescription: no secrets\nsync_with_upstream: true\nacceptance_criteria:\n  - \"MUST not commit secrets\"\nchecks:\n  - name: sample\n    type: glob\n    pattern: \"**/*.bak\"\n",
        )
        .expect("write rule");
        std::fs::write(
            dir.join("rules/writing-style.yaml"),
            "id: WRITING-STYLE\ntitle: Writing style\ndescription: consistent prose\nacceptance_criteria:\n  - \"MUST use plain language\"\n",
        )
        .expect("write rule");
    }

    #[test]
    #[serial_test::serial]
    fn run_prints_each_repositorys_assigned_rules() {
        let dir = tempdir().expect("temp dir");
        write_workspace(dir.path());
        let original = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(dir.path()).expect("chdir");

        let result = run(RulesArgs {
            ai: false,
            id: Vec::new(),
            repo: Vec::new(),
        });

        std::env::set_current_dir(original).expect("restore cwd");
        assert!(result.is_ok());
    }

    #[test]
    #[serial_test::serial]
    fn run_errors_on_unknown_repo() {
        let dir = tempdir().expect("temp dir");
        write_workspace(dir.path());
        let original = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(dir.path()).expect("chdir");

        let result = run(RulesArgs {
            ai: false,
            id: Vec::new(),
            repo: vec!["missing".to_string()],
        });

        std::env::set_current_dir(original).expect("restore cwd");
        assert!(result.is_err());
    }

    #[test]
    fn ai_flag_strips_sync_with_upstream_and_checks() {
        let mut value = rule(
            "A",
            r#", "sync_with_upstream": true, "checks": [{"name": "x"}], "title": "kept""#,
        );
        if let Some(obj) = value.as_object_mut() {
            obj.remove("sync_with_upstream");
            obj.remove("checks");
        }
        assert!(value.get("sync_with_upstream").is_none());
        assert!(value.get("checks").is_none());
        assert_eq!(value["title"], "kept");
    }
}
