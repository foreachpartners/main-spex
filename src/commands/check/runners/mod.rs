use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::rule::Check;

use super::{Finding, ScopedFile};

mod ast_grep;
mod glob;
mod regexp;
mod script;

pub(super) use ast_grep::run_ast_grep;
pub(super) use glob::run_glob;
pub(super) use regexp::run_regexp;
pub(super) use script::{
    locate_script, resolve_script, run_per_file_script, run_script_once, run_workspace_checks,
};

pub(super) type Outcome = core::result::Result<Vec<Hit>, String>;

pub(super) struct Hit {
    location: Option<String>,
    message: String,
}

impl Hit {
    fn new(location: Option<String>, message: impl Into<String>) -> Self {
        Self {
            location,
            message: message.into(),
        }
    }
}

pub(super) fn outcome_findings(rule_id: &str, check: &Check, outcome: Outcome) -> Vec<Finding> {
    match outcome {
        Ok(hits) => hits
            .into_iter()
            .map(|hit| Finding::violation(check, rule_id, hit.location, hit.message))
            .collect(),
        Err(message) => vec![Finding::config_error(rule_id, &check.name, message)],
    }
}

pub(super) fn scope_files<'a>(
    check: &Check,
    files: &'a [ScopedFile],
) -> core::result::Result<Vec<&'a ScopedFile>, String> {
    let include =
        build_globset(&check.include).map_err(|err| format!("invalid include glob: {err}"))?;
    let exclude =
        build_globset(&check.exclude).map_err(|err| format!("invalid exclude glob: {err}"))?;

    Ok(files
        .iter()
        .filter(|file| {
            if let Some(include) = &include
                && !include.is_match(&file.rel)
            {
                return false;
            }
            if let Some(exclude) = &exclude
                && exclude.is_match(&file.rel)
            {
                return false;
            }
            true
        })
        .collect())
}

pub(super) fn build_globset(
    patterns: &[String],
) -> core::result::Result<Option<GlobSet>, globset::Error> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    Ok(Some(builder.build()?))
}
