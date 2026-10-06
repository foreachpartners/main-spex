#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::support::testing;
    use tempfile::tempdir;

    #[test]
    fn select_repos_with_files_picks_owning_and_named_repos() {
        let dir = tempdir().expect("dir");
        let spex = dir.path();
        for (repo, file) in [("api", "a.txt"), ("web", "b.txt"), ("cli", "c.txt")] {
            std::fs::create_dir_all(spex.join(repo)).expect("repo dir");
            std::fs::write(spex.join(repo).join(file), "").expect("file");
        }

        let config = testing::config(vec![
            testing::repository("api", "api"),
            testing::repository("web", "web"),
            testing::repository("cli", "cli"),
        ]);

        let files = vec!["a.txt".to_string(), "b.txt".to_string()];

        // Files in api and web select exactly their owning repositories.
        let selected = select_repos(&config, spex, &[], &files).expect("select");
        let names: Vec<&str> = selected
            .iter()
            .map(|repo| repo.repository_name.as_str())
            .collect();
        assert_eq!(names, vec!["api", "web"]);

        // --repo cli unions in even though no provided file lives there.
        let selected = select_repos(&config, spex, &["cli".to_string()], &files).expect("select");
        let names: Vec<&str> = selected
            .iter()
            .map(|repo| repo.repository_name.as_str())
            .collect();
        assert_eq!(names, vec!["api", "web", "cli"]);

        // An unknown --repo name still errors.
        assert!(select_repos(&config, spex, &["ghost".to_string()], &files).is_err());

        // A --files entry that doesn't exist in any repository errors.
        let missing = vec!["missing.txt".to_string()];
        assert!(select_repos(&config, spex, &[], &missing).is_err());
    }
}
