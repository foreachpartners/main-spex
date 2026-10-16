#[cfg(test)]
mod tests {
    use super::super::{Exclude, PackageFiles};
    use crate::error::describe;
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    fn exclude(root: &Path, patterns: &[&str]) -> Exclude {
        let patterns: Vec<String> = patterns.iter().map(ToString::to_string).collect();
        Exclude::new(root, &patterns).expect("valid exclude")
    }

    fn walked_files(root: &Path) -> HashSet<PathBuf> {
        walked_files_excluding(root, &[])
    }

    fn walked_files_excluding(root: &Path, patterns: &[&str]) -> HashSet<PathBuf> {
        PackageFiles::new(root, &exclude(root, patterns))
            .filter_map(Result::ok)
            .map(|path| path.strip_prefix(root).unwrap_or(&path).to_path_buf())
            .collect()
    }

    fn walked_in_order(root: &Path) -> Vec<PathBuf> {
        PackageFiles::new(root, &exclude(root, &[]))
            .filter_map(Result::ok)
            .map(|path| path.strip_prefix(root).unwrap_or(&path).to_path_buf())
            .collect()
    }

    fn write_files(root: &Path, files: &[&str]) {
        for file in files {
            let path = root.join(file);
            std::fs::create_dir_all(path.parent().expect("parent")).expect("parent dir");
            std::fs::write(path, "").expect("file");
        }
    }

    #[test]
    fn yields_files_sorted_by_name_within_each_directory() {
        let dir = tempdir().expect("temp dir");
        for file in [
            "zeta.txt",
            "nested/beta.txt",
            "nested/alpha.txt",
            "a.rs",
            "a/x.rs",
        ] {
            let path = dir.path().join(file);
            std::fs::create_dir_all(path.parent().expect("parent")).expect("parent dir");
            std::fs::write(path, "").expect("file");
        }

        let files = walked_in_order(dir.path());

        assert_eq!(
            files,
            [
                "a/x.rs",
                "a.rs",
                "nested/alpha.txt",
                "nested/beta.txt",
                "zeta.txt"
            ]
            .map(PathBuf::from)
        );
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
    fn yields_files_but_not_directories() {
        let dir = tempdir().expect("temp dir");
        std::fs::create_dir_all(dir.path().join("nested")).expect("nested dir");
        std::fs::write(dir.path().join("nested/file.txt"), "").expect("file");

        let files = walked_files(dir.path());

        assert_eq!(files, HashSet::from([PathBuf::from("nested/file.txt")]));
    }

    #[test]
    fn skips_a_folder_whose_path_matches_an_exclude_with_everything_inside_it() {
        let dir = tempdir().expect("temp dir");
        write_files(
            dir.path(),
            &["vendor/lib.js", "vendor/deep/more.js", "src/vendor/b.js"],
        );

        let files = walked_files_excluding(dir.path(), &["vendor"]);

        assert_eq!(files, HashSet::from([PathBuf::from("src/vendor/b.js")]));
    }

    #[test]
    fn skips_the_files_inside_a_folder_matched_by_a_slash_double_star_exclude() {
        let dir = tempdir().expect("temp dir");
        write_files(
            dir.path(),
            &["vendor/lib.js", "vendor/deep/more.js", "src/vendor/b.js"],
        );

        let files = walked_files_excluding(dir.path(), &["vendor/**"]);

        assert_eq!(files, HashSet::from([PathBuf::from("src/vendor/b.js")]));
    }

    #[test]
    fn skips_a_folder_at_any_depth_with_a_leading_double_star_exclude() {
        let dir = tempdir().expect("temp dir");
        write_files(
            dir.path(),
            &["vendor/lib.js", "src/vendor/b.js", "src/main.rs"],
        );

        let files = walked_files_excluding(dir.path(), &["**/vendor"]);

        assert_eq!(files, HashSet::from([PathBuf::from("src/main.rs")]));
    }

    #[test]
    fn skips_files_matched_by_an_exclude_glob_at_any_depth() {
        let dir = tempdir().expect("temp dir");
        write_files(
            dir.path(),
            &[
                "api.generated.ts",
                "src/models.generated.ts",
                "src/models.ts",
            ],
        );

        let files = walked_files_excluding(dir.path(), &["**/*.generated.ts"]);

        assert_eq!(files, HashSet::from([PathBuf::from("src/models.ts")]));
    }

    #[test]
    fn skips_files_matched_by_any_of_several_exclude_patterns() {
        let dir = tempdir().expect("temp dir");
        write_files(dir.path(), &["a.md", "b.txt", "c.rs"]);

        let files = walked_files_excluding(dir.path(), &["*.md", "*.txt"]);

        assert_eq!(files, HashSet::from([PathBuf::from("c.rs")]));
    }

    #[test]
    fn excludes_nothing_without_patterns() {
        let dir = tempdir().expect("temp dir");
        write_files(dir.path(), &["a.txt", "nested/b.txt"]);

        let files = walked_files_excluding(dir.path(), &[]);

        assert_eq!(files.len(), 2);
    }

    #[test]
    fn matches_a_path_relative_to_the_package_or_under_its_root() {
        let dir = tempdir().expect("temp dir");
        let exclude = exclude(dir.path(), &["vendor/**", "**/*.generated.ts"]);

        assert!(exclude.matches(Path::new("vendor/lib/deep.js")));
        assert!(exclude.matches(&dir.path().join("vendor/lib/deep.js")));
        assert!(exclude.matches(Path::new("src/api.generated.ts")));
        assert!(!exclude.matches(Path::new("src/api.ts")));
        assert!(!exclude.matches(&dir.path().join("src/api.ts")));
    }

    #[test]
    fn rejects_an_exclude_pattern_that_is_not_a_valid_glob() {
        let dir = tempdir().expect("temp dir");

        let error = Exclude::new(dir.path(), &["src/{a,b".to_string()]).expect_err("invalid glob");

        assert!(
            describe(&error).contains("src/{a,b"),
            "{}",
            describe(&error)
        );
    }
}
