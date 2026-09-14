use std::collections::HashSet;
use std::path::Path;

use serde::Deserialize;

use crate::spec::SPECS_DIR;
use crate::util::yaml_files;

use super::{Finding, SPEC_ID, Severity, rel};

const DUPLICATE_AC_ID_CHECK: &str = "no-duplicate-ac-ids";

#[derive(Deserialize)]
struct AcIdsOnly {
    #[serde(default)]
    acceptance_criteria: Vec<AcRefOnly>,
}

#[derive(Deserialize)]
struct AcRefOnly {
    id: String,
}

// An acceptance criterion id must be unique within its own spec: annotations
// reference `<spec-id>#<ac-id>`, so a duplicate id within one spec would make
// that reference ambiguous.
pub(super) fn validate_no_duplicate_ac_ids(spex_dir: &Path, findings: &mut Vec<Finding>) {
    for file in yaml_files(&spex_dir.join(SPECS_DIR)) {
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        let Ok(parsed) = serde_yaml_ng::from_str::<AcIdsOnly>(&text) else {
            continue;
        };

        let mut seen: HashSet<String> = HashSet::new();
        for ac in parsed.acceptance_criteria {
            if !seen.insert(ac.id.clone()) {
                findings.push(Finding {
                    severity: Severity::Error,
                    spec_id: SPEC_ID.to_string(),
                    check: DUPLICATE_AC_ID_CHECK.to_string(),
                    location: Some(rel(spex_dir, &file)),
                    message: format!(
                        "duplicate acceptance criterion id `{}` within this spec",
                        ac.id
                    ),
                });
            }
        }
    }
}
