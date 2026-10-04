use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use error_stack::ResultExt;
use regex::Regex;

use crate::config::Config;
use crate::error::{CliError, Result};
use crate::repo_files::RepoFiles;
use crate::spec::{ContractLevel, CoverageLayer, Spec};
use crate::util;

#[cfg(test)]
#[path = "annotations_test.rs"]
mod annotations_test;

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
            .map(|ac| {
                let required = |layer| match layer {
                    CoverageLayer::Contract => spec.ac_contract_required(&ac.id),
                    _ => !spec.ac_ignored(&ac.id, layer),
                };
                AcCoverage {
                    id: ac.id.clone(),
                    text: ac.statement.text().to_string(),
                    is_done: ac.is_done(),
                    missing: CoverageLayer::ALL
                        .into_iter()
                        .filter(|&layer| required(layer) && !self.has_ac(&spec.id, &ac.id, layer))
                        .collect(),
                }
            })
            .collect()
    }

    // Spec-level contract coverage. Under `contract: single` one `@spec
    // contract <spec-id>` annotation anywhere satisfies the whole spec; `none`
    // disables contract coverage; `for-each-ac` has no spec-level requirement —
    // each acceptance criterion carries its own (see `ac_coverage`).
    pub fn contract_missing(&self, spec: &Spec) -> bool {
        spec.contract == ContractLevel::Single
            && spec.contract_required()
            && !self.has_bare(&spec.id, CoverageLayer::Contract)
    }

    // Scans one piece of content for spec-id annotations; a test-only helper for
    // building a `Found` from a fixture string instead of real files on disk.
    #[cfg(test)]
    pub fn scan(content: &str) -> Self {
        let mut found = Self::new();
        found.record_all(Parser::new().annotations(content));
        found
    }

    // Scans every workspace repository except spex itself for spec-id annotations.
    pub fn scan_workspace(spex_dir: &Path, config: &Config) -> Result<Self> {
        let parser = Parser::new();
        let spex_canon = util::canonical(spex_dir);
        let mut found = Self::new();

        for repo in &config.repositories {
            // Interface contracts live inside the spex repository's `contracts/`
            // folder, so scan them explicitly even though the spex dir itself is
            // skipped below; their `@spec contract` annotations are the contract layer.
            for contract in &repo.contracts {
                let contract_path = spex_dir.join(&contract.path);
                if let Ok(content) = fs::read_to_string(&contract_path) {
                    found.record_all(parser.annotations(&content));
                }
            }

            let repo_path = spex_dir.join(&repo.path);
            if !repo_path.is_dir() || util::canonical(&repo_path) == spex_canon {
                continue;
            }
            found.scan_repo(&parser, &repo_path)?;
        }

        Ok(found)
    }

    fn scan_repo(&mut self, parser: &Parser, repo_path: &Path) -> Result<()> {
        for result in RepoFiles::new(repo_path) {
            let path = result.change_context(CliError::Walk)?;
            let Ok(content) = fs::read_to_string(path) else {
                continue;
            };
            self.record_all(parser.annotations(&content));
        }

        Ok(())
    }

    fn record_all(&mut self, annotations: Vec<Annotation>) {
        for annotation in annotations {
            self.record(annotation);
        }
    }

    fn record(&mut self, annotation: Annotation) {
        let Annotation {
            layer,
            spec_id,
            ac_id,
        } = annotation;
        let Some(ac_id) = ac_id else {
            self.bare.entry(spec_id).or_default().insert(layer);
            return;
        };
        if layer == CoverageLayer::Contract {
            self.bare.entry(spec_id.clone()).or_default().insert(layer);
        }
        self.by_ac
            .entry((spec_id, ac_id))
            .or_default()
            .insert(layer);
    }
}

pub struct AcCoverage {
    pub id: String,
    pub text: String,
    pub is_done: bool,
    pub missing: Vec<CoverageLayer>,
}

impl AcCoverage {
    pub fn is_missing(&self, layer: CoverageLayer) -> bool {
        self.missing.contains(&layer)
    }

    pub fn impl_ok(&self) -> bool {
        !self.is_missing(CoverageLayer::Impl)
    }

    pub fn is_ok(&self) -> bool {
        self.missing.is_empty()
    }

    pub fn is_gap(&self) -> bool {
        self.is_done && !self.is_ok()
    }

    pub fn is_pending(&self) -> bool {
        !self.is_done && !self.is_ok()
    }
}

pub struct Annotation {
    pub layer: CoverageLayer,
    pub spec_id: String,
    pub ac_id: Option<String>,
}

pub struct Parser {
    annotation: Regex,
    token: Regex,
}

impl Parser {
    pub fn new() -> Self {
        let id = r"[a-z0-9]+(?:-[a-z0-9]+)*";
        let token = format!(r"{id}(?:#{id})?");
        Self {
            annotation: Regex::new(&format!(
                r"@spec\s+(contract|impl|test)\s+({token}(?:\s*,\s*{token})*)"
            ))
            .expect("valid @spec regex"),
            token: Regex::new(&token).expect("valid token regex"),
        }
    }

    pub fn annotations(&self, content: &str) -> Vec<Annotation> {
        let mut annotations = Vec::new();
        for caps in self.annotation.captures_iter(content) {
            let layer = match &caps[1] {
                "contract" => CoverageLayer::Contract,
                "impl" => CoverageLayer::Impl,
                "test" => CoverageLayer::Test,
                _ => continue,
            };
            for token in self.token.find_iter(&caps[2]) {
                let (spec_id, ac_id) = match token.as_str().split_once('#') {
                    Some((spec_id, ac_id)) => (spec_id, Some(ac_id)),
                    None => (token.as_str(), None),
                };
                annotations.push(Annotation {
                    layer,
                    spec_id: spec_id.to_string(),
                    ac_id: ac_id.map(str::to_string),
                });
            }
        }
        annotations
    }

    pub fn malformed_lines(&self, content: &str) -> Vec<usize> {
        content
            .lines()
            .enumerate()
            .filter(|(_, line)| line.contains("@spec ") && !self.annotation.is_match(line))
            .map(|(index, _)| index + 1)
            .collect()
    }
}
