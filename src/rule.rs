use std::path::Path;

use serde::{Deserialize, Serialize};
use strum::IntoStaticStr;

use crate::commitment::Commitment;
use crate::error::Result;
use crate::util;

pub const RULES_DIR: &str = "rules";
pub const SCRIPTS_DIR: &str = "scripts";

pub type RuleId = String;

#[derive(Debug, Deserialize)]
pub struct Rule {
    pub id: RuleId,
    #[serde(skip, default)]
    pub path: String,
    #[serde(default)]
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    #[serde(default)]
    pub checks: Vec<Check>,
}

impl Rule {
    // Where the rule is defined, relative to the workspace root (e.g.
    // `rules/secret-handling.yaml`).
    pub fn location(&self) -> String {
        format!("{RULES_DIR}/{}", self.path)
    }

    pub fn load_all(rules_dir: &Path) -> Result<Vec<Self>> {
        Ok(util::load_yaml_dir::<Self>(rules_dir)?
            .into_iter()
            .map(|file| {
                let mut rule = file.item;
                rule.path = file.rel_path;
                util::number_criteria(&mut rule.acceptance_criteria, &file.raw, |ac, line| {
                    ac.line = line
                });
                rule
            })
            .collect())
    }
}

// A rule's acceptance criteria are plain `"MUST ..."` strings (unlike a
// spec's, which key the commitment level), so each carries its own text.
#[derive(Debug, Clone)]
pub struct AcceptanceCriterion {
    pub text: String,
    // The 1-indexed source line of this criterion, filled in by `Rule::load_all`
    // from the raw file text (serde tracks no spans).
    pub line: usize,
}

impl<'de> serde::Deserialize<'de> for AcceptanceCriterion {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(AcceptanceCriterion {
            text: String::deserialize(deserializer)?,
            line: 0,
        })
    }
}

impl AcceptanceCriterion {
    // `None` when the text doesn't start with a commitment keyword.
    pub fn commitment(&self) -> Option<Commitment> {
        Commitment::from_text(&self.text)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Check {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(flatten)]
    pub kind: CheckKind,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub severity: Severity,
}

impl Check {
    pub fn message(&self, default: &str) -> String {
        self.description
            .clone()
            .unwrap_or_else(|| default.to_string())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum CheckKind {
    Regexp {
        pattern: String,
        #[serde(default)]
        negate: bool,
    },
    Glob {
        pattern: String,
        #[serde(default)]
        negate: bool,
    },
    AstGrep {
        #[serde(default)]
        pattern: Option<String>,
        #[serde(default)]
        rule: Option<serde_yaml_ng::Value>,
        #[serde(default)]
        language: Option<String>,
        #[serde(default)]
        negate: bool,
    },
    Script {
        script: String,
        #[serde(default)]
        run: RunScope,
    },
}

impl CheckKind {
    pub fn negate(&self) -> bool {
        match self {
            CheckKind::Regexp { negate, .. }
            | CheckKind::Glob { negate, .. }
            | CheckKind::AstGrep { negate, .. } => *negate,
            CheckKind::Script { .. } => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RunScope {
    #[default]
    PerFile,
    PerRepo,
    Workspace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize, IntoStaticStr)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Severity {
    Warn,
    #[default]
    Error,
}
