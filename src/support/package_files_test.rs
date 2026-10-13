#[cfg(test)]
mod tests {
    use super::super::PackageFiles;
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    fn walked_files(root: &Path) -> HashSet<PathBuf> {
        PackageFiles::new(root)
            .filter_map(Result::ok)
            .map(|path| path.strip_prefix(root).unwrap_or(&path).to_path_buf())
            .collect()
    }

    fn walked_in_order(root: &Path) -> Vec<PathBuf> {
        PackageFiles::new(root)
            .filter_map(Result::ok)
            .map(|path| path.strip_prefix(root).unwrap_or(&path).to_path_buf())
            .collect()
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
}
