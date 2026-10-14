use serde::Deserialize;

use super::id::{AcId, SpecId};
use super::spec::{ContractLevel, CoverageIgnore, Statement, Status};

#[cfg(test)]
#[path = "spec_config_test.rs"]
mod spec_config_test;

#[derive(Debug, Deserialize)]
pub struct SpecConfig {
    pub id: SpecId,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub acceptance_criteria: Vec<AcceptanceCriterionConfig>,
    #[serde(default)]
    pub blocked_by: Vec<SpecId>,
    #[serde(default)]
    pub packages: Vec<String>,
    #[serde(default)]
    pub status: Status,
    pub contract: ContractLevel,
    #[serde(default)]
    pub coverage_ignore: Vec<CoverageIgnore>,
}

#[derive(Debug, Deserialize)]
pub struct AcceptanceCriterionConfig {
    pub id: AcId,
    #[serde(flatten)]
    pub statement: Statement,
    #[serde(default)]
    pub is_done: Option<bool>,
}
