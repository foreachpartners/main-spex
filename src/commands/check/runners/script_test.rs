#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::commands::check::fixtures::write_script;
    use serial_test::serial;
    use tempfile::tempdir;

    #[test]
    fn resolve_script_joins_script_dir_and_reference() {
        let workspace = tempdir().expect("workspace");
        let script_dir = workspace.path().join("spex/rules/x");
        std::fs::create_dir_all(&script_dir).expect("rule script dir");
        std::fs::write(script_dir.join("lint.sh"), "").expect("rule script");

        let resolved = resolve_script(&script_dir, "lint.sh").expect("resolved");
        assert_eq!(resolved, script_dir.join("lint.sh"));
    }

    #[test]
    fn resolve_script_returns_none_when_missing() {
        let workspace = tempdir().expect("workspace");
        let script_dir = workspace.path().join("spex/rules/x");
        std::fs::create_dir_all(&script_dir).expect("rule script dir");

        assert!(resolve_script(&script_dir, "lint.sh").is_none());
    }

    #[test]
    #[serial(script_exec)]
    fn script_run_once_reports_a_hit_on_failure() {
        let workspace = tempdir().expect("workspace");
        let repo = workspace.path().join("api");
        let spex = workspace.path().join("spex");
        std::fs::create_dir_all(&repo).expect("repo");
        let program = spex.join("rules/commit-message-format/sample.sh");
        write_script(&program, "#!/bin/sh\necho boom\nexit 1\n");

        let hits = run_script_once(&program, &repo, &spex).expect("script outcome");

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].message, "boom");
    }

    #[test]
    #[serial(script_exec)]
    fn per_file_script_passes_files_as_arguments() {
        let workspace = tempdir().expect("workspace");
        let repo = workspace.path().join("api");
        let spex = workspace.path().join("spex");
        std::fs::create_dir_all(&repo).expect("repo");
        let program = spex.join("rules/writing-style/echo-args.sh");
        write_script(
            &program,
            "#!/bin/sh\n[ \"$#\" -gt 0 ] || exit 0\necho \"$1\"\nexit 1\n",
        );

        let file = ScopedFile {
            abs: repo.join("README.md"),
            rel: PathBuf::from("README.md"),
        };

        let hits = run_per_file_script(&program, &repo, &spex, &[&file]).expect("script outcome");

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].message, "README.md");
    }

    #[test]
    #[serial(script_exec)]
    fn script_emits_one_hit_per_location_line() {
        let workspace = tempdir().expect("workspace");
        let repo = workspace.path().join("api");
        let spex = workspace.path().join("spex");
        std::fs::create_dir_all(&repo).expect("repo");
        let program = spex.join("rules/writing-style/locations.sh");
        write_script(
            &program,
            "#!/bin/sh\nprintf 'docs/intro.md:40\\tmatches forbidden writing pattern\\n'\nprintf '\\n'\nprintf 'a stray diagnostic\\n'\nexit 1\n",
        );

        let hits = run_script_once(&program, &repo, &spex).expect("script outcome");

        assert_eq!(hits.len(), 2);
        assert_eq!(
            hits[0].location,
            Some(Location::at_line("docs/intro.md", 40))
        );
        assert_eq!(hits[0].message, "matches forbidden writing pattern");
        assert!(hits[1].location.is_none());
        assert_eq!(hits[1].message, "a stray diagnostic");
    }
}
