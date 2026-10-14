#[cfg(test)]
mod tests {
    use super::super::*;
    use tempfile::tempdir;

    const USER_SKILL: &str = ".claude/skills/my-skill/SKILL.md";

    fn bundled_agents_md() -> String {
        let bytes = BUNDLED_TEMPLATES_DIR
            .get_file(AGENTS_FILENAME)
            .expect("bundled AGENTS.md")
            .contents();
        String::from_utf8(bytes.to_vec()).expect("utf8 AGENTS.md")
    }

    fn write(root: &Path, relative: &str, contents: &str) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().expect("parent")).expect("create parent dirs");
        fs::write(path, contents).expect("write file");
    }

    #[test]
    fn verify_passes_in_an_empty_agent_root() {
        let root = tempdir().expect("create agent root");

        assert!(verify_claude_skills(root.path()).is_ok());
    }

    #[test]
    fn verify_fails_when_a_claude_skill_has_a_bundled_name() {
        let root = tempdir().expect("create agent root");
        write(root.path(), ".claude/skills/spex-spec/SKILL.md", "mine");

        assert!(verify_claude_skills(root.path()).is_err());
    }

    #[test]
    fn verify_fails_when_a_claude_skill_clashes_with_an_agents_skill() {
        let root = tempdir().expect("create agent root");
        write(root.path(), USER_SKILL, "from claude");
        write(
            root.path(),
            ".agents/skills/my-skill/SKILL.md",
            "from agents",
        );

        assert!(verify_claude_skills(root.path()).is_err());
    }

    #[test]
    fn write_agents_md_writes_the_template_when_nothing_exists() {
        let root = tempdir().expect("create agent root");

        write_agents_md(root.path()).expect("write AGENTS.md");

        assert_eq!(
            fs::read_to_string(root.path().join(AGENTS_FILENAME)).expect("read AGENTS.md"),
            bundled_agents_md()
        );
    }

    #[test]
    fn write_agents_md_keeps_an_existing_agents_md_below_the_marker() {
        let root = tempdir().expect("create agent root");
        write(root.path(), "AGENTS.md", "# My package\n\nuse tabs\n");

        write_agents_md(root.path()).expect("write AGENTS.md");

        let written = fs::read_to_string(root.path().join(AGENTS_FILENAME)).expect("read");
        let template = bundled_agents_md();
        let (managed, _) = template
            .split_once("\n---\n\n# Project")
            .expect("bundled AGENTS.md has the project marker");
        assert!(written.starts_with(managed), "managed section not on top");
        assert!(
            written.ends_with("# My package\n\nuse tabs\n"),
            "existing content not kept"
        );
        assert_eq!(written.matches("# Project").count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn write_agents_md_links_claude_md_to_agents_md() {
        let root = tempdir().expect("create agent root");

        write_agents_md(root.path()).expect("write AGENTS.md");

        assert_eq!(
            fs::read_link(root.path().join(CLAUDE_FILENAME)).expect("read symlink"),
            Path::new(AGENTS_FILENAME)
        );
    }

    #[cfg(unix)]
    #[test]
    fn write_agents_md_moves_an_existing_claude_md_into_agents_md() {
        let root = tempdir().expect("create agent root");
        write(root.path(), "CLAUDE.md", "# Claude notes\n");

        write_agents_md(root.path()).expect("write AGENTS.md");

        let written = fs::read_to_string(root.path().join(AGENTS_FILENAME)).expect("read");
        assert!(written.ends_with("# Claude notes\n"));
        assert_eq!(
            fs::read_link(root.path().join(CLAUDE_FILENAME)).expect("read symlink"),
            Path::new(AGENTS_FILENAME)
        );
    }

    #[cfg(unix)]
    #[test]
    fn write_agents_md_prefers_claude_md_over_agents_md_when_both_exist() {
        let root = tempdir().expect("create agent root");
        write(root.path(), "AGENTS.md", "# Agents notes\n");
        write(root.path(), "CLAUDE.md", "# Claude notes\n");

        write_agents_md(root.path()).expect("write AGENTS.md");

        let written = fs::read_to_string(root.path().join(AGENTS_FILENAME)).expect("read");
        assert!(written.ends_with("# Claude notes\n"));
        assert!(!written.contains("# Agents notes"));
        assert_eq!(
            fs::read_link(root.path().join(CLAUDE_FILENAME)).expect("read symlink"),
            Path::new(AGENTS_FILENAME)
        );
    }

    #[cfg(unix)]
    #[test]
    fn link_claude_dir_links_the_whole_directory_when_absent() {
        let root = tempdir().expect("create agent root");
        fs::create_dir_all(root.path().join(".agents/skills")).expect("create skills");

        link_claude_dir(root.path()).expect("link");

        assert_eq!(
            fs::read_link(root.path().join(CLAUDE_DIRNAME)).expect("read symlink"),
            Path::new(AGENTS_DIRNAME)
        );
    }

    #[cfg(unix)]
    #[test]
    fn link_claude_dir_links_only_skills_into_an_existing_claude_directory() {
        let root = tempdir().expect("create agent root");
        write(root.path(), ".claude/settings.json", "{}");
        fs::create_dir_all(root.path().join(".agents/skills")).expect("create skills");

        link_claude_dir(root.path()).expect("link");

        assert!(root.path().join(".claude/settings.json").is_file());
        assert_eq!(
            fs::read_link(root.path().join(".claude/skills")).expect("read symlink"),
            Path::new("../.agents/skills")
        );
    }

    #[cfg(unix)]
    #[test]
    fn link_claude_dir_leaves_an_existing_link_alone() {
        let root = tempdir().expect("create agent root");
        fs::create_dir_all(root.path().join("elsewhere")).expect("create target");
        std::os::unix::fs::symlink("elsewhere", root.path().join(CLAUDE_DIRNAME))
            .expect("create symlink");

        link_claude_dir(root.path()).expect("link");

        assert_eq!(
            fs::read_link(root.path().join(CLAUDE_DIRNAME)).expect("read symlink"),
            Path::new("elsewhere")
        );
    }

    #[cfg(unix)]
    #[test]
    fn move_claude_skills_moves_them_into_agents_and_leaves_a_skills_link() {
        let root = tempdir().expect("create agent root");
        write(root.path(), USER_SKILL, "mine");
        write(root.path(), ".claude/settings.json", "{}");

        move_claude_skills(root.path()).expect("move");
        link_claude_dir(root.path()).expect("link");

        assert_eq!(
            fs::read_to_string(root.path().join(".agents/skills/my-skill/SKILL.md"))
                .expect("read moved skill"),
            "mine"
        );
        assert!(root.path().join(".claude/settings.json").is_file());
        assert!(
            root.path().join(USER_SKILL).is_file(),
            "the moved skill should resolve through the `.claude/skills` link"
        );
        assert!(
            fs::read_link(root.path().join(".claude/skills")).is_ok(),
            "`.claude/skills` should be a symlink"
        );
    }

    #[test]
    fn move_claude_skills_does_nothing_without_claude_skills() {
        let root = tempdir().expect("create agent root");

        move_claude_skills(root.path()).expect("move");

        assert!(!root.path().join(".agents").exists());
    }
}
