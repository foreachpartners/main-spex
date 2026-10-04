use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use error_stack::{Report, ResultExt};
use regex::Regex;
use thiserror::Error;

use crate::config::Config;
use crate::spec::{ContractLevel, CoverageLayer, Spec};
use crate::util;

#[cfg(test)]
#[path = "annotations_test.rs"]
mod annotations_test;

type Result<T> = core::result::Result<T, Report<AnnotationsError>>;

#[derive(Debug, Error)]
pub enum AnnotationsError {
    #[error("could not walk workspace repository files")]
    Walk,
}

// Spec-id annotations found across the workspace: `bare` holds layers found
// without a `#ac-id` (a whole-spec annotation), `by_ac` holds layers found
// qualified with a specific acceptance criterion. A contract annotation
// anchored to one criterion lands in both: it satisfies that criterion's own
// contract requirement under `contract: for-each-ac`, and still stands for
// the whole spec under `contract: single`.
#[derive(Debug, Default)]
pub struct Found {
    bare: HashMap<String, HashSet<CoverageLayer>>,
    by_ac: HashMap<(String, String), HashSet<CoverageLayer>>,
}

impl Found {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_bare(&self, spec_id: &str, layer: CoverageLayer) -> bool {
        self.bare
            .get(spec_id)
            .is_some_and(|layers| layers.contains(&layer))
    }

    // A whole-spec annotation never satisfies per-criterion coverage — only a
    // `#ac-id`-qualified one does, otherwise a single bare annotation would
    // trivially satisfy every criterion and defeat the point of tracking them.
    fn has_ac(&self, spec_id: &str, ac_id: &str, layer: CoverageLayer) -> bool {
        self.by_ac
            .get(&(spec_id.to_string(), ac_id.to_string()))
            .is_some_and(|layers| layers.contains(&layer))
    }

    pub fn ac_coverage(&self, spec: &Spec) -> Vec<AcCoverage> {
        spec.acceptance_criteria
            .iter()
            .map(|ac| AcCoverage {
                id: ac.id.clone(),
                text: ac.commitment.text().to_string(),
                is_done: ac.is_done(),
                contract_required: spec.ac_contract_required(&ac.id),
                contract_present: self.has_ac(&spec.id, &ac.id, CoverageLayer::Contract),
                impl_required: !spec.ac_ignored(&ac.id, CoverageLayer::Impl),
                impl_present: self.has_ac(&spec.id, &ac.id, CoverageLayer::Impl),
                test_required: !spec.ac_ignored(&ac.id, CoverageLayer::Test),
                test_present: self.has_ac(&spec.id, &ac.id, CoverageLayer::Test),
            })
            .collect()
    }

    // Spec-level contract coverage. Under `contract: single` one `@spec
    // contract <spec-id>` annotation anywhere satisfies the whole spec; `none`
    // disables contract coverage; `for-each-ac` has no spec-level requirement —
    // each acceptance criterion carries its own (see `ac_coverage`).
    pub fn contract_coverage(&self, spec: &Spec) -> (bool, bool) {
        match spec.contract {
            ContractLevel::None => (false, false),
            ContractLevel::Single => (
                spec.contract_required(),
                self.has_bare(&spec.id, CoverageLayer::Contract),
            ),
            ContractLevel::ForEachAc => (false, false),
        }
    }

    // Scans one piece of content for spec-id annotations; a test-only helper for
    // building a `Found` from a fixture string instead of real files on disk.
    #[cfg(test)]
    pub fn scan(content: &str) -> Self {
        let mut found = Self::new();
        Scanner::new().scan(content, &mut found);
        found
    }

