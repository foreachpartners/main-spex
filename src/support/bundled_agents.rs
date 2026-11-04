use crate::templates::{AGENTS_FILENAME, BUNDLED_TEMPLATES_DIR};

#[cfg(test)]
#[path = "bundled_agents_test.rs"]
mod bundled_agents_test;

const GLOSSARY: &str = include_str!("../../docs/glossary.md");
const GLOSSARY_MARKER: &str = "- [docs/glossary.md](docs/glossary.md) inlined.";

pub fn bundled_agents_md() -> Option<String> {
    let template = BUNDLED_TEMPLATES_DIR
        .get_file(AGENTS_FILENAME)?
        .contents_utf8()?;
    Some(inline_glossary(template, GLOSSARY))
}

fn inline_glossary(template: &str, glossary: &str) -> String {
    let terms = glossary
        .split_once("\n\n")
        .map_or(glossary, |(_, terms)| terms)
        .trim_end();
    template.replacen(GLOSSARY_MARKER, terms, 1)
}
