use std::collections::{HashMap, HashSet};
use std::fs;

use crate::annotations::Parser;

use super::{Finding, RepoCtx, ScopedFile};

#[cfg(test)]
#[path = "spec_id_tracing_test.rs"]
mod spec_id_tracing_test;

// Spec-id annotation checks are built into `spex check` itself rather than
// declared as a rule, since every repository must satisfy them regardless of
// which rules it opts into.
const RULE_ID: &str = "spex-internal-spec-id-tracing";
const FORMAT_CHECK: &str = "annotation-format";
const ORPHAN_CHECK: &str = "no-orphan-annotations";

pub(super) fn validate_annotations(
    ctx: &RepoCtx,
    is_spex_repo: bool,
    known_ids: &HashMap<String, HashSet<String>>,
    findings: &mut Vec<Finding>,
) {
    let parser = Parser::new();
    for file in ctx.candidates {
        if is_excluded(ctx, file, is_spex_repo) {
            continue;
        }
        let Ok(content) = fs::read_to_string(&file.abs) else {
            continue;
        };
        check_format(&parser, file, &content, findings);
        check_orphans(&parser, file, &content, known_ids, findings);
    }
}

// Markdown files carry illustrative example annotations in prose, not real
// references; every workspace's schemas/ directory holds the same kind of
// illustrative content (e.g. a JSON Schema description showing the
// `<spec-id>` grammar placeholder), and the spex workspace repo's own
// specs/, rules/, and scripts/ hold real-looking but illustrative examples
// too.
fn is_excluded(ctx: &RepoCtx, file: &ScopedFile, is_spex_repo: bool) -> bool {
    if file.rel.extension().is_some_and(|ext| ext == "md") {
        return true;
    }
    if file
        .abs
        .starts_with(ctx.spex_dir.join(crate::templates::SCHEMAS_DIR))
    {
        return true;
    }
    is_spex_repo
        && file.rel.components().next().is_some_and(|first| {
            let first = first.as_os_str();
            first == "specs" || first == "rules" || first == "scripts"
        })
}

fn check_format(parser: &Parser, file: &ScopedFile, content: &str, findings: &mut Vec<Finding>) {
    for line in parser.malformed_lines(content) {
        findings.push(finding(
            FORMAT_CHECK,
            Some(format!("{}:{line}", file.rel_str())),
            "malformed spec-id annotation".to_string(),
        ));
    }
}

fn check_orphans(
    parser: &Parser,
    file: &ScopedFile,
    content: &str,
    known: &HashMap<String, HashSet<String>>,
    findings: &mut Vec<Finding>,
) {
    let refs: HashSet<(String, Option<String>)> = parser
        .annotations(content)
        .into_iter()
        .map(|annotation| (annotation.spec_id, annotation.ac_id))
        .collect();

    let mut orphans: Vec<(String, Option<String>)> = refs
        .into_iter()
        .filter(|(spec_id, ac_id)| match known.get(spec_id) {
            None => true,
            Some(ac_ids) => ac_id.as_ref().is_some_and(|ac_id| !ac_ids.contains(ac_id)),
        })
        .collect();
    orphans.sort();
    for (spec_id, ac_id) in orphans {
        let message = match ac_id {
            Some(ac_id) if known.contains_key(&spec_id) => format!(
                "orphan annotation references unknown acceptance criterion `{ac_id}` of spec `{spec_id}`"
            ),
            _ => format!("orphan annotation references unknown spec `{spec_id}`"),
        };
        findings.push(finding(ORPHAN_CHECK, Some(file.rel_str()), message));
    }
}

fn finding(check: &str, location: Option<String>, message: String) -> Finding {
    Finding::error(RULE_ID, check, location, message)
}
