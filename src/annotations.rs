use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use error_stack::{Report, ResultExt};
use regex::Regex;
use thiserror::Error;

use crate::config::Config;
use crate::spec::{CoverageLayer, Spec};
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
// without a `#ac-id` (a whole-spec annotation, or any contract annotation,
// since contracts have no per-criterion granularity); `by_ac` holds layers
// found qualified with a specific acceptance criterion.
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
}

// Whether an acceptance criterion has the impl/test annotations `coverage_ignore`
// doesn't waive.
pub struct AcCoverage {
    pub id: String,
    pub impl_required: bool,
    pub impl_present: bool,
    pub test_required: bool,
    pub test_present: bool,
}

impl AcCoverage {
    pub fn is_ok(&self) -> bool {
        (!self.impl_required || self.impl_present) && (!self.test_required || self.test_present)
    }
}

pub fn ac_coverage(spec: &Spec, found: &Found) -> Vec<AcCoverage> {
    spec.acceptance_criteria
        .iter()
        .map(|ac| AcCoverage {
            id: ac.id.clone(),
            impl_required: !spec.ac_ignored(&ac.id, CoverageLayer::Impl),
            impl_present: found.has_ac(&spec.id, &ac.id, CoverageLayer::Impl),
            test_required: !spec.ac_ignored(&ac.id, CoverageLayer::Test),
            test_present: found.has_ac(&spec.id, &ac.id, CoverageLayer::Test),
        })
        .collect()
}

// Contract coverage stays spec-level: `x-spec` has no per-criterion form.
pub fn contract_coverage(spec: &Spec, found: &Found) -> (bool, bool) {
    let required = !spec.layer_ignored(CoverageLayer::Contract);
    let present = found.has_bare(&spec.id, CoverageLayer::Contract);
    (required, present)
}

// Scans one piece of content for spec-id annotations; a test-only helper for
// building a `Found` from a fixture string instead of real files on disk.
#[cfg(test)]
pub fn scan(content: &str) -> Found {
    let mut found = Found::new();
    Scanner::new().scan(content, &mut found);
    found
}

// Scans every workspace repository except spex itself for spec-id annotations.
pub fn scan_workspace(spex_dir: &Path, config: &Config) -> Result<Found> {
    let scanner = Scanner::new();
    let spex_canon = canonical(spex_dir);
    let mut found = Found::new();

    for repo in &config.repositories {
        // Interface contracts live inside the spex repository's `contracts/`
        // folder, so scan them explicitly even though the spex dir itself is
        // skipped below; their `x-spec` annotations are the contract layer.
        for contract in &repo.contracts {
            let contract_path = spex_dir.join(&contract.path);
            if let Ok(content) = fs::read_to_string(&contract_path) {
                scanner.scan(&content, &mut found);
            }
        }

        let repo_path = spex_dir.join(&repo.path);
        if !repo_path.is_dir() || canonical(&repo_path) == spex_canon {
            continue;
        }
        scan_repo(&scanner, &repo_path, &mut found)?;
    }

    Ok(found)
}

fn scan_repo(scanner: &Scanner, repo_path: &Path, found: &mut Found) -> Result<()> {
    for result in util::repo_walker(repo_path) {
        let entry = result.change_context(AnnotationsError::Walk)?;
        if !entry.file_type().is_some_and(|kind| kind.is_file()) {
            continue;
        }
        // Skip files that aren't valid UTF-8 (binaries); they carry no annotations.
        let Ok(content) = fs::read_to_string(entry.path()) else {
            continue;
        };
        scanner.scan(&content, found);
    }

    Ok(())
}

struct Scanner {
    spec: Regex,
    xspec: Regex,
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
            xspec: Regex::new(&format!(r#"x-spec:\s*"?({id})"?"#)).expect("valid x-spec regex"),
            token: Regex::new(&token).expect("valid token regex"),
        }
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
                // Contract has no per-criterion granularity, so any `#ac-id`
                // on a contract annotation is ignored and it always counts bare.
                match token.as_str().split_once('#') {
                    Some((spec_id, ac_id)) if layer != CoverageLayer::Contract => found
                        .by_ac
                        .entry((spec_id.to_string(), ac_id.to_string()))
                        .or_default()
                        .insert(layer),
                    Some((spec_id, _)) => found
                        .bare
                        .entry(spec_id.to_string())
                        .or_default()
                        .insert(layer),
                    None => found
                        .bare
                        .entry(token.as_str().to_string())
                        .or_default()
                        .insert(layer),
                };
            }
        }

        for caps in self.xspec.captures_iter(content) {
            found
                .bare
                .entry(caps[1].to_string())
                .or_default()
                .insert(CoverageLayer::Contract);
        }
    }
}

fn canonical(path: &Path) -> std::path::PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}
