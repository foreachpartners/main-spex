use std::collections::BTreeSet;
use std::process::ExitCode;

use strum::IntoEnumIterator;

use crate::spec::{Spec, Status};

use super::annotation_index::AnnotationIndex;
use super::spec_coverage::SpecCoverage;

#[cfg(test)]
#[path = "report_test.rs"]
mod report_test;

pub struct CoverageReport<'a> {
    pub specs: Vec<SpecCoverage<'a>>,
}

impl<'a> CoverageReport<'a> {
    pub fn new(specs: &'a [Spec], index: &AnnotationIndex) -> Self {
        Self {
            specs: specs
                .iter()
                .map(|spec| SpecCoverage::new(spec, index))
                .collect(),
        }
    }

    pub fn count(&self, status: Status) -> usize {
        self.specs
            .iter()
            .filter(|coverage| coverage.spec.status == status)
            .count()
    }

    pub fn has_tracked_specs(&self) -> bool {
        self.specs.iter().any(SpecCoverage::is_tracked)
    }

    pub fn status_groups(&self) -> Vec<(Status, Vec<&SpecCoverage<'a>>)> {
        Status::iter()
            .filter_map(|status| {
                let group: Vec<&SpecCoverage> = self
                    .specs
                    .iter()
                    .filter(|coverage| coverage.spec.status == status)
                    .collect();
                (!group.is_empty()).then_some((status, group))
            })
            .collect()
    }

    pub fn gapped(&self) -> Vec<&SpecCoverage<'a>> {
        self.specs
            .iter()
            .filter(|coverage| coverage.has_gap())
            .collect()
    }

    pub fn gapped_by_package(&self) -> Vec<(&str, Vec<&SpecCoverage<'a>>)> {
        let gapped = self.gapped();
        let packages: BTreeSet<&str> = gapped
            .iter()
            .flat_map(|coverage| coverage.spec.packages.iter().map(String::as_str))
            .collect();

        packages
            .into_iter()
            .map(|package| {
                let in_package = gapped
                    .iter()
                    .filter(|coverage| coverage.spec.packages.iter().any(|name| name == package))
                    .copied()
                    .collect();
                (package, in_package)
            })
            .collect()
    }

    pub fn exit_code(&self) -> ExitCode {
        if self.gapped().is_empty() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        }
    }
}
