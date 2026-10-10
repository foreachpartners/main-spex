#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::commands::check::fixtures::scoped_file;
    use tempfile::tempdir;

    #[test]
    fn provided_files_resolves_existing_relative_paths() {
        let dir = tempdir().expect("temp dir");
        scoped_file(&dir, "src/api/main.rs", "");
        scoped_file(&dir, "README.md", "");

        let files = vec![
            "src/api/main.rs".to_string(),
            "./README.md".to_string(),
            "missing.rs".to_string(),
        ];
        let resolved = ScopedFile::provided(dir.path(), &files);
        let names: Vec<&Path> = resolved.iter().map(|file| file.rel.as_path()).collect();

        assert_eq!(
            names,
            vec![Path::new("src/api/main.rs"), Path::new("README.md")]
        );
    }
}
