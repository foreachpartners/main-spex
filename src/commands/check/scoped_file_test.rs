#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::commands::check::fixtures::scoped_file;
    use crate::support::testing;
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

    fn package_excluding(dir: &tempfile::TempDir, patterns: &[&str]) -> Package {
        Package {
            path: dir.path().to_path_buf(),
            exclude: patterns.iter().map(ToString::to_string).collect(),
            ..testing::package("api", "api")
        }
    }

    fn rel_names(files: &[ScopedFile]) -> Vec<&Path> {
        files.iter().map(|file| file.rel.as_path()).collect()
    }

    #[test]
    fn collect_skips_the_files_and_folders_the_package_excludes() {
        let dir = tempdir().expect("temp dir");
        for file in [
            "src/main.rs",
            "src/models.generated.ts",
            "vendor/lib.js",
            "README.md",
        ] {
            scoped_file(&dir, file, "");
        }

        let files = ScopedFile::collect(&package_excluding(&dir, &["vendor", "**/*.generated.ts"]))
            .expect("collect");

        assert_eq!(
            rel_names(&files),
            vec![Path::new("README.md"), Path::new("src/main.rs")]
        );
    }

    #[test]
    fn collect_fails_naming_the_package_when_an_exclude_is_not_a_valid_glob() {
        let dir = tempdir().expect("temp dir");

        let error = ScopedFile::collect(&package_excluding(&dir, &["src/{a,b"]))
            .err()
            .expect("invalid exclude");

        let message = crate::error::describe(&error);
        assert!(message.contains("`api`"), "{message}");
        assert!(message.contains("src/{a,b"), "{message}");
    }

    #[test]
    fn provided_not_excluded_drops_listed_files_the_package_excludes() {
        let dir = tempdir().expect("temp dir");
        scoped_file(&dir, "src/main.rs", "");
        scoped_file(&dir, "vendor/lib.js", "");
        let files = vec!["src/main.rs".to_string(), "vendor/lib.js".to_string()];

        let resolved =
            ScopedFile::provided_not_excluded(&package_excluding(&dir, &["vendor/**"]), &files)
                .expect("provided");

        assert_eq!(rel_names(&resolved), vec![Path::new("src/main.rs")]);
    }
}
