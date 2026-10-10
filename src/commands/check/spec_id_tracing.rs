use std::collections::{HashMap, HashSet};
use std::fs;

use crate::spec::{AcId, SpecId};
use crate::tracing::annotations;

use super::check::RepoCtx;
use super::finding::{Finding, Location, Source};
use super::scoped_file::ScopedFile;

#[cfg(test)]
#[path = "spec_id_tracing_test.rs"]
mod spec_id_tracing_test;

const RULE_ID: &str = "spex-internal-spec-id-tracing";
const FORMAT_CHECK: &str = "annotation-format";
const ORPHAN_CHECK: &str = "no-orphan-annotations";

pub(super) fn validate_annotations(
    ctx: &RepoCtx,
    is_spex_repo: bool,
    known_ids: &HashMap<SpecId, HashSet<AcId>>,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for file in ctx.candidates {
        if holds_illustrative_annotations(ctx, file, is_spex_repo) {
            continue;
        }
        let Ok(content) = fs::read_to_string(&file.abs) else {
            continue;
        };
        findings.extend(check_format(file, &content));
        findings.extend(check_orphans(file, &content, known_ids));
    }
    findings
}

fn holds_illustrative_annotations(ctx: &RepoCtx, file: &ScopedFile, is_spex_repo: bool) -> bool {
    if file.rel.extension().is_some_and(|ext| ext == "md") {
        return true;
    }
    if file
        .abs
        .starts_with(ctx.spex_dir.join(crate::templates::SCHEMAS_DIRNAME))
    {
        return true;
    }
    is_spex_repo
        && file.rel.components().next().is_some_and(|first| {
            let first = first.as_os_str();
            first == "specs" || first == "rules"
        })
}

fn check_format(file: &ScopedFile, content: &str) -> Vec<Finding> {
    annotations::malformed_lines(content)
        .into_iter()
        .map(|line| {
            finding(
                FORMAT_CHECK,
                Location::at_line(&file.rel, line),
                "malformed spec-id annotation".to_string(),
            )
        })
        .collect()
}

fn check_orphans(
    file: &ScopedFile,
    content: &str,
    known: &HashMap<SpecId, HashSet<AcId>>,
) -> Vec<Finding> {
    let refs: HashSet<(SpecId, Option<AcId>)> = annotations::parse(content)
        .into_iter()
        .map(|annotation| (annotation.spec_id, annotation.ac_id))
        .collect();

    let mut orphans: Vec<(SpecId, Option<AcId>)> = refs
        .into_iter()
        .filter(|(spec_id, ac_id)| match known.get(spec_id) {
            None => true,
            Some(ac_ids) => ac_id.as_ref().is_some_and(|ac_id| !ac_ids.contains(ac_id)),
        })
        .collect();
    orphans.sort();
    orphans
        .into_iter()
        .map(|(spec_id, ac_id)| {
            let message = match ac_id {
                Some(ac_id) if known.contains_key(&spec_id) => format!(
                    "orphan annotation references unknown acceptance criterion `{ac_id}` of spec `{spec_id}`"
                ),
                _ => format!("orphan annotation references unknown spec `{spec_id}`"),
            };
            finding(ORPHAN_CHECK, Location::in_file(&file.rel), message)
        })
        .collect()
}

fn finding(check: &str, location: Location, message: String) -> Finding {
    Finding::error(Source::internal(RULE_ID, check), Some(location), message)
}
