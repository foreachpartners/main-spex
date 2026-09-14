#[cfg(test)]
mod tests {
    use super::super::{load_yaml_dir, load_yaml_docs, repo_walker, yaml_files};
    use serde::Deserialize;
    use std::collections::HashSet;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use thiserror::Error;

    #[derive(Debug, Deserialize)]
    struct Item {
        id: String,
    }

    #[derive(Debug, Error)]
    #[error("test error")]
    struct TestError;

    fn walked_files(root: &std::path::Path) -> HashSet<PathBuf> {
        repo_walker(root)
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_some_and(|kind| kind.is_file()))
            .map(|entry| {
                entry
                    .path()
                    .strip_prefix(root)
                    .unwrap_or(entry.path())
                    .to_path_buf()
            })
            .collect()
    }

    #[test]
    fn honors_gitignore_without_a_git_directory() {
        let dir = tempdir().expect("temp dir");
        std::fs::write(dir.path().join(".gitignore"), "ignored.txt\n").expect("gitignore");
        std::fs::write(dir.path().join("ignored.txt"), "").expect("ignored");
        std::fs::write(dir.path().join("kept.txt"), "").expect("kept");

        let files = walked_files(dir.path());

        assert!(files.contains(&PathBuf::from("kept.txt")));
        assert!(!files.contains(&PathBuf::from("ignored.txt")));
    }

    #[test]
    fn skips_dependency_lock_files() {
        let dir = tempdir().expect("temp dir");
        std::fs::write(dir.path().join("Cargo.lock"), "").expect("cargo lock");
        std::fs::write(dir.path().join("package-lock.json"), "").expect("npm lock");
        std::fs::write(dir.path().join("Cargo.toml"), "").expect("cargo manifest");

        let files = walked_files(dir.path());

        assert!(files.contains(&PathBuf::from("Cargo.toml")));
        assert!(!files.contains(&PathBuf::from("Cargo.lock")));
        assert!(!files.contains(&PathBuf::from("package-lock.json")));
    }

    #[test]
    fn surfaces_dotfiles_and_skips_git_directory() {
        let dir = tempdir().expect("temp dir");
        std::fs::create_dir_all(dir.path().join(".git")).expect("git dir");
        std::fs::write(dir.path().join(".git/config"), "").expect("git config");
        std::fs::write(dir.path().join(".env"), "").expect("env");

        let files = walked_files(dir.path());

        assert!(files.contains(&PathBuf::from(".env")));
        assert!(!files.iter().any(|path| path.starts_with(".git")));
    }

    #[test]
    fn load_yaml_dir_reads_nested_subdirectories() {
        let dir = tempdir().expect("temp dir");
        std::fs::write(dir.path().join("top.yaml"), "id: TOP\n").expect("top");
        std::fs::create_dir_all(dir.path().join("frontend")).expect("frontend dir");
        std::fs::write(dir.path().join("frontend/nested.yaml"), "id: NESTED\n").expect("nested");

        let items: Vec<(Item, String, String)> =
            load_yaml_dir(dir.path(), || TestError, |_| TestError, |_| TestError).expect("load");
        let mut ids: Vec<&str> = items.iter().map(|(item, _, _)| item.id.as_str()).collect();
        ids.sort();

        assert_eq!(ids, vec!["NESTED", "TOP"]);
        assert!(
            items
                .iter()
                .any(|(item, path, _)| item.id == "NESTED" && path == "frontend/nested.yaml")
        );
    }

    #[test]
    fn load_yaml_dir_reads_files_two_levels_deep() {
        let dir = tempdir().expect("temp dir");
        std::fs::create_dir_all(dir.path().join("frontend/deep")).expect("nested dirs");
        std::fs::write(dir.path().join("frontend/deep/nested.yaml"), "id: NESTED\n")
            .expect("nested");

        let items: Vec<(Item, String, String)> =
            load_yaml_dir(dir.path(), || TestError, |_| TestError, |_| TestError).expect("load");

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].0.id, "NESTED");
        assert_eq!(items[0].1, "frontend/deep/nested.yaml");
    }

    #[test]
    fn load_yaml_docs_reads_yaml_files_recursively() {
        let dir = tempdir().expect("temp dir");
        std::fs::write(dir.path().join("a.yaml"), "id: A\ntitle: A doc\n").expect("write a.yaml");
        std::fs::create_dir_all(dir.path().join("nested")).expect("nested dir");
        std::fs::write(dir.path().join("nested/b.yaml"), "id: B\n").expect("write b.yaml");

        let docs = load_yaml_docs(dir.path(), |_| TestError, |_| TestError).expect("load docs");
        let ids: Vec<&str> = docs
            .iter()
            .map(|(d, _)| d["id"].as_str().unwrap())
            .collect();
        assert_eq!(ids, vec!["A", "B"]);
        assert_eq!(docs[0].0["title"], "A doc");
        assert_eq!(docs[0].1, "a.yaml");
        assert_eq!(docs[1].1, "nested/b.yaml");
    }

    #[test]
    fn load_yaml_docs_on_missing_dir_is_empty() {
        let dir = tempdir().expect("temp dir");
        let docs = load_yaml_docs(&dir.path().join("missing"), |_| TestError, |_| TestError)
            .expect("load docs");
        assert!(docs.is_empty());
    }

    #[test]
    fn yaml_files_lists_nested_subdirectories() {
        let dir = tempdir().expect("temp dir");
        std::fs::write(dir.path().join("top.yaml"), "").expect("top");
        std::fs::create_dir_all(dir.path().join("backend/deep")).expect("backend dir");
        std::fs::write(dir.path().join("backend/nested.yaml"), "").expect("nested");
        std::fs::write(dir.path().join("backend/deep/deeper.yaml"), "").expect("deeper");

        let files = yaml_files(dir.path());

        assert_eq!(
            files,
            vec![
                dir.path().join("backend/deep/deeper.yaml"),
                dir.path().join("backend/nested.yaml"),
                dir.path().join("top.yaml"),
            ]
        );
    }
}
