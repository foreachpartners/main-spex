use std::collections::{HashMap, HashSet};
use std::fs;

use regex::Regex;

use super::{Finding, RepoCtx, ScopedFile, Severity, rel_str};

#[cfg(test)]
#[path = "spec_id_tracing_test.rs"]
mod spec_id_tracing_test;

// Spec-id annotation checks are built into `spex check` itself rather than
// declared as a rule, since every repository must satisfy them regardless of
// which rules it opts into.
const SPEC_ID: &str = "spex-internal-spec-id-tracing";
const FORMAT_CHECK: &str = "annotation-format";
const ORPHAN_CHECK: &str = "no-orphan-annotations";

pub(super) fn validate_annotations(
    ctx: &RepoCtx,
    is_spex_repo: bool,
    known_ids: &HashMap<String, HashSet<String>>,
    findings: &mut Vec<Finding>,
) {
    let checker = Checker::new();
    for file in ctx.candidates {
        if is_excluded(ctx, file, is_spex_repo) {
            continue;
        }
        let Ok(content) = fs::read_to_string(&file.abs) else {
            continue;
        };
        checker.check_format(file, &content, findings);
        checker.check_orphans(file, &content, known_ids, findings);
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
    if is_spex_repo {
        if let Some(first) = file.rel.components().next() {
            let first = first.as_os_str();
            if first == "specs" || first == "rules" || first == "scripts" {
                return true;
            }
        }
    }
    false
}

struct Checker {
    spec_line: Regex,
    spec_extract: Regex,
    token: Regex,
}

impl Checker {
    fn new() -> Self {
        // Loosened past the strict `[a-z]+(?:-[a-z]+)*` id grammar (digits
        // disallowed) so a match always consumes a full token instead of
        // silently stopping short at the first digit — e.g. `delivery-time2`
        // must be captured whole, not truncated into the real id
        // `delivery-time` plus a dangling `2`.
        let id = r"[a-z0-9]+(?:-[a-z0-9]+)*";
        let token = format!(r"{id}(?:#{id})?");
        Self {
            spec_line: Regex::new(&format!(
                r"@spec\s+(?:contract|impl|test)\s+{token}(?:\s*,\s*{token})*"
            ))
            .expect("valid spec-line regex"),
            spec_extract: Regex::new(&format!(
                r"@spec\s+(?:contract|impl|test)\s+({token}(?:\s*,\s*{token})*)"
            ))
            .expect("valid spec-extract regex"),
            token: Regex::new(&token).expect("valid token regex"),
        }
    }

    // Flags a line that opens an annotation (`@spec `) but does not match its
    // full grammar anywhere in the line.
    fn check_format(&self, file: &ScopedFile, content: &str, findings: &mut Vec<Finding>) {
        for (index, line) in content.lines().enumerate() {
            if !line.contains("@spec ") {
                continue;
            }
            if !self.spec_line.is_match(line) {
                findings.push(finding(
                    FORMAT_CHECK,
                    Some(format!("{}:{}", rel_str(file), index + 1)),
                    "malformed spec-id annotation".to_string(),
                ));
            }
        }
    }

    // Flags a Spec ID referenced by an annotation that matches no known spec,
    // and an `#<ac-id>` suffix that matches no acceptance criterion
    // of its spec; deduplicated per file since a repeated reference in the
    // same file isn't a separate problem.
    fn check_orphans(
        &self,
        file: &ScopedFile,
        content: &str,
        known: &HashMap<String, HashSet<String>>,
        findings: &mut Vec<Finding>,
    ) {
        let mut refs: HashSet<(String, Option<String>)> = HashSet::new();
        for caps in self.spec_extract.captures_iter(content) {
            for token in self.token.find_iter(&caps[1]) {
                let (spec_id, ac_id) = match token.as_str().split_once('#') {
                    Some((id, ac)) => (id.to_string(), Some(ac.to_string())),
                    None => (token.as_str().to_string(), None),
                };
                refs.insert((spec_id, ac_id));
            }
        }

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
            findings.push(finding(ORPHAN_CHECK, Some(rel_str(file)), message));
        }
    }
}

fn finding(check: &str, location: Option<String>, message: String) -> Finding {
    Finding {
        severity: Severity::Error,
        spec_id: SPEC_ID.to_string(),
        check: check.to_string(),
        location,
        message,
    }
}
