#[cfg(test)]
mod tests {
    use super::super::{file_url, load_yaml_dir, load_yaml_docs, yaml_files};
    use serde::Deserialize;
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

    #[test]
    fn file_url_prefixes_an_absolute_path() {
        assert_eq!(
            file_url(PathBuf::from("/workspace/foo.yaml").as_path()),
            "file:///workspace/foo.yaml"
        );
    }

    #[test]
    fn file_url_encodes_spaces() {
        assert_eq!(
            file_url(PathBuf::from("/workspace/my repo/foo.yaml").as_path()),
            "file:///workspace/my%20repo/foo.yaml"
        );
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
