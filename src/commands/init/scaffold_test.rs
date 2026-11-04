#[cfg(test)]
mod tests_unit {
    use super::super::*;
    use std::fs;

    use tempfile::tempdir;

    use crate::support::bundled_agents::bundled_agents_md;
    use crate::templates::{AGENTS_FILENAME, CLAUDE_FILENAME};

    const SCHEMA: &str = "schemas/spex.schema.json";
    const DOC: &str = "docs/glossary.md";
    const SKILL: &str = ".agents/skills/spex-setup/SKILL.md";

    #[test]
    fn scaffold_writes_managed_content() {
        let spex_dir = tempdir().expect("create spex dir");

        scaffold(spex_dir.path(), spex_dir.path()).expect("scaffold");

        for managed in [SCHEMA, DOC, SKILL] {
            assert!(
                spex_dir.path().join(managed).is_file(),
                "missing managed file `{managed}`"
            );
        }
    }

    #[test]
    fn scaffold_writes_no_rule() {
        let spex_dir = tempdir().expect("create spex dir");

        scaffold(spex_dir.path(), spex_dir.path()).expect("scaffold");

        assert!(
            !spex_dir.path().join("rules").exists(),
            "a rule is taken from the rules library, never scaffolded"
        );
    }

    #[test]
    fn scaffold_writes_every_bundled_skill() {
        let spex_dir = tempdir().expect("create spex dir");

        scaffold(spex_dir.path(), spex_dir.path()).expect("scaffold");

        let skills = BUNDLED_TEMPLATES_DIR
            .get_dir(SKILLS_DIRNAME)
            .expect("bundled skills dir");
        for skill in skills.dirs() {
            let name = skill.path().file_name().expect("skill dir name");
            let bundled = skill
                .get_file(skill.path().join("SKILL.md"))
                .expect("bundled SKILL.md");
            let dest = spex_dir
                .path()
                .join(AGENTS_DIRNAME)
                .join(SKILLS_DIRNAME)
                .join(name)
                .join("SKILL.md");
            assert_eq!(
                fs::read(&dest).expect("read skill"),
                bundled.contents(),
                "`{}` differs from the bundled skill",
                dest.display()
            );
        }
    }

    #[test]
    fn scaffold_writes_bundled_docs_only() {
        let spex_dir = tempdir().expect("create spex dir");

        scaffold(spex_dir.path(), spex_dir.path()).expect("scaffold");

        let docs = BUNDLED_TEMPLATES_DIR
            .get_dir(DOCS_DIRNAME)
            .expect("bundled docs dir");
        for file in docs.files() {
            let name = file.path().file_name().expect("doc file name");
            let dest = spex_dir.path().join(DOCS_DIRNAME).join(name);
            assert!(dest.is_file(), "missing bundled doc `{}`", dest.display());
            assert_eq!(
                fs::read(&dest).expect("read doc"),
                file.contents(),
                "`{}` differs from the bundled doc",
                dest.display()
            );
        }
        for name in ["faq.md", "prd.md", "specification.md", "tutorial.md"] {
            assert!(
                !spex_dir.path().join(DOCS_DIRNAME).join(name).exists(),
                "`{name}` is not a bundled doc and must not scaffold"
            );
        }
    }

    #[test]
    fn scaffold_writes_agents_md_and_claude_link() {
        let spex_dir = tempdir().expect("create spex dir");

        scaffold(spex_dir.path(), spex_dir.path()).expect("scaffold");

        let agents = spex_dir.path().join(AGENTS_FILENAME);
        let expected = bundled_agents_md().expect("bundled AGENTS.md");
        assert_eq!(
            fs::read_to_string(&agents).expect("read AGENTS.md"),
            expected
        );

        #[cfg(unix)]
        {
            let claude = spex_dir.path().join(CLAUDE_FILENAME);
            let target = fs::read_link(&claude).expect("read CLAUDE.md symlink");
            assert_eq!(target, Path::new(AGENTS_FILENAME));
        }
    }

    #[test]
    fn bundled_docs_match_the_single_source_of_truth() {
        let root_docs = include_dir::include_dir!("$CARGO_MANIFEST_DIR/docs");
        let bundled = BUNDLED_TEMPLATES_DIR
            .get_dir(DOCS_DIRNAME)
            .expect("bundled docs dir");

        for file in bundled.files() {
            let name = file.path().file_name().expect("doc file name");
            let source = root_docs.get_file(name).unwrap_or_else(|| {
                panic!(
                    "bundled `docs/{}` has no source in the `docs/` folder",
                    name.to_string_lossy()
                )
            });
            assert_eq!(
                file.contents(),
                source.contents(),
                "bundled `docs/{}` differs from `docs/{}`",
                name.to_string_lossy(),
                name.to_string_lossy()
            );
        }
    }
}
