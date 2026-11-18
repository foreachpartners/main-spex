use std::iter;
use std::path::Path;

use serde::Serialize;
use strum::IntoEnumIterator;

use crate::config::CLI_VERSION;
use crate::spec::{AcId, ContractLevel, CoverageIgnore, CoverageLayer, Spec, SpecId, Status};

use super::annotation_index::{AnnotationIndex, PackageScan, ScanState, Trace};
use super::report::CoverageReport;
use super::spec_coverage::{CriterionCoverage, SpecCoverage, SpecHealth};

pub(super) struct CoverageEvidence<'a> {
    index: &'a AnnotationIndex,
    pub(super) strict: bool,
    pub(super) show_traces: bool,
    pub(super) unavailable: Vec<InputIssue<'a>>,
}

impl<'a> CoverageEvidence<'a> {
    pub(super) fn new(
        index: &'a AnnotationIndex,
        report: &CoverageReport<'_>,
        selected_packages: &[&str],
        strict: bool,
        show_traces: bool,
    ) -> Self {
        let mut unavailable = Vec::new();
        for scan in &index.packages {
            let blocking = strict
                || selected_packages.is_empty()
                || selected_packages.contains(&scan.short_name.as_str())
                || report.specs.iter().any(|coverage| {
                    coverage.is_tracked() && coverage.spec.packages.contains(&scan.short_name)
                });
            let inputs = iter::once((InputKind::Package, &scan.root))
                .chain(
                    scan.contracts
                        .iter()
                        .map(|path| (InputKind::Contract, path)),
                )
                .chain(
                    scan.unavailable_source_files
                        .iter()
                        .map(|path| (InputKind::SourceFile, path)),
                );
            unavailable.extend(
                inputs
                    .filter(|(_, input)| input.state != ScanState::Scanned)
                    .map(|(kind, input)| InputIssue {
                        kind,
                        state: input.state,
                        package: &scan.package,
                        path: &input.path,
                        blocking,
                    }),
            );
        }

        Self {
            index,
            strict,
            show_traces,
            unavailable,
        }
    }

    pub(super) fn traces(&self, id: &SpecId) -> &[Trace] {
        self.index.traces(id)
    }

    pub(super) fn has_blocking_inputs(&self) -> bool {
        self.unavailable.iter().any(|input| input.blocking)
    }

    pub(super) fn json<'report>(
        &'report self,
        project: &'report str,
        report: &'report CoverageReport<'_>,
    ) -> impl Serialize + 'report {
        JsonReport {
            schema_version: 1,
            command: "coverage",
            cli_version: CLI_VERSION,
            project_name: project,
            strict: self.strict,
            test_execution: "not-run",
            packages: &self.index.packages,
            unavailable_inputs: &self.unavailable,
            specs: report
                .specs
                .iter()
                .map(|coverage| SpecEvidence::new(coverage, self.traces(&coverage.spec.id)))
                .collect(),
            summary: Summary {
                active: report.count(Status::Active),
                in_progress: report.count(Status::InProgress),
                planned: report.count(Status::Planned),
                blocked: report.count(Status::Blocked),
                deprecated: report.count(Status::Deprecated),
                gapped_specs: report.gapped().len(),
                blocking_unavailable_inputs: self
                    .unavailable
                    .iter()
                    .filter(|input| input.blocking)
                    .count(),
            },
        }
    }
}

#[derive(Serialize)]
pub(super) struct InputIssue<'a> {
    pub(super) kind: InputKind,
    pub(super) state: ScanState,
    pub(super) package: &'a str,
    pub(super) path: &'a Path,
    pub(super) blocking: bool,
}

#[derive(Clone, Copy, Serialize, strum::IntoStaticStr)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(super) enum InputKind {
    Package,
    Contract,
    SourceFile,
}

