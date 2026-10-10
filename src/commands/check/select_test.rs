#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::support::testing;
    use tempfile::tempdir;

    #[test]
    fn select_repos_with_files_picks_owning_and_named_repos() {
        let dir = tempdir().expect("dir");
        let spex_dir = dir.path();
        for (repo, file) in [("api", "a.txt"), ("web", "b.txt"), ("cli", "c.txt")] {
            std::fs::create_dir_all(spex_dir.join(repo)).expect("repo dir");
            std::fs::write(spex_dir.join(repo).join(file), "").expect("file");
        }

        let spex = Spex {
            dir: spex_dir.to_path_buf(),
            ..testing::spex(
                ["api", "web", "cli"]
                    .into_iter()
                    .map(|name| Repository {
                        path: spex_dir.join(name),
                        ..testing::repository(name, name)
                    })
                    .collect(),
            )
        };

        let files = vec!["a.txt".to_string(), "b.txt".to_string()];

        let selected = select_repos(&spex, &[], &files).expect("select");
        let names: Vec<&str> = selected
            .iter()
            .map(|repo| repo.repository_name.as_str())
            .collect();
        assert_eq!(names, vec!["api", "web"]);

        let selected = select_repos(&spex, &["cli".to_string()], &files).expect("select");
        let names: Vec<&str> = selected
            .iter()
            .map(|repo| repo.repository_name.as_str())
            .collect();
        assert_eq!(names, vec!["api", "web", "cli"]);

        assert!(select_repos(&spex, &["ghost".to_string()], &files).is_err());

        let missing = vec!["missing.txt".to_string()];
        assert!(select_repos(&spex, &[], &missing).is_err());
    }
}
