use std::fmt;
use std::sync::LazyLock;

use regex::Regex;

#[cfg(test)]
#[path = "id_test.rs"]
mod id_test;

pub const ID_PATTERN: &str = "^[a-z]+(-[a-z]+)*$";
pub const ID_MAX_LENGTH: usize = 40;

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

pub fn validate(kind: &'static str, value: &str) -> Result<(), InvalidId> {
    if value.len() <= ID_MAX_LENGTH && ID.is_match(value) {
        Ok(())
    } else {
        Err(InvalidId {
            kind,
            value: value.to_string(),
        })
    }
}

macro_rules! id_type {
    ($name:ident, $kind:literal) => {
        #[derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            Hash,
            PartialOrd,
            Ord,
            ::serde::Deserialize,
            ::serde::Serialize,
        )]
        #[serde(try_from = "String")]
        pub struct $name(String);

        impl $name {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl ::std::str::FromStr for $name {
            type Err = $crate::support::id::InvalidId;

            fn from_str(value: &str) -> ::std::result::Result<Self, Self::Err> {
                $crate::support::id::validate($kind, value)?;
                Ok(Self(value.to_string()))
            }
        }

        impl TryFrom<String> for $name {
            type Error = $crate::support::id::InvalidId;

            fn try_from(value: String) -> ::std::result::Result<Self, Self::Error> {
                $crate::support::id::validate($kind, &value)?;
                Ok(Self(value))
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

pub(crate) use id_type;
