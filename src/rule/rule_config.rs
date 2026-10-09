use serde::Deserialize;

use super::id::RuleId;
use super::rule::Check;

#[derive(Debug, Deserialize)]
pub struct RuleConfig {
    pub id: RuleId,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    #[serde(default)]
    pub checks: Vec<Check>,
}
