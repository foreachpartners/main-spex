use include_dir::{Dir, include_dir};

#[cfg(test)]
#[path = "templates_test.rs"]
mod templates_test;

pub static BUNDLED_TEMPLATES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

pub const SCHEMAS_DIRNAME: &str = "schemas";
pub const DOCS_DIRNAME: &str = "docs";
pub const SKILLS_DIRNAME: &str = "skills";
pub const AGENTS_DIRNAME: &str = ".agents";
pub const CLAUDE_DIRNAME: &str = ".claude";
pub const AGENTS_FILENAME: &str = "AGENTS.md";
pub const CLAUDE_FILENAME: &str = "CLAUDE.md";
