use std::fmt;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use super::spex_config::SPEX_DIR_NAME;

#[cfg(test)]
#[path = "layout_test.rs"]
mod layout_test;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Layout {
    #[default]
    MultiRepo,
    SingleRepo,
    Monorepo,
}

impl Layout {
    pub fn spex_dir_name(self, project_name: &str) -> String {
        match self {
            Self::MultiRepo => format!("{project_name}-{SPEX_DIR_NAME}"),
            Self::SingleRepo | Self::Monorepo => SPEX_DIR_NAME.to_string(),
        }
    }

    pub fn agent_root(self) -> &'static str {
        match self {
            Self::MultiRepo => ".",
            Self::SingleRepo | Self::Monorepo => "..",
        }
    }

    pub fn is_multi_repo(self) -> bool {
        self == Self::MultiRepo
    }

    pub fn choices() -> String {
        Self::value_variants()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl fmt::Display for Layout {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self.to_possible_value().ok_or(fmt::Error)?;
        formatter.write_str(value.get_name())
    }
}
