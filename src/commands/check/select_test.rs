#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::support::testing;
    use tempfile::tempdir;

    #[test]
    fn select_packages_with_files_picks_owning_and_named_packages() {
        let dir = tempdir().expect("dir");
        let spex_dir = dir.path();
        for (package, file) in [("api", "a.txt"), ("web", "b.txt"), ("cli", "c.txt")] {
            std::fs::create_dir_all(spex_dir.join(package)).expect("package dir");
            std::fs::write(spex_dir.join(package).join(file), "").expect("file");
        }

        let spex = Spex {
            dir: spex_dir.to_path_buf(),
            ..testing::spex(
                ["api", "web", "cli"]
                    .into_iter()
                    .map(|name| Package {
                        path: spex_dir.join(name),
                        ..testing::package(name, name)
                    })
                    .collect(),
            )
        };

        let files = vec!["a.txt".to_string(), "b.txt".to_string()];

        let selected = select_packages(&spex, &[], &files).expect("select");
        let names: Vec<&str> = selected
            .iter()
            .map(|package| package.package_name.as_str())
            .collect();
        assert_eq!(names, vec!["api", "web"]);

        let selected = select_packages(&spex, &["cli".to_string()], &files).expect("select");
        let names: Vec<&str> = selected
            .iter()
            .map(|package| package.package_name.as_str())
            .collect();
        assert_eq!(names, vec!["api", "web", "cli"]);

        assert!(select_packages(&spex, &["ghost".to_string()], &files).is_err());

        let missing = vec!["missing.txt".to_string()];
        assert!(select_packages(&spex, &[], &missing).is_err());
    }
}
