#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::commands::check::finding::{Location, Source};
    use crate::config::PreexistingError;
    use crate::support::testing;

    fn finding(severity: Severity, check: &str, location: Option<Location>) -> Finding {
        Finding::new(
            severity,
            Source::rule("X", check),
            location,
            "boom".to_string(),
        )
    }

    fn preexisting(check: &str, files: Vec<&str>) -> PreexistingError {
        PreexistingError {
            rule_id: "X".to_string(),
            check: check.to_string(),
            files: files.into_iter().map(str::to_string).collect(),
        }
    }

    fn package_at(path: &str, check_ignore: Vec<PreexistingError>) -> Package {
        Package {
            check_ignore,
            is_spex_dir: path == ".",
            ..testing::package(path, path)
        }
    }

    fn is_baselined_in(
        finding: &Finding,
        baseline: Vec<PreexistingError>,
        file: Option<&str>,
    ) -> bool {
        let package = package_at("../api", baseline);
        let owner = Owner {
            package: &package,
            file: file.map(PathBuf::from),
        };
        is_baselined(finding, &owner)
    }

    #[test]
    fn empty_files_baselines_the_whole_check() {
        let baseline = || vec![preexisting("no-cyrillic", vec![])];
        let located = finding(Severity::Error, "no-cyrillic", None);

        assert!(is_baselined_in(&located, baseline(), Some("README.md")));
        assert!(is_baselined_in(&located, baseline(), None));
    }

    #[test]
    fn listed_files_baseline_only_those_files() {
        let baseline = || vec![preexisting("no-cyrillic", vec!["docs/old.md"])];
        let error = finding(Severity::Error, "no-cyrillic", None);

        assert!(is_baselined_in(&error, baseline(), Some("docs/old.md")));
        assert!(!is_baselined_in(&error, baseline(), Some("docs/new.md")));
        assert!(!is_baselined_in(&error, baseline(), None));
    }

    #[test]
    fn warnings_and_other_checks_are_never_baselined() {
        let baseline = || vec![preexisting("no-cyrillic", vec![])];

        assert!(!is_baselined_in(
            &finding(Severity::Warn, "no-cyrillic", None),
            baseline(),
            Some("README.md")
        ));
        assert!(!is_baselined_in(
            &finding(Severity::Error, "writing", None),
            baseline(),
            Some("README.md")
        ));
    }

    #[test]
    fn a_spec_finding_is_baselined_by_its_spec_id_and_check() {
        let baseline = vec![PreexistingError {
            rule_id: "place-order".to_string(),
            check: "status".to_string(),
            files: Vec::new(),
        }];
        let spec_finding = Finding::error(
            Source::spec("place-order", "status"),
            None,
            "boom".to_string(),
        );

        assert!(is_baselined_in(&spec_finding, baseline, None));
    }

    #[test]
    fn a_finding_without_a_check_is_never_baselined() {
        let baseline = vec![preexisting("", vec![])];
        let unknown_rule = Finding::error(
            Source::UnknownRule {
                rule_id: "X".to_string(),
            },
            None,
            "boom".to_string(),
        );

        assert!(!is_baselined_in(&unknown_rule, baseline, None));
    }

    fn project_owner(
        finding: &Finding,
        packages: &[&Package],
    ) -> Option<(String, Option<PathBuf>)> {
        let target = ReportTarget::Project {
            spex_dir: testing::spex_dir(),
        };
        owner(&target, finding, packages)
            .map(|owner| (owner.package.package_name.clone(), owner.file))
    }

    fn project_finding(location: Option<Location>) -> Finding {
        finding(Severity::Error, "check", location)
    }

    #[test]
    fn a_package_report_owns_its_findings() {
        let api = package_at("../api", vec![]);
        let located = finding(
            Severity::Error,
            "check",
            Some(Location::at_line("src/main.rs", 4)),
        );

        let owned = owner(&ReportTarget::Package(&api), &located, &[]).expect("owner");

        assert_eq!(owned.package.package_name, "../api");
        assert_eq!(owned.file, Some(PathBuf::from("src/main.rs")));
    }

    #[test]
    fn a_project_location_resolves_to_the_package_containing_it() {
        let api = package_at("../api", vec![]);
        let web = package_at("../web", vec![]);
        let located = project_finding(Some(Location::at_line("../web/src/main.ts", 4)));

        assert_eq!(
            project_owner(&located, &[&api, &web]),
            Some(("../web".to_string(), Some(PathBuf::from("src/main.ts"))))
        );
    }

    #[test]
    fn a_project_sibling_location_does_not_belong_to_the_spex_repository() {
        let spex = package_at(".", vec![]);
        let api = package_at("../api", vec![]);
        let located = project_finding(Some(Location::in_file("../api/src/main.rs")));

        assert_eq!(
            project_owner(&located, &[&spex, &api]),
            Some(("../api".to_string(), Some(PathBuf::from("src/main.rs"))))
        );
    }

    #[test]
    fn a_project_spex_dir_location_belongs_to_the_spex_repository() {
        let spex = package_at(".", vec![]);
        let api = package_at("../api", vec![]);
        let located = project_finding(Some(Location::at_line("specs/a.yaml", 2)));

        assert_eq!(
            project_owner(&located, &[&api, &spex]),
            Some((".".to_string(), Some(PathBuf::from("specs/a.yaml"))))
        );
    }

    #[test]
    fn a_project_finding_without_a_location_or_containing_package_has_no_owner() {
        let api = package_at("../api", vec![]);

        assert_eq!(project_owner(&project_finding(None), &[&api]), None);
        assert_eq!(
            project_owner(
                &project_finding(Some(Location::in_file("../web/x.ts"))),
                &[&api]
            ),
            None
        );
    }

    #[test]
    fn drop_baselined_findings_applies_each_targets_owner_baseline() {
        let api = package_at(
            "../api",
            vec![preexisting("no-cyrillic", vec!["src/old.rs"])],
        );
        let error_in = |file: &str| {
            finding(
                Severity::Error,
                "no-cyrillic",
                Some(Location::at_line(file, 9)),
            )
        };
        let mut reports = vec![
            CheckReport {
                target: ReportTarget::Package(&api),
                findings: vec![error_in("src/old.rs"), error_in("src/new.rs")],
            },
            CheckReport {
                target: ReportTarget::Project {
                    spex_dir: testing::spex_dir(),
                },
                findings: vec![
                    error_in("../api/src/old.rs"),
                    error_in("../api/src/new.rs"),
                    error_in("../web/src/old.rs"),
                ],
            },
        ];

        drop_baselined_findings(&mut reports, &[&api]);

        let remaining: Vec<Vec<String>> = reports
            .iter()
            .map(|report| {
                report
                    .findings
                    .iter()
                    .filter_map(|finding| finding.location.as_ref().map(Location::to_string))
                    .collect()
            })
            .collect();
        assert_eq!(
            remaining,
            vec![
                vec!["src/new.rs:9".to_string()],
                vec![
                    "../api/src/new.rs:9".to_string(),
                    "../web/src/old.rs:9".to_string()
                ],
            ]
        );
    }
}
