use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use error_stack::ResultExt;

use crate::config::spex::Spex;
use crate::error::{CliError, Result};
use crate::spec::{AcId, CoverageLayer, SpecId};
use crate::support::repo_files::RepoFiles;
use crate::tracing::annotations::{self, Annotation};

#[cfg(test)]
#[path = "annotation_index_test.rs"]
mod annotation_index_test;

#[derive(Debug, Default)]
pub struct AnnotationIndex {
    bare: HashMap<SpecId, HashSet<CoverageLayer>>,
    by_ac: HashMap<(SpecId, AcId), HashSet<CoverageLayer>>,
}

impl AnnotationIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_bare(&self, spec_id: &SpecId, layer: CoverageLayer) -> bool {
        self.bare
            .get(spec_id)
            .is_some_and(|layers| layers.contains(&layer))
    }

    pub fn has_ac(&self, spec_id: &SpecId, ac_id: &AcId, layer: CoverageLayer) -> bool {
        self.by_ac
            .get(&(spec_id.clone(), ac_id.clone()))
            .is_some_and(|layers| layers.contains(&layer))
    }

    #[cfg(test)]
    pub fn scan(content: &str) -> Self {
        let mut index = Self::new();
        index.record_all(annotations::parse(content));
        index
    }

    pub fn scan_workspace(spex: &Spex) -> Result<Self> {
        let mut index = Self::new();

        for repo in &spex.repositories {
            for contract_path in &repo.contracts {
                if let Ok(content) = fs::read_to_string(contract_path) {
                    index.record_all(annotations::parse(&content));
                }
            }

            if !repo.path.is_dir() || repo.is_spex_repo {
                continue;
            }
            index.scan_repo(&repo.path)?;
        }

        Ok(index)
    }

    fn scan_repo(&mut self, repo_path: &Path) -> Result<()> {
        for result in RepoFiles::new(repo_path) {
            let path = result.change_context_lazy(|| {
                CliError::Internal("could not walk workspace repository files".into())
            })?;
            let Ok(content) = fs::read_to_string(path) else {
                continue;
            };
            self.record_all(annotations::parse(&content));
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
