use strum::IntoEnumIterator;

use crate::spec::{AcId, AcceptanceCriterion, ContractLevel, CoverageLayer, Spec};

use super::annotation_index::AnnotationIndex;

#[cfg(test)]
#[path = "spec_coverage_test.rs"]
mod spec_coverage_test;

pub struct CriterionCoverage {
    pub id: AcId,
    pub text: String,
    pub is_done: bool,
    pub missing: Vec<CoverageLayer>,
}

impl CriterionCoverage {
    fn new(spec: &Spec, criterion: &AcceptanceCriterion, index: &AnnotationIndex) -> Self {
        let required = |layer| match layer {
            CoverageLayer::Contract => spec.ac_contract_required(&criterion.id),
            _ => !spec.ac_ignored(&criterion.id, layer),
        };
        Self {
            id: criterion.id.clone(),
            text: criterion.statement.text().to_string(),
            is_done: criterion.is_done,
            missing: CoverageLayer::iter()
                .filter(|&layer| required(layer) && !index.has_ac(&spec.id, &criterion.id, layer))
                .collect(),
        }
    }

    pub fn is_missing(&self, layer: CoverageLayer) -> bool {
        self.missing.contains(&layer)
    }

    pub fn is_implemented(&self) -> bool {
        !self.is_missing(CoverageLayer::Impl)
    }

    pub fn is_covered(&self) -> bool {
        self.missing.is_empty()
    }

    pub fn is_gap(&self) -> bool {
        self.is_done && !self.is_covered()
    }

    pub fn is_pending(&self) -> bool {
        !self.is_done && !self.is_covered()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecHealth {
    Gap,
    Pending,
    Complete,
}

pub struct SpecCoverage<'a> {
    pub spec: &'a Spec,
    pub contract_missing: bool,
    pub criteria: Vec<CriterionCoverage>,
}

impl<'a> SpecCoverage<'a> {
    pub fn new(spec: &'a Spec, index: &AnnotationIndex) -> Self {
        if !spec.status.is_tracked() {
            return Self {
                spec,
                contract_missing: false,
                criteria: Vec::new(),
            };
        }
        Self {
            spec,
            contract_missing: is_spec_contract_missing(spec, index),
            criteria: spec
                .acceptance_criteria
                .iter()
                .map(|criterion| CriterionCoverage::new(spec, criterion, index))
                .collect(),
        }
    }

    pub fn is_tracked(&self) -> bool {
        self.spec.status.is_tracked()
    }

    pub fn implemented_count(&self) -> usize {
        self.criteria
            .iter()
            .filter(|criterion| criterion.is_implemented())
            .count()
    }

    pub fn is_fully_implemented(&self) -> bool {
        self.implemented_count() == self.criteria.len()
    }

    pub fn has_gap(&self) -> bool {
        self.contract_missing || self.criteria.iter().any(CriterionCoverage::is_gap)
    }

    pub fn has_pending(&self) -> bool {
        self.criteria.iter().any(CriterionCoverage::is_pending)
    }

    pub fn health(&self) -> SpecHealth {
        if self.has_gap() {
            SpecHealth::Gap
        } else if self.has_pending() {
            SpecHealth::Pending
        } else {
            SpecHealth::Complete
        }
    }

    pub fn gapped_criteria(&self) -> Vec<&CriterionCoverage> {
        self.criteria
            .iter()
            .filter(|criterion| criterion.is_gap())
            .collect()
    }

    pub fn pending_criteria(&self) -> Vec<&CriterionCoverage> {
        self.criteria
            .iter()
            .filter(|criterion| criterion.is_pending())
            .collect()
    }
}

// Spec-level contract coverage. Under `contract: single` one `@spec
// contract <spec-id>` annotation anywhere satisfies the whole spec; `none`
// disables contract coverage; `for-each-ac` has no spec-level requirement —
// each acceptance criterion carries its own (see `CriterionCoverage::new`).
fn is_spec_contract_missing(spec: &Spec, index: &AnnotationIndex) -> bool {
    spec.contract == ContractLevel::Single
        && !spec.layer_ignored(CoverageLayer::Contract)
        && !index.has_bare(&spec.id, CoverageLayer::Contract)
}
