use std::path::Path;

use serde::Deserialize;
use strum::{EnumIter, EnumString, IntoStaticStr};

use crate::commitment::Commitment;
use crate::error::Result;
use crate::util;

#[cfg(test)]
#[path = "spec_test.rs"]
mod spec_test;

pub const SPECS_DIR: &str = "specs";

#[derive(Debug, Deserialize)]
pub struct Spec {
    pub id: String,
    #[serde(skip, default)]
    pub path: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    #[serde(default)]
    pub blocked_by: Vec<String>,
    #[serde(default)]
    pub scope: Vec<String>,
    #[serde(default)]
    pub status: Status,
    pub contract: ContractLevel,
    #[serde(default)]
    pub coverage_ignore: Vec<CoverageIgnore>,
}

impl Spec {
    // Where the spec is defined, relative to the workspace root (e.g.
    // `specs/admin/foo.yaml`).
    pub fn location(&self) -> String {
        format!("{SPECS_DIR}/{}", self.path)
    }

    pub fn ac_done_total(&self) -> (usize, usize) {
        let done = self
            .acceptance_criteria
            .iter()
            .filter(|ac| ac.is_done())
            .count();
        (done, self.acceptance_criteria.len())
    }

    pub fn scope_selected(&self, scopes: &[String]) -> bool {
        scopes.is_empty()
            || scopes
                .iter()
                .any(|short_name| self.scope.contains(short_name))
    }

    // Whether this one acceptance criterion needs its own contract
    // annotation — only under `contract: for-each-ac`, and only when the
    // contract layer (or this criterion's contract requirement specifically)
    // isn't waived by `coverage_ignore`.
    pub fn ac_contract_required(&self, ac_id: &str) -> bool {
        self.contract == ContractLevel::ForEachAc
            && !self.ac_ignored(ac_id, CoverageLayer::Contract)
    }

    // A layer waived for the whole spec, via a `coverage_ignore` entry with
    // no `ac`.
    pub fn layer_ignored(&self, layer: CoverageLayer) -> bool {
        self.coverage_ignore
            .iter()
            .any(|ignore| ignore.layer == layer && ignore.ac.is_none())
    }

    // A layer waived for one acceptance criterion, either because the whole
    // layer is ignored or because this criterion specifically is.
    pub fn ac_ignored(&self, ac_id: &str, layer: CoverageLayer) -> bool {
        self.layer_ignored(layer)
            || self
                .coverage_ignore
                .iter()
                .any(|ignore| ignore.layer == layer && ignore.ac.as_deref() == Some(ac_id))
    }

    pub fn load_all(specs_dir: &Path) -> Result<Vec<Self>> {
        let mut specs: Vec<Self> = util::load_yaml_dir::<Self>(specs_dir)?
            .into_iter()
            .map(|file| {
                let mut spec = file.item;
                spec.path = file.rel_path;
                util::number_criteria(&mut spec.acceptance_criteria, &file.raw, |ac, line| {
                    ac.line = line
                });
                spec
            })
            .collect();
        specs.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(specs)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AcceptanceCriterion {
    pub id: String,
    #[serde(flatten)]
    pub statement: Statement,
    #[serde(default)]
    pub is_done: Option<bool>,
    // The 1-indexed source line of this criterion's `- id: ...` entry, filled
    // in by `load_all` from the raw file text (serde tracks no spans).
    #[serde(skip, default)]
    pub line: usize,
}

impl AcceptanceCriterion {
    // Absent means done: `is_done` only exists to flag a criterion as
    // deliberately not done yet.
    pub fn is_done(&self) -> bool {
        self.is_done.unwrap_or(true)
    }
}

// The statement of an acceptance criterion as written in a spec file: the key
// itself (`must`, `must_not`, ...) names the commitment, its value the
// criterion text.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Statement {
    Must(String),
    MustNot(String),
    Should(String),
    ShouldNot(String),
    May(String),
}

impl Statement {
    pub fn commitment(&self) -> Commitment {
        match self {
            Statement::Must(_) => Commitment::Must,
            Statement::MustNot(_) => Commitment::MustNot,
            Statement::Should(_) => Commitment::Should,
            Statement::ShouldNot(_) => Commitment::ShouldNot,
            Statement::May(_) => Commitment::May,
        }
    }

    pub fn text(&self) -> &str {
        match self {
            Statement::Must(text)
            | Statement::MustNot(text)
            | Statement::Should(text)
            | Statement::ShouldNot(text)
            | Statement::May(text) => text,
        }
    }
}

// A `coverage_ignore` entry: `layer` is required, `ac` is optional. Without
// `ac` it waives that layer for the whole spec (every acceptance criterion);
// with `ac` it waives just that one criterion's impl, test, or (under
// `contract: for-each-ac`) contract annotation. `spex check` verifies the
// entry matches the spec's `contract` setting.
#[derive(Debug, Clone, Deserialize)]
pub struct CoverageIgnore {
    pub layer: CoverageLayer,
    #[serde(default)]
    pub ac: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, EnumIter, IntoStaticStr)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Status {
    Active,
    #[default]
    Planned,
    InProgress,
    Blocked,
    Deprecated,
}

impl Status {
    pub fn label(self) -> &'static str {
        self.into()
    }

    pub fn is_implemented(self) -> bool {
        matches!(self, Status::Active | Status::Deprecated)
    }

    pub fn is_tracked(self) -> bool {
        matches!(self, Status::Active | Status::InProgress)
    }
}

// `contract` coverage granularity of a spec: `none` disables contract
// coverage, `single` requires one spec-level annotation, `for-each-ac`
// requires one annotation per acceptance criterion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, EnumIter, IntoStaticStr)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ContractLevel {
    None,
    Single,
    ForEachAc,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, EnumIter, EnumString, IntoStaticStr,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum CoverageLayer {
    Contract,
    Impl,
    Test,
}

impl CoverageLayer {
    pub fn label(self) -> &'static str {
        self.into()
    }
}
