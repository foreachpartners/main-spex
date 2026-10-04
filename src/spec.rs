use std::path::Path;

use serde::Deserialize;

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

    // Whether contract coverage is required at all: `none` disables it,
    // `single` requires one spec-level `@spec contract <spec-id>`,
    // `for-each-ac` requires one per acceptance criterion (see
    // `ac_contract_required`).
    pub fn contract_required(&self) -> bool {
        match self.contract {
            ContractLevel::None => false,
            ContractLevel::Single | ContractLevel::ForEachAc => {
                !self.layer_ignored(CoverageLayer::Contract)
            }
        }
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
    pub commitment: Commitment,
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

// The commitment level of an acceptance criterion, RFC 2119-style: the key
// itself (`must`, `must_not`, ...) names the level, its value the criterion
// text.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Commitment {
    Must(#[allow(dead_code)] String),
    MustNot(#[allow(dead_code)] String),
    Should(#[allow(dead_code)] String),
    ShouldNot(#[allow(dead_code)] String),
    May(#[allow(dead_code)] String),
}

impl Commitment {
    // RFC 2119 order: MUST, MUST NOT, SHOULD, SHOULD NOT, MAY.
    pub fn rank(&self) -> u8 {
        match self {
            Commitment::Must(_) => 1,
            Commitment::MustNot(_) => 2,
            Commitment::Should(_) => 3,
            Commitment::ShouldNot(_) => 4,
            Commitment::May(_) => 5,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Commitment::Must(_) => "MUST",
            Commitment::MustNot(_) => "MUST NOT",
            Commitment::Should(_) => "SHOULD",
            Commitment::ShouldNot(_) => "SHOULD NOT",
            Commitment::May(_) => "MAY",
        }
    }

    pub fn text(&self) -> &str {
        match self {
            Commitment::Must(text) => text,
            Commitment::MustNot(text) => text,
            Commitment::Should(text) => text,
            Commitment::ShouldNot(text) => text,
            Commitment::May(text) => text,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Active,
    #[serde(rename = "in-progress")]
    InProgress,
    #[default]
    Planned,
    Blocked,
    Deprecated,
}

impl Status {
    pub const ALL: [Status; 5] = [
        Status::Active,
        Status::Planned,
        Status::InProgress,
        Status::Blocked,
        Status::Deprecated,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Status::Active => "active",
            Status::InProgress => "in-progress",
            Status::Planned => "planned",
            Status::Blocked => "blocked",
            Status::Deprecated => "deprecated",
        }
    }

    pub fn is_implemented(self) -> bool {
        matches!(self, Status::Active | Status::Deprecated)
    }
}

// `contract` coverage granularity of a spec: `none` disables contract
// coverage, `single` requires one spec-level annotation, `for-each-ac`
// requires one annotation per acceptance criterion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContractLevel {
    None,
    Single,
    #[serde(rename = "for-each-ac")]
    ForEachAc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CoverageLayer {
    Contract,
    Impl,
    Test,
}

impl CoverageLayer {
    pub const ALL: [CoverageLayer; 3] = [
        CoverageLayer::Contract,
        CoverageLayer::Impl,
        CoverageLayer::Test,
    ];

    pub fn label(self) -> &'static str {
        match self {
            CoverageLayer::Contract => "contract",
            CoverageLayer::Impl => "impl",
            CoverageLayer::Test => "test",
        }
    }
}
