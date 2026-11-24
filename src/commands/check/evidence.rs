use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::Serialize;

use crate::config::{CLI_VERSION, Package, spex::Spex};
use crate::error::Result;
use crate::rule::{CheckKind, RunScope, Severity};
use crate::support::fs::normalize;
use crate::support::output::print_json;

use super::report::{CheckReport, ReportTarget};
use super::runners::Outcome;
use super::scoped_file::ScopedFile;

#[derive(Serialize)]
pub(super) struct RuleExecution {
    scope: &'static str,
    root: PathBuf,
    source: String,
    status: &'static str,
    files: Vec<PathBuf>,
}

impl RuleExecution {
    pub(super) fn skipped(root: &Path, source: String) -> Self {
        Self {
            scope: "per-file",
            root: normalize(root),
            source,
            status: "skipped-partial-selection",
            files: Vec::new(),
        }
    }

    pub(super) fn new(
        root: &Path,
        source: String,
        kind: &CheckKind,
        files: &[&ScopedFile],
        outcome: &Outcome,
    ) -> Self {
        let scope = match kind {
            CheckKind::Script {
                run: RunScope::PerPackage,
                ..
            } => "package",
            CheckKind::Script {
                run: RunScope::Project,
                ..
            } => "project",
            _ => "per-file",
        };
        let status = match outcome {
            Err(_) => "execution-error",
            Ok(findings) if !findings.is_empty() => "findings",
            Ok(_) if scope == "per-file" && files.is_empty() && !kind.negate() => {
                "no-matching-files"
            }
            Ok(_) => "passed",
        };
        Self {
            scope,
            root: normalize(root),
            source,
            status,
            files: files.iter().map(|file| normalize(&file.abs)).collect(),
        }
    }
}

pub(super) fn report_json(
    spex: &Spex,
    selected: &[&Package],
    files: &[String],
    reports: &[CheckReport<'_>],
    inherited: &[CheckReport<'_>],
    executions: &[RuleExecution],
) -> Result<ExitCode> {
    let errors = count_errors(reports);
    let finding_count = reports
        .iter()
        .map(|report| report.findings.len())
        .sum::<usize>();
    let packages: Vec<_> = spex
        .packages
        .iter()
        .map(|package| {
            let selected = selected
                .iter()
                .any(|entry| entry.package_name == package.package_name);
            serde_json::json!({
                "name": package.package_name,
                "path": package.path,
                "selected": selected,
                "available": package.path.is_dir(),
                "declared_contracts": package.contracts,
            })
        })
        .collect();
    print_json(&serde_json::json!({
        "schema_version": 1,
        "command": "check",
        "cli_version": CLI_VERSION,
        "project_name": &spex.project_name,
        "project_root": &spex.dir,
        "scope": if files.is_empty() && selected.len() == spex.packages.len() { "project" } else { "partial" },
        "requested_files": files,
        "packages": packages,
        "rule_executions": executions,
        "findings": findings(reports),
        "inherited_findings": findings(inherited),
        "test_execution": "not-run",
        "manual_acceptance": "not-run",
        "summary": {
            "errors": errors,
            "warnings": finding_count - errors,
            "inherited": inherited.iter().map(|report| report.findings.len()).sum::<usize>(),
        },
    }))?;
    Ok(if errors == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn count_errors(reports: &[CheckReport<'_>]) -> usize {
    reports
        .iter()
        .flat_map(|report| &report.findings)
        .filter(|finding| finding.severity == Severity::Error)
        .count()
}

fn findings(reports: &[CheckReport<'_>]) -> Vec<serde_json::Value> {
    reports
        .iter()
        .flat_map(|report| {
            report.findings.iter().map(move |finding| {
                let severity: &str = finding.severity.into();
                let location = finding.location.as_ref().map(|location| {
                    serde_json::json!({
                        "file": location.file,
                        "path": normalize(&report.target.base_dir().join(&location.file)),
                        "line": location.line,
                    })
                });
                let target = match report.target {
                    ReportTarget::Package(package) => package.package_name.as_str(),
                    ReportTarget::Project { .. } => "project",
                };
                serde_json::json!({
                    "target": target,
                    "severity": severity,
                    "kind": finding.source.kind(),
                    "source": finding.source.to_string(),
                    "message": finding.message,
                    "location": location,
                })
            })
        })
        .collect()
}
