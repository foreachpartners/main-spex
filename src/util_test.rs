#[cfg(test)]
mod tests {
    use super::super::repo_walker;
    use std::collections::HashSet;
    use std::path::PathBuf;
    use tempfile::tempdir;

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
}
