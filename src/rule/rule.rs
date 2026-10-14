use std::path::{Path, PathBuf};

use serde::Deserialize;
use strum::IntoStaticStr;

use super::id::RuleId;
use super::rule_config::RuleConfig;
use crate::error::Result;
use crate::support::acceptance_criteria_lines::with_source_lines;
use crate::support::commitment::Commitment;
use crate::support::yaml_dir::load_yaml_dir;

#[cfg(test)]
#[path = "rule_test.rs"]
mod rule_test;

pub const RULES_DIR: &str = "rules";

#[derive(Debug)]
pub struct Rule {
    pub id: RuleId,
    pub path: PathBuf,
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    pub checks: Vec<Check>,
}

impl Rule {
    pub fn script_dir(&self) -> PathBuf {
        self.path.with_extension("")
    }

    pub fn load_all(rules_dir: &Path) -> Result<Vec<Self>> {
        Ok(load_yaml_dir::<RuleConfig>(rules_dir)?
            .into_iter()
            .map(|file| Self::from_config(file.item, file.path, &file.raw))
            .collect())
    }

    fn from_config(config: RuleConfig, path: PathBuf, raw: &str) -> Self {
        Self {
            id: config.id,
            path,
            acceptance_criteria: with_source_lines(
                config.acceptance_criteria,
                raw,
                |text, line| AcceptanceCriterion { text, line },
            ),
            checks: config.checks,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AcceptanceCriterion {
    pub text: String,
    pub line: usize,
}

impl AcceptanceCriterion {
    pub fn commitment(&self) -> Option<Commitment> {
        Commitment::from_text(&self.text)
    }
}

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
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

    pub fn is_project_scoped(&self) -> bool {
        matches!(
            self,
            CheckKind::Script {
                run: RunScope::Project,
                ..
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RunScope {
    #[default]
    PerFile,
    PerPackage,
    Project,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, IntoStaticStr)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Severity {
    Warn,
    #[default]
    Error,
}
