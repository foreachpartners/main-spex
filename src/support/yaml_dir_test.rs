#[cfg(test)]
mod tests {
    use super::super::{load_yaml_dir, yaml_files};
    use serde::Deserialize;
    use tempfile::tempdir;

    #[derive(Debug, Deserialize)]
    struct Item {
        id: String,
    }

    #[test]
    fn load_yaml_dir_reads_nested_subdirectories() {
        let dir = tempdir().expect("temp dir");
        std::fs::write(dir.path().join("top.yaml"), "id: TOP\n").expect("top");
        std::fs::create_dir_all(dir.path().join("frontend")).expect("frontend dir");
        std::fs::write(dir.path().join("frontend/nested.yaml"), "id: NESTED\n").expect("nested");

        let files = load_yaml_dir::<Item>(dir.path()).expect("load");
        let mut ids: Vec<&str> = files.iter().map(|file| file.item.id.as_str()).collect();
        ids.sort();

        assert_eq!(ids, vec!["NESTED", "TOP"]);
        assert!(
            files.iter().any(|file| file.item.id == "NESTED"
                && file.path == dir.path().join("frontend/nested.yaml"))
        );
    }

    #[test]
    fn load_yaml_dir_reads_files_two_levels_deep() {
        let dir = tempdir().expect("temp dir");
        std::fs::create_dir_all(dir.path().join("frontend/deep")).expect("nested dirs");
        std::fs::write(dir.path().join("frontend/deep/nested.yaml"), "id: NESTED\n")
            .expect("nested");

        let files = load_yaml_dir::<Item>(dir.path()).expect("load");

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].item.id, "NESTED");
        assert_eq!(files[0].path, dir.path().join("frontend/deep/nested.yaml"));
    }

    #[test]
    fn load_yaml_dir_reads_generic_json_values_recursively() {
        let dir = tempdir().expect("temp dir");
        std::fs::write(dir.path().join("a.yaml"), "id: A\ntitle: A doc\n").expect("write a.yaml");
        std::fs::create_dir_all(dir.path().join("nested")).expect("nested dir");
        std::fs::write(dir.path().join("nested/b.yaml"), "id: B\n").expect("write b.yaml");

        let docs = load_yaml_dir::<serde_json::Value>(dir.path()).expect("load docs");
        let ids: Vec<&str> = docs
            .iter()
            .map(|doc| doc.item["id"].as_str().unwrap())
            .collect();
        assert_eq!(ids, vec!["A", "B"]);
        assert_eq!(docs[0].item["title"], "A doc");
        assert_eq!(docs[0].path, dir.path().join("a.yaml"));
        assert_eq!(docs[1].path, dir.path().join("nested/b.yaml"));
    }

    #[test]
    fn load_yaml_dir_on_missing_dir_is_empty() {
        let dir = tempdir().expect("temp dir");
        let docs = load_yaml_dir::<serde_json::Value>(&dir.path().join("missing")).expect("load");
        assert!(docs.is_empty());
    }

    #[test]
    fn yaml_files_lists_nested_subdirectories() {
        let dir = tempdir().expect("temp dir");
        std::fs::write(dir.path().join("top.yaml"), "").expect("top");
        std::fs::create_dir_all(dir.path().join("backend/deep")).expect("backend dir");
        std::fs::write(dir.path().join("backend/nested.yaml"), "").expect("nested");
        std::fs::write(dir.path().join("backend/deep/deeper.yaml"), "").expect("deeper");

        let files = yaml_files(dir.path()).expect("list");

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
