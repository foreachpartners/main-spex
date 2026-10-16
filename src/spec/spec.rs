use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use strum::{EnumIter, EnumString, IntoStaticStr};

use super::id::{AcId, SpecId};
use super::spec_config::{AcceptanceCriterionConfig, SpecConfig};
use crate::error::Result;
use crate::support::acceptance_criteria_lines::with_source_lines;
use crate::support::commitment::Commitment;
use crate::support::yaml_dir::load_yaml_dir;

#[cfg(test)]
#[path = "spec_test.rs"]
mod spec_test;

pub const SPECS_DIR: &str = "specs";

#[derive(Debug)]
pub struct Spec {
    pub id: SpecId,
    pub path: PathBuf,
    pub description: String,
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    pub blocked_by: Vec<SpecId>,
    pub packages: Vec<String>,
    pub status: Status,
    pub contract: ContractLevel,
    pub coverage_ignore: Vec<CoverageIgnore>,
    pub package_coverage_ignore: BTreeMap<String, Vec<CoverageIgnore>>,
}

impl Spec {
    pub fn ac_done_total(&self) -> (usize, usize) {
        let done = self
            .acceptance_criteria
            .iter()
            .filter(|ac| ac.is_done)
            .count();
        (done, self.acceptance_criteria.len())
    }

    pub fn package_selected(&self, short_names: &[&str]) -> bool {
        short_names.is_empty()
            || short_names
                .iter()
                .any(|short_name| self.packages.iter().any(|name| name == short_name))
    }

    pub fn ac_contract_required(&self, ac_id: &AcId) -> bool {
        self.contract == ContractLevel::ForEachAc
            && !self.ac_ignored(ac_id, CoverageLayer::Contract)
    }

    pub fn layer_ignored(&self, layer: CoverageLayer) -> bool {
        self.waived(layer, None)
    }

    pub fn ac_ignored(&self, ac_id: &AcId, layer: CoverageLayer) -> bool {
        self.waived(layer, Some(ac_id))
    }

    fn waived(&self, layer: CoverageLayer, ac_id: Option<&AcId>) -> bool {
        waives(&self.coverage_ignore, layer, ac_id)
            || self.every_listed_package_waives(layer, ac_id)
    }

    fn every_listed_package_waives(&self, layer: CoverageLayer, ac_id: Option<&AcId>) -> bool {
        !self.packages.is_empty()
            && self.packages.iter().all(|short_name| {
                self.package_coverage_ignore
                    .get(short_name)
                    .is_some_and(|ignores| waives(ignores, layer, ac_id))
            })
    }

    pub fn load_all(specs_dir: &Path) -> Result<Vec<Self>> {
        let mut specs: Vec<Self> = load_yaml_dir::<SpecConfig>(specs_dir)?
            .into_iter()
            .map(|file| Self::from_config(file.item, file.path, &file.raw))
            .collect();
        specs.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(specs)
    }

    fn from_config(config: SpecConfig, path: PathBuf, raw: &str) -> Self {
        Self {
            id: config.id,
            path,
            description: config.description,
            acceptance_criteria: with_source_lines(
                config.acceptance_criteria,
                raw,
                AcceptanceCriterion::from_config,
            ),
            blocked_by: config.blocked_by,
            packages: config.packages,
            status: config.status,
            contract: config.contract,
            coverage_ignore: config.coverage_ignore,
            package_coverage_ignore: BTreeMap::new(),
        }
    }
}

fn waives(ignores: &[CoverageIgnore], layer: CoverageLayer, ac_id: Option<&AcId>) -> bool {
    ignores
        .iter()
        .any(|ignore| ignore.layer == layer && (ignore.ac.is_none() || ignore.ac.as_ref() == ac_id))
}

#[derive(Debug, Clone)]
pub struct AcceptanceCriterion {
    pub id: AcId,
    pub statement: Statement,
    pub is_done: bool,
    pub line: usize,
}

impl AcceptanceCriterion {
    fn from_config(config: AcceptanceCriterionConfig, line: usize) -> Self {
        Self {
            id: config.id,
            statement: config.statement,
            is_done: config.is_done.unwrap_or(true),
            line,
        }
    }
}

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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoverageIgnore {
    pub layer: CoverageLayer,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ac: Option<AcId>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpecCoverageIgnore {
    pub spec_id: SpecId,
    #[serde(flatten)]
    pub ignore: CoverageIgnore,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, EnumIter, IntoStaticStr)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ContractLevel {
    None,
    Single,
    ForEachAc,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Deserialize,
    Serialize,
    EnumIter,
    EnumString,
    IntoStaticStr,
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
