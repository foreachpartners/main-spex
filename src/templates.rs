use include_dir::{Dir, include_dir};

pub static BUNDLED_TEMPLATES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

pub const SCHEMAS_DIRNAME: &str = "schemas";
pub const SPEC_SCHEMA_FILENAME: &str = "spec.schema.json";
pub const RULE_SCHEMA_FILENAME: &str = "rule.schema.json";
pub const DOCS_DIRNAME: &str = "docs";
pub const SKILLS_DIRNAME: &str = "skills";
pub const SPECS_LIBRARY_DIRNAME: &str = "specs";
pub const RULES_LIBRARY_DIRNAME: &str = "rules";
pub const AGENTS_DIRNAME: &str = ".agents";
pub const CLAUDE_DIRNAME: &str = ".claude";
pub const AGENTS_FILENAME: &str = "AGENTS.md";
