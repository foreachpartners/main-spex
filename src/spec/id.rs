use std::fmt;
use std::sync::LazyLock;

use derive_more::Display;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "id_test.rs"]
mod id_test;

// Kebab-case: lowercase ASCII words separated by single hyphens, no digits, no
// leading, trailing or repeated hyphens (e.g. `shopping-list`), at most
// `ID_MAX_LENGTH` characters.
const ID_PATTERN: &str = "^[a-z]+(-[a-z]+)*$";
const ID_MAX_LENGTH: usize = 40;

static ID: LazyLock<Regex> = LazyLock::new(|| Regex::new(ID_PATTERN).expect("valid id regex"));

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidId {
    kind: &'static str,
    value: String,
}

impl fmt::Display for InvalidId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid {} `{}`: use lowercase letters separated by single hyphens, at most {ID_MAX_LENGTH} characters",
            self.kind, self.value
        )
    }
}

impl std::error::Error for InvalidId {}

fn validate_id(kind: &'static str, value: &str) -> Result<(), InvalidId> {
    if value.len() <= ID_MAX_LENGTH && ID.is_match(value) {
        Ok(())
    } else {
        Err(InvalidId {
            kind,
            value: value.to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Display, Deserialize, Serialize)]
#[serde(try_from = "String")]
pub struct SpecId(String);

impl SpecId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for SpecId {
    type Error = InvalidId;

    fn try_from(value: String) -> Result<Self, InvalidId> {
        validate_id("spec id", &value)?;
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Display, Deserialize, Serialize)]
#[serde(try_from = "String")]
pub struct AcId(String);

impl AcId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for AcId {
    type Error = InvalidId;

    fn try_from(value: String) -> Result<Self, InvalidId> {
        validate_id("acceptance criterion id", &value)?;
        Ok(Self(value))
    }
}