#[derive(Serialize)]
struct JsonReport<'a> {
    schema_version: u8,
    command: &'static str,
    cli_version: &'static str,
    project_name: &'a str,
    strict: bool,
    test_execution: &'static str,
    packages: &'a [PackageScan],
    unavailable_inputs: &'a [InputIssue<'a>],
    specs: Vec<SpecEvidence<'a>>,
    summary: Summary,
}

#[derive(Serialize)]
struct Summary {
    active: usize,
    in_progress: usize,
    planned: usize,
    blocked: usize,
    deprecated: usize,
    gapped_specs: usize,
    blocking_unavailable_inputs: usize,
}

#[derive(Serialize)]
struct SpecEvidence<'a> {
    id: &'a SpecId,
    path: &'a Path,
    packages: &'a [String],
    status: &'static str,
    state: &'static str,
    contract: &'static str,
    criteria: Vec<CriterionEvidence<'a>>,
    waivers: Vec<DeclaredWaiver<'a>>,
    traces: &'a [Trace],
}

impl<'a> SpecEvidence<'a> {
    fn new(coverage: &'a SpecCoverage<'_>, traces: &'a [Trace]) -> Self {
        let spec = coverage.spec;
        let state = if coverage.is_tracked() {
            match coverage.health() {
                SpecHealth::Gap => "gap",
                SpecHealth::Pending => "pending",
                SpecHealth::Complete => "covered",
            }
        } else {
            "untracked"
        };

        Self {
            id: &spec.id,
            path: &spec.path,
            packages: &spec.packages,
            status: spec.status.label(),
            state,
            contract: contract_state(coverage),
            criteria: coverage
                .criteria
                .iter()
                .map(|criterion| CriterionEvidence::new(spec, criterion))
                .collect(),
            waivers: declared_waivers(spec).collect(),
            traces,
        }
    }
}

fn contract_state(coverage: &SpecCoverage<'_>) -> &'static str {
    if !coverage.is_tracked() {
        return "untracked";
    }
    match coverage.spec.contract {
        ContractLevel::None => "not-required",
        ContractLevel::ForEachAc => "per-criterion",
        ContractLevel::Single if coverage.spec.layer_ignored(CoverageLayer::Contract) => "waived",
        ContractLevel::Single if coverage.has_contract_gap() => "gap",
        ContractLevel::Single if coverage.has_contract_pending() => "pending",
        ContractLevel::Single => "covered",
    }
}

#[derive(Serialize)]
struct CriterionEvidence<'a> {
    id: &'a AcId,
    is_done: bool,
    state: &'static str,
    missing: &'a [CoverageLayer],
    waived: Vec<CoverageLayer>,
}

impl<'a> CriterionEvidence<'a> {
    fn new(spec: &Spec, criterion: &'a CriterionCoverage) -> Self {
        let state = if criterion.is_gap() {
            "gap"
        } else if criterion.is_pending() {
            "pending"
        } else {
            "covered"
        };

        Self {
            id: &criterion.id,
            is_done: criterion.is_done,
            state,
            missing: &criterion.missing,
            waived: CoverageLayer::iter()
                .filter(|&layer| {
                    (layer != CoverageLayer::Contract || spec.contract == ContractLevel::ForEachAc)
                        && spec.ac_ignored(&criterion.id, layer)
                })
                .collect(),
        }
    }
}

#[derive(Serialize)]
pub(super) struct DeclaredWaiver<'a> {
    pub(super) package: Option<&'a str>,
    #[serde(flatten)]
    pub(super) ignore: &'a CoverageIgnore,
}

pub(super) fn declared_waivers(spec: &Spec) -> impl Iterator<Item = DeclaredWaiver<'_>> {
    spec.coverage_ignore
        .iter()
        .map(|ignore| DeclaredWaiver {
            package: None,
            ignore,
        })
        .chain(
            spec.package_coverage_ignore
                .iter()
                .flat_map(|(package, waivers)| {
                    waivers.iter().map(move |ignore| DeclaredWaiver {
                        package: Some(package),
                        ignore,
                    })
                }),
        )
}
