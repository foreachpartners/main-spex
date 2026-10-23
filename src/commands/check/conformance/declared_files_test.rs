#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::super::*;
    use crate::commands::check::finding::Source;
    use crate::rule::Severity;
    use crate::support::testing;

    fn package_with_contracts(name: &str, contracts: Vec<std::path::PathBuf>) -> Package {
        Package {
            contracts,
            ..testing::package(name, name)
        }
    }

    #[test]
    fn validate_declared_files_accepts_existing_prd_and_contracts() {
        let spex_dir = tempdir().expect("create spex dir");
        let prd = spex_dir.path().join("prd.md");
        let contract = spex_dir.path().join("api.openapi.yaml");
        fs::write(&prd, "").expect("write prd");
        fs::write(&contract, "").expect("write contract");
        let packages = vec![package_with_contracts("api", vec![contract])];

        let findings = validate_declared_files(spex_dir.path(), &prd, &packages);

        assert!(findings.is_empty());
    }

    #[test]
    fn validate_declared_files_flags_missing_prd() {
        let spex_dir = tempdir().expect("create spex dir");
        let prd = spex_dir.path().join("docs/prd.md");

        let findings = validate_declared_files(spex_dir.path(), &prd, &[]);

        assert_eq!(findings.len(), 1);
        let finding = &findings[0];
        assert_eq!(finding.severity, Severity::Error);
        assert!(
            matches!(&finding.source, Source::Internal { check, .. } if check == "declared-files-exist")
        );
        assert_eq!(
            finding.location,
            Some(Location::in_file(spex_dir.path().join("spex.yaml")))
        );
        assert_eq!(finding.message, "`prd` file `docs/prd.md` does not exist");
    }

    #[test]
    fn validate_declared_files_flags_each_missing_contract_with_its_package() {
        let spex_dir = tempdir().expect("create spex dir");
        let prd = spex_dir.path().join("prd.md");
        let present = spex_dir.path().join("present.md");
        fs::write(&prd, "").expect("write prd");
        fs::write(&present, "").expect("write contract");
        let packages = vec![
            package_with_contracts(
                "api",
                vec![
                    spex_dir.path().join("contracts/api.openapi.yaml"),
                    present.clone(),
                ],
            ),
            package_with_contracts("frontend", vec![spex_dir.path().join("../figma.md")]),
        ];

        let findings = validate_declared_files(spex_dir.path(), &prd, &packages);

        let messages: Vec<&str> = findings
            .iter()
            .map(|finding| finding.message.as_str())
            .collect();
        assert_eq!(
            messages,
            [
                "`contracts` file `contracts/api.openapi.yaml` of package `api` does not exist",
                "`contracts` file `../figma.md` of package `frontend` does not exist",
            ]
        );
    }

    #[test]
    fn validate_declared_files_flags_a_directory_declared_as_a_file() {
        let spex_dir = tempdir().expect("create spex dir");
        let prd = spex_dir.path().join("docs");
        fs::create_dir(&prd).expect("create docs dir");

        let findings = validate_declared_files(spex_dir.path(), &prd, &[]);

        assert_eq!(findings.len(), 1);
    }
}
