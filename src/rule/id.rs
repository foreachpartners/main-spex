use std::fmt;
use std::sync::LazyLock;

use derive_more::Display;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "id_test.rs"]
mod id_test;

const ID_PATTERN: &str = "^[a-z]+(-[a-z]+)*$";
const ID_MAX_LENGTH: usize = 40;

static ID: LazyLock<Regex> = LazyLock::new(|| Regex::new(ID_PATTERN).expect("valid id regex"));

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidRuleId(String);

impl fmt::Display for InvalidRuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid rule id `{}`: use lowercase letters separated by single hyphens, at most {ID_MAX_LENGTH} characters",
            self.0
        )
    }
}

impl std::error::Error for InvalidRuleId {}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Display, Deserialize, Serialize)]
#[serde(try_from = "String")]
pub struct RuleId(String);

impl RuleId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for RuleId {
    type Error = InvalidRuleId;

    fn try_from(value: String) -> Result<Self, InvalidRuleId> {
        if value.len() <= ID_MAX_LENGTH && ID.is_match(&value) {
            Ok(Self(value))
        } else {
            Err(InvalidRuleId(value))
        }
    }
}
