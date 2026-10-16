#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::config::PACKAGE_CONFIG_FILE;
    use crate::error::describe;
    use crate::spec::CoverageLayer;
    use crate::support::testing;
    use indoc::indoc;
    use tempfile::tempdir;

    fn spex() -> Spex {
        testing::spex(vec![
            testing::package("api", "../api"),
            testing::package("web", "../web"),
        ])
    }

    fn names(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    #[test]
    fn package_short_names_without_names_is_empty() {
        assert!(spex().package_short_names(&[]).expect("names").is_empty());
    }

    #[test]
    fn package_short_names_resolves_a_full_name_and_a_short_name_to_the_short_name() {
        let spex = testing::spex(vec![
            Package {
                package_name: "demo-api".to_string(),
                ..testing::package("api", "../api")
            },
            testing::package("web", "../web"),
        ]);

        let short_names = spex
            .package_short_names(&names(&["web", "demo-api"]))
            .expect("names");

        assert_eq!(short_names, vec!["api", "web"]);
    }

    #[test]
    fn package_short_names_rejects_the_first_unknown_name() {
        let error = spex()
            .package_short_names(&names(&["api", "nope", "other"]))
            .expect_err("unknown package");
        assert_eq!(
            describe(&error),
            "unknown package `nope` — not listed in spex.yaml"
        );
    }

    fn selected_short_names(names: &[&str]) -> Vec<String> {
        let names: Vec<String> = names.iter().map(|name| name.to_string()).collect();
        spex()
            .select_packages(&names)
            .expect("select")
            .iter()
            .map(|package| package.package_short_name.clone())
            .collect()
    }

    #[test]
    fn select_packages_without_names_returns_every_package() {
        assert_eq!(selected_short_names(&[]), vec!["api", "web"]);
    }

    #[test]
    fn select_packages_keeps_spex_yaml_order_and_matches_name_or_short_name() {
        assert_eq!(selected_short_names(&["web", "api"]), vec!["api", "web"]);
        assert_eq!(selected_short_names(&["web"]), vec!["web"]);
    }

    #[test]
    fn select_packages_rejects_the_first_unknown_name() {
        let names = ["api".to_string(), "ghost".to_string(), "other".to_string()];
        let error = spex().select_packages(&names).expect_err("unknown package");
        assert_eq!(
            describe(&error),
            "unknown package `ghost` — not listed in spex.yaml"
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
    fn specs_adds_the_project_coverage_ignore_of_each_spec_to_its_own() {
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
    fn with_package_configs_merges_every_package_file_into_the_project() {
        let project = tempdir().expect("project");
        let api = project.path().join("api");
        let web = project.path().join("web");
        let bare = project.path().join("bare");
        for dir in [&api, &web, &bare] {
            std::fs::create_dir_all(dir).expect("package dir");
        }
        std::fs::write(
            api.join(PACKAGE_CONFIG_FILE),
            indoc! {"
                coverage_ignore:
                  - spec_id: from-api
                    layer: impl
                check_ignore:
                  - rule_id: writing-style
                    check: no-cyrillic
                    files: [api.md]
            "},
        )
        .expect("write api config");
        std::fs::write(
            web.join(PACKAGE_CONFIG_FILE),
            "coverage_ignore:\n  - spec_id: from-web\n    layer: test\n",
        )
        .expect("write web config");
        let mut api_package = testing::package("api", "api");
        api_package.path = api;
        api_package.check_ignore = serde_yaml_ng::from_str(indoc! {"
            - rule_id: writing-style
              check: no-todo
        "})
        .expect("parse baseline");
        let mut web_package = testing::package("web", "web");
        web_package.path = web;
        let mut bare_package = testing::package("bare", "bare");
        bare_package.path = bare;
        let spex = Spex {
            coverage_ignore: vec![waiver("from-spex-yaml", "contract")],
            ..testing::spex(vec![api_package, web_package, bare_package])
        };

        let spex = spex.with_package_configs().expect("merge package configs");

        let spec_ids: Vec<&str> = spex
            .coverage_ignore
            .iter()
            .map(|waiver| waiver.spec_id.as_str())
            .collect();
        assert_eq!(spec_ids, vec!["from-spex-yaml", "from-api", "from-web"]);
        let checks = |name: &str| -> Vec<String> {
            spex.packages
                .iter()
                .find(|package| package.package_name == name)
                .expect("package")
                .check_ignore
                .iter()
                .map(|entry| entry.check.clone())
                .collect()
        };
        assert_eq!(checks("api"), vec!["no-todo", "no-cyrillic"]);
        assert!(checks("web").is_empty());
        assert!(checks("bare").is_empty());
    }

    #[test]
    fn with_package_configs_fails_when_a_package_file_is_invalid() {
        let package_dir = tempdir().expect("package dir");
        std::fs::write(
            package_dir.path().join(PACKAGE_CONFIG_FILE),
            "coverage_ignore: 3\n",
        )
        .expect("write package config");
        let mut package = testing::package("api", "api");
        package.path = package_dir.path().to_path_buf();

        let error = testing::spex(vec![package])
            .with_package_configs()
            .err()
            .expect("invalid package config");

        let message = describe(&error);
        assert!(message.starts_with("could not parse `"), "{message}");
        assert!(message.contains(PACKAGE_CONFIG_FILE), "{message}");
    }
}
