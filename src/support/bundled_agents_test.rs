#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn inline_glossary_replaces_the_marker_line_with_the_terms_without_the_title() {
        let template = format!("## Glossary\n\n{GLOSSARY_MARKER}\n\n---\n\n# Project\n");

        let inlined = inline_glossary(&template, "# Glossary\n\n- **Spec** — a capability.\n");

        assert_eq!(
            inlined,
            "## Glossary\n\n- **Spec** — a capability.\n\n---\n\n# Project\n"
        );
    }

    #[test]
    fn bundled_agents_md_inlines_the_bundled_glossary() {
        let glossary = BUNDLED_TEMPLATES_DIR
            .get_file("docs/glossary.md")
            .and_then(|file| file.contents_utf8())
            .expect("bundled glossary");
        let (_, terms) = glossary.split_once("\n\n").expect("glossary title");

        let agents = bundled_agents_md().expect("bundled AGENTS.md");

        assert!(agents.contains(terms.trim_end()));
        assert!(!agents.contains(GLOSSARY_MARKER));
        assert!(!agents.contains("\n# Glossary\n"));
    }
}
