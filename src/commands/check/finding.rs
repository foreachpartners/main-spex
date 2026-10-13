use std::fmt;
use std::path::{Path, PathBuf};

use crate::rule::Severity;
use crate::support::fs::relative_to;

#[cfg(test)]
#[path = "finding_test.rs"]
mod finding_test;

pub(super) struct Finding {
    pub(super) severity: Severity,
    pub(super) source: Source,
    pub(super) location: Option<Location>,
    pub(super) message: String,
}

pub(super) enum Source {
    Rule { rule_id: String, check: String },
    Spec { spec_id: String, check: String },
    Internal { id: String, check: String },
    UnknownRule { rule_id: String },
    UnreachablePackage { name: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Location {
    pub(super) file: PathBuf,
    pub(super) line: Option<usize>,
}

impl Finding {
    pub(super) fn new(
        severity: Severity,
        source: Source,
        location: Option<Location>,
        message: String,
    ) -> Self {
        Self {
            severity,
            source,
            location,
            message,
        }
    }

    pub(super) fn error(source: Source, location: Option<Location>, message: String) -> Self {
        Self::new(Severity::Error, source, location, message)
    }

    pub(super) fn warn(source: Source, location: Option<Location>, message: String) -> Self {
        Self::new(Severity::Warn, source, location, message)
    }

    pub(super) fn relative_to(mut self, base: &Path) -> Self {
        self.location = self.location.map(|location| location.relative_to(base));
        self
    }
}

impl Source {
    pub(super) fn rule(rule_id: &str, check: &str) -> Self {
        Self::Rule {
            rule_id: rule_id.to_string(),
            check: check.to_string(),
        }
    }

    pub(super) fn spec(spec_id: &str, check: &str) -> Self {
        Self::Spec {
            spec_id: spec_id.to_string(),
            check: check.to_string(),
        }
    }

    pub(super) fn internal(id: &str, check: &str) -> Self {
        Self::Internal {
            id: id.to_string(),
            check: check.to_string(),
        }
    }

    pub(super) fn baseline_key(&self) -> Option<(&str, &str)> {
        match self {
            Self::Rule {
                rule_id: owner,
                check,
            }
            | Self::Spec {
                spec_id: owner,
                check,
            }
            | Self::Internal { id: owner, check } => Some((owner, check)),
            Self::UnknownRule { .. } | Self::UnreachablePackage { .. } => None,
        }
    }
}

impl fmt::Display for Source {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Rule {
                rule_id: owner,
                check,
            }
            | Self::Spec {
                spec_id: owner,
                check,
            }
            | Self::Internal { id: owner, check } => write!(formatter, "{owner}/{check}"),
            Self::UnknownRule { rule_id } => write!(formatter, "{rule_id}"),
            Self::UnreachablePackage { name } => write!(formatter, "{name}"),
        }
    }
}

impl Location {
    pub(super) fn in_file(file: impl Into<PathBuf>) -> Self {
        Self {
            file: file.into(),
            line: None,
        }
    }

    pub(super) fn at_line(file: impl Into<PathBuf>, line: usize) -> Self {
        Self {
            file: file.into(),
            line: Some(line),
        }
    }

    pub(super) fn parse(text: &str) -> Self {
        if let Some((file, line)) = text.rsplit_once(':')
            && line.bytes().all(|byte| byte.is_ascii_digit())
            && let Ok(line) = line.parse()
        {
            return Self::at_line(file, line);
        }
        Self::in_file(text)
    }

    pub(super) fn relative_to(self, base: &Path) -> Self {
        Self {
            file: relative_to(&self.file, base).to_path_buf(),
            line: self.line,
        }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.file.display())?;
        match self.line {
            Some(line) => write!(formatter, ":{line}"),
            None => Ok(()),
        }
    }
}
