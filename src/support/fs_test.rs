#[cfg(test)]
mod tests {
    use super::super::{
        file_url, normalize, relative_to, write_embedded_dir, write_embedded_dir_at,
    };
    use crate::templates::BUNDLED_TEMPLATES_DIR;
    use include_dir::{Dir, include_dir};
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    static EMBEDDED: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates/schemas");

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
            file_url(PathBuf::from("/workspace/my package/foo.yaml").as_path()),
            "file:///workspace/my%20package/foo.yaml"
        );
    }

    #[test]
    fn file_url_encodes_characters_that_would_break_the_url() {
        assert_eq!(
            file_url(PathBuf::from("/workspace/proj#2/50%/a?.yaml").as_path()),
            "file:///workspace/proj%232/50%25/a%3F.yaml"
        );
    }

    #[test]
    fn normalize_resolves_parent_and_current_components() {
        assert_eq!(
            normalize(Path::new("/workspace/snake/spex/..")),
            Path::new("/workspace/snake")
        );
        assert_eq!(
            normalize(Path::new("/workspace/demo-spex/./../demo-api")),
            Path::new("/workspace/demo-api")
        );
        assert_eq!(
            normalize(Path::new("/workspace/demo-spex/.")),
            Path::new("/workspace/demo-spex")
        );
    }

    #[test]
    fn normalize_keeps_a_parent_component_it_cannot_resolve() {
        assert_eq!(normalize(Path::new("../api")), Path::new("../api"));
    }

    #[test]
    fn relative_to_drops_the_base_prefix() {
        assert_eq!(
            relative_to(
                Path::new("/workspace/spex/specs/a.yaml"),
                Path::new("/workspace/spex")
            ),
            Path::new("specs/a.yaml")
        );
    }

    #[test]
    fn relative_to_keeps_a_path_outside_the_base() {
        assert_eq!(
            relative_to(Path::new("/elsewhere/a.yaml"), Path::new("/workspace/spex")),
            Path::new("/elsewhere/a.yaml")
        );
    }

    #[test]
    fn write_embedded_dir_mirrors_bundled_paths_under_the_destination() {
        let dest = tempdir().expect("temp dir");

        write_embedded_dir(&EMBEDDED, dest.path()).expect("write embedded dir");

        for file in EMBEDDED.files() {
            assert_eq!(
                fs::read(dest.path().join(file.path())).expect("read written file"),
                file.contents(),
                "`{}` differs from the embedded file",
                file.path().display()
            );
        }
    }

    #[test]
    fn write_embedded_dir_at_writes_the_dir_contents_into_the_destination() {
        let dest = tempdir().expect("temp dir");
        let scripts = BUNDLED_TEMPLATES_DIR
            .get_dir("rules/writing-style")
            .expect("bundled script folder");

        write_embedded_dir_at(scripts, dest.path()).expect("write embedded dir");

        for file in scripts.files() {
            let name = file.path().file_name().expect("file name");
            assert_eq!(
                fs::read(dest.path().join(name)).expect("read written file"),
                file.contents(),
                "`{}` differs from the embedded file",
                file.path().display()
            );
        }
        assert!(!dest.path().join("rules").exists());
    }

    #[cfg(unix)]
    mod unix {
        use super::super::super::{make_files_executable, symlink_if_absent};
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        use std::path::Path;
        use tempfile::tempdir;

        #[test]
        fn symlink_if_absent_creates_the_link() {
            let dir = tempdir().expect("temp dir");
            let link = dir.path().join("CLAUDE.md");

            symlink_if_absent(&link, "AGENTS.md").expect("link");

            assert_eq!(
                fs::read_link(&link).expect("read link"),
                Path::new("AGENTS.md")
            );
        }

        #[test]
        fn symlink_if_absent_keeps_an_existing_file() {
            let dir = tempdir().expect("temp dir");
            let link = dir.path().join("CLAUDE.md");
            fs::write(&link, "own content").expect("existing file");

            symlink_if_absent(&link, "AGENTS.md").expect("link");

            assert_eq!(fs::read_to_string(&link).expect("read file"), "own content");
        }

        #[test]
        fn make_files_executable_covers_nested_files() {
            let dir = tempdir().expect("temp dir");
            fs::create_dir_all(dir.path().join("nested")).expect("nested dir");
            let top = dir.path().join("top.sh");
            let nested = dir.path().join("nested/inner.sh");
            fs::write(&top, "#!/bin/sh\n").expect("top");
            fs::write(&nested, "#!/bin/sh\n").expect("nested");

            make_files_executable(dir.path()).expect("make executable");

            for script in [top, nested] {
                let mode = fs::metadata(&script)
                    .expect("metadata")
                    .permissions()
                    .mode();
                assert_eq!(
                    mode & 0o111,
                    0o111,
                    "`{}` is not executable",
                    script.display()
                );
            }
        }

        #[test]
        fn make_files_executable_ignores_a_missing_directory() {
            let dir = tempdir().expect("temp dir");

            make_files_executable(&dir.path().join("missing")).expect("no-op");
        }
    }
}