    // Scans every workspace repository except spex itself for spec-id annotations.
    pub fn scan_workspace(spex_dir: &Path, config: &Config) -> Result<Self> {
        let scanner = Scanner::new();
        let spex_canon = util::canonical(spex_dir);
        let mut found = Self::new();

        for repo in &config.repositories {
            // Interface contracts live inside the spex repository's `contracts/`
            // folder, so scan them explicitly even though the spex dir itself is
            // skipped below; their `@spec contract` annotations are the contract layer.
            for contract in &repo.contracts {
                let contract_path = spex_dir.join(&contract.path);
                if let Ok(content) = fs::read_to_string(&contract_path) {
                    scanner.scan(&content, &mut found);
                }
            }

            let repo_path = spex_dir.join(&repo.dir);
            if !repo_path.is_dir() || util::canonical(&repo_path) == spex_canon {
                continue;
            }
            scanner.scan_repo(&repo_path, &mut found)?;
        }

        Ok(found)
    }
}

// Whether an acceptance criterion has the impl/test/contract annotations
// `coverage_ignore` doesn't waive. Contract is per-criterion only under
// `contract: for-each-ac`.
pub struct AcCoverage {
    pub id: String,
    pub text: String,
    pub is_done: bool,
    pub contract_required: bool,
    pub contract_present: bool,
    pub impl_required: bool,
    pub impl_present: bool,
    pub test_required: bool,
    pub test_present: bool,
}

impl AcCoverage {
    // Implemented, regardless of whether it's tested yet — what the
    // `X/Y AC` counter tracks, since implementation is the milestone that
    // marks a criterion as built.
    pub fn impl_ok(&self) -> bool {
        !self.impl_required || self.impl_present
    }

    pub fn is_ok(&self) -> bool {
        self.impl_ok()
            && (!self.test_required || self.test_present)
            && (!self.contract_required || self.contract_present)
    }

    // Not yet traced, but not a gap either: the criterion is itself flagged
    // `is_done: false`, so missing impl/test annotations are expected, not a
    // slip — that's what makes an in-progress spec in-progress.
    pub fn is_pending(&self) -> bool {
        !self.is_done && !self.is_ok()
    }
}

struct Scanner {
    spec: Regex,
    token: Regex,
}

impl Scanner {
    fn new() -> Self {
        let id = r"[a-z]+(?:-[a-z]+)*";
        let token = format!(r"{id}(?:#{id})?");
        Self {
            spec: Regex::new(&format!(
                r"@spec\s+(contract|impl|test)\s+({token}(?:\s*,\s*{token})*)"
            ))
            .expect("valid @spec regex"),
            token: Regex::new(&token).expect("valid token regex"),
        }
    }

    fn scan_repo(&self, repo_path: &Path, found: &mut Found) -> Result<()> {
        for result in util::repo_walker(repo_path) {
            let entry = result.change_context(AnnotationsError::Walk)?;
            if !entry.file_type().is_some_and(|kind| kind.is_file()) {
                continue;
            }
            // Skip files that aren't valid UTF-8 (binaries); they carry no annotations.
            let Ok(content) = fs::read_to_string(entry.path()) else {
                continue;
            };
            self.scan(&content, found);
        }

        Ok(())
    }

    fn scan(&self, content: &str, found: &mut Found) {
        for caps in self.spec.captures_iter(content) {
            let layer = match &caps[1] {
                "contract" => CoverageLayer::Contract,
                "impl" => CoverageLayer::Impl,
                "test" => CoverageLayer::Test,
                _ => continue,
            };
            for token in self.token.find_iter(&caps[2]) {
                match token.as_str().split_once('#') {
                    Some((spec_id, ac_id)) => {
                        found
                            .by_ac
                            .entry((spec_id.to_string(), ac_id.to_string()))
                            .or_default()
                            .insert(layer);
                        // A contract annotation anchored to one criterion
                        // still counts for the whole spec, so
                        // `contract: single` is satisfied by it too.
                        if layer == CoverageLayer::Contract {
                            found
                                .bare
                                .entry(spec_id.to_string())
                                .or_default()
                                .insert(layer);
                        }
                    }
                    None => {
                        found
                            .bare
                            .entry(token.as_str().to_string())
                            .or_default()
                            .insert(layer);
                    }
                };
            }
        }
    }
}
