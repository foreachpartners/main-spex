#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::config::REPO_CONFIG_FILE;
    use crate::error::describe;
    use crate::spec::CoverageLayer;
    use crate::support::testing;
    use indoc::indoc;
    use tempfile::tempdir;

    fn spex() -> Spex {
        testing::spex(vec![
            testing::repository("api", "../api"),
            testing::repository("web", "../web"),
        ])
    }

    #[test]
    fn validate_scopes_accepts_no_scopes() {
        assert!(spex().validate_scopes(&[]).is_ok());
    }

    #[test]
    fn validate_scopes_accepts_known_repository_short_names() {
        let scopes = ["api".to_string(), "web".to_string()];
        assert!(spex().validate_scopes(&scopes).is_ok());
    }

    #[test]
    fn validate_scopes_rejects_the_first_unknown_short_name() {
        let scopes = ["api".to_string(), "nope".to_string(), "other".to_string()];
        let error = spex().validate_scopes(&scopes).expect_err("unknown scope");
        assert_eq!(
            describe(&error),
            "unknown repository short name `nope` — not listed in spex.yaml"
        );
    }

    fn selected_short_names(names: &[&str]) -> Vec<String> {
        let names: Vec<String> = names.iter().map(|name| name.to_string()).collect();
        spex()
            .select_repositories(&names)
            .expect("select")
            .iter()
            .map(|repo| repo.repository_short_name.clone())
            .collect()
    }

    #[test]
    fn select_repositories_without_names_returns_every_repository() {
        assert_eq!(selected_short_names(&[]), vec!["api", "web"]);
    }

    #[test]
    fn select_repositories_keeps_spex_yaml_order_and_matches_name_or_short_name() {
        assert_eq!(selected_short_names(&["web", "api"]), vec!["api", "web"]);
        assert_eq!(selected_short_names(&["web"]), vec!["web"]);
    }

    #[test]
    fn select_repositories_rejects_the_first_unknown_name() {
        let names = ["api".to_string(), "ghost".to_string(), "other".to_string()];
        let error = spex()
            .select_repositories(&names)
            .expect_err("unknown repo");
        assert_eq!(
            describe(&error),
            "unknown workspace repository `ghost` — not listed in spex.yaml"
        );
    }

    fn write_spec(spex_dir: &std::path::Path, id: &str) {
        std::fs::create_dir_all(spex_dir.join(spec::SPECS_DIR)).expect("specs dir");
        std::fs::write(
            spex_dir.join(spec::SPECS_DIR).join(format!("{id}.yaml")),
            format!(
                "id: {id}\ntitle: {id}\ndescription: {id}\ncontract: single\ncoverage_ignore:\n  - layer: test\n"
            ),
        )
        .expect("write spec");
    }

    fn waiver(spec_id: &str, layer: &str) -> SpecCoverageIgnore {
        serde_yaml_ng::from_str(&format!("spec_id: {spec_id}\nlayer: {layer}\n"))
            .expect("parse waiver")
    }

    #[test]
    fn specs_adds_the_workspace_coverage_ignore_of_each_spec_to_its_own() {
        let dir = tempdir().expect("spex dir");
        write_spec(dir.path(), "place-order");
        write_spec(dir.path(), "list-menu");
        let spex = Spex {
            dir: dir.path().to_path_buf(),
            coverage_ignore: vec![waiver("place-order", "impl")],
            ..testing::spex(Vec::new())
        };

        let specs = spex.specs().expect("specs");

        let layers = |id: &str| -> Vec<CoverageLayer> {
            specs
                .iter()
                .find(|spec| spec.id.as_str() == id)
                .expect("spec")
                .coverage_ignore
                .iter()
                .map(|ignore| ignore.layer)
                .collect()
        };
        assert_eq!(
            layers("place-order"),
            vec![CoverageLayer::Test, CoverageLayer::Impl]
        );
        assert_eq!(layers("list-menu"), vec![CoverageLayer::Test]);
    }

    #[test]
    fn with_repo_configs_merges_every_repository_file_into_the_workspace() {
        let workspace = tempdir().expect("workspace");
        let api = workspace.path().join("api");
        let web = workspace.path().join("web");
        let bare = workspace.path().join("bare");
        for dir in [&api, &web, &bare] {
            std::fs::create_dir_all(dir).expect("repo dir");
        }
        std::fs::write(
            api.join(REPO_CONFIG_FILE),
            indoc! {"
                coverage_ignore:
                  - spec_id: from-api
                    layer: impl
                ignored_errors:
                  - rule_id: writing-style
                    check: no-cyrillic
                    files: [api.md]
            "},
        )
        .expect("write api config");
        std::fs::write(
            web.join(REPO_CONFIG_FILE),
            "coverage_ignore:\n  - spec_id: from-web\n    layer: test\n",
        )
        .expect("write web config");
        let mut api_repo = testing::repository("api", "api");
        api_repo.path = api;
        api_repo.ignored_errors = serde_yaml_ng::from_str(indoc! {"
            - rule_id: writing-style
              check: no-todo
        "})
        .expect("parse baseline");
        let mut web_repo = testing::repository("web", "web");
        web_repo.path = web;
        let mut bare_repo = testing::repository("bare", "bare");
        bare_repo.path = bare;
        let spex = Spex {
            coverage_ignore: vec![waiver("from-spex-yaml", "contract")],
            ..testing::spex(vec![api_repo, web_repo, bare_repo])
        };

        let spex = spex.with_repo_configs().expect("merge repository configs");

        let spec_ids: Vec<&str> = spex
            .coverage_ignore
            .iter()
            .map(|waiver| waiver.spec_id.as_str())
            .collect();
        assert_eq!(spec_ids, vec!["from-spex-yaml", "from-api", "from-web"]);
        let checks = |name: &str| -> Vec<String> {
            spex.repositories
                .iter()
                .find(|repo| repo.repository_name == name)
                .expect("repository")
                .ignored_errors
                .iter()
                .map(|entry| entry.check.clone())
                .collect()
        };
        assert_eq!(checks("api"), vec!["no-todo", "no-cyrillic"]);
        assert!(checks("web").is_empty());
        assert!(checks("bare").is_empty());
    }

    #[test]
    fn with_repo_configs_fails_when_a_repository_file_is_invalid() {
        let repo_dir = tempdir().expect("repo dir");
        std::fs::write(
            repo_dir.path().join(REPO_CONFIG_FILE),
            "coverage_ignore: 3\n",
        )
        .expect("write repo config");
        let mut repo = testing::repository("api", "api");
        repo.path = repo_dir.path().to_path_buf();

        let error = testing::spex(vec![repo])
            .with_repo_configs()
            .err()
            .expect("invalid repo config");

        let message = describe(&error);
        assert!(message.starts_with("could not parse `"), "{message}");
        assert!(message.contains(REPO_CONFIG_FILE), "{message}");
    }
}
