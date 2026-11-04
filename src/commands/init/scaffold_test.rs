#[cfg(test)]
mod tests_unit {
    use super::super::*;
    use std::fs;

    use tempfile::tempdir;

    use crate::support::bundled_agents::bundled_agents_md;
    use crate::templates::{AGENTS_FILENAME, CLAUDE_FILENAME};

    const SCHEMA: &str = "schemas/spex.schema.json";
    const SKILL: &str = ".agents/skills/spex-setup/SKILL.md";

    #[test]
    fn scaffold_writes_managed_content() {
        let spex_dir = tempdir().expect("create spex dir");

        scaffold(spex_dir.path(), spex_dir.path()).expect("scaffold");

        for managed in [SCHEMA, SKILL] {
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
    fn scaffold_writes_no_doc() {
        let spex_dir = tempdir().expect("create spex dir");

        scaffold(spex_dir.path(), spex_dir.path()).expect("scaffold");

        assert!(
            !spex_dir.path().join("docs").exists(),
            "the glossary is inlined in AGENTS.md, so scaffold writes no doc"
        );
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
}
