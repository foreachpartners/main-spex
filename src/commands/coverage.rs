use std::process::ExitCode;

use error_stack::{Report, ResultExt};
use thiserror::Error;

use crate::annotations::{self, AcCoverage, Found};
use crate::spec::{Spec, Status};
use crate::style::Style;
use crate::workspace::Workspace;

#[cfg(test)]
#[path = "coverage_test.rs"]
mod coverage_test;

type Result<T> = core::result::Result<T, Report<CoverageError>>;

#[derive(Debug, Error)]
pub enum CoverageError {
    #[error("could not load the spex workspace")]
    Workspace,
    #[error("could not walk workspace repository files")]
    Walk,
}

struct SpecRow {
    id: String,
    status: Status,
    contract_required: bool,
    contract_present: bool,
    ac_rows: Vec<AcCoverage>,
}

impl SpecRow {
    fn ac_done(&self) -> usize {
        self.ac_rows.iter().filter(|ac| ac.is_ok()).count()
    }

    fn ac_total(&self) -> usize {
        self.ac_rows.len()
    }

    fn ac_complete(&self) -> bool {
        self.ac_done() == self.ac_total()
    }

    fn contract_ok(&self) -> bool {
        !self.contract_required || self.contract_present
    }

    // A gap is a missing required contract annotation, or any acceptance
    // criterion still missing a required impl or test annotation.
    fn has_gap(&self) -> bool {
        !self.contract_ok() || !self.ac_complete()
    }
}

pub fn run() -> Result<ExitCode> {
    let workspace = Workspace::discover().change_context(CoverageError::Workspace)?;
    let specs = workspace.specs().change_context(CoverageError::Workspace)?;

    let found = annotations::scan_workspace(&workspace.dir, &workspace.config)
        .change_context(CoverageError::Walk)?;

    Ok(report(&specs, &found))
}

fn tracked_rows(specs: &[Spec], found: &Found) -> Vec<SpecRow> {
    specs
        .iter()
        .filter(|spec| matches!(spec.status, Status::Active | Status::InProgress))
        .map(|spec| {
            let (contract_required, contract_present) = annotations::contract_coverage(spec, found);
            SpecRow {
                id: spec.id.clone(),
                status: spec.status,
                contract_required,
                contract_present,
                ac_rows: annotations::ac_coverage(spec, found),
            }
        })
        .collect()
}

// Lists what's missing on a gapped row: the contract annotation, then which
// acceptance criteria still lack a required impl or test annotation.
fn missing_summary(row: &SpecRow) -> String {
    let mut parts = Vec::new();

    if !row.contract_ok() {
        parts.push("contract".to_string());
    }

    let by_layer: [(&str, Vec<&str>); 2] = [
        (
            "impl",
            row.ac_rows
                .iter()
                .filter(|ac| ac.impl_required && !ac.impl_present)
                .map(|ac| ac.id.as_str())
                .collect(),
        ),
        (
            "test",
            row.ac_rows
                .iter()
                .filter(|ac| ac.test_required && !ac.test_present)
                .map(|ac| ac.id.as_str())
                .collect(),
        ),
    ];
    for (layer, missing) in by_layer {
        if !missing.is_empty() {
            parts.push(format!("{layer}({})", missing.join(", ")));
        }
    }

    parts.join(", ")
}

fn report(specs: &[Spec], found: &Found) -> ExitCode {
    let style = Style::detect();

    print_status_totals(&style, specs);

    let rows = tracked_rows(specs, found);
    let gaps = print_coverage(&style, &rows);

    println!("{}", summary_line(&style, specs, gaps));

    if active_gaps(&rows) == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

// Only active spec gaps fail the command; in-progress gaps are reported but
// don't block, since an in-progress spec is expected to still have gaps.
fn active_gaps(rows: &[SpecRow]) -> usize {
    rows.iter()
        .filter(|row| row.status == Status::Active && row.has_gap())
        .count()
}

fn count_status(specs: &[Spec], status: Status) -> usize {
    specs.iter().filter(|spec| spec.status == status).count()
}

fn summary_line(style: &Style, specs: &[Spec], gaps: usize) -> String {
    let in_progress = count_status(specs, Status::InProgress);
    let planned = count_status(specs, Status::Planned);
    let active = count_status(specs, Status::Active);
    let gaps_text = if gaps == 0 {
        style.green(&format!("gaps ({gaps})"))
    } else {
        style.red(&format!("gaps ({gaps})"))
    };
    format!("in-progress ({in_progress}), planned ({planned}), active ({active}), {gaps_text}")
}

// Spec ids per status, in Status::ALL order, skipping empty statuses.
fn status_groups(specs: &[Spec]) -> Vec<(Status, Vec<&str>)> {
    Status::ALL
        .into_iter()
        .filter_map(|status| {
            let ids: Vec<&str> = specs
                .iter()
                .filter(|spec| spec.status == status)
                .map(|spec| spec.id.as_str())
                .collect();
            (!ids.is_empty()).then_some((status, ids))
        })
        .collect()
}

fn print_status_totals(style: &Style, specs: &[Spec]) {
    println!("{}", style.bold(&format!("Specs ({})", specs.len())));
    if specs.is_empty() {
        println!("  {}", style.dim("none found"));
        println!();
        return;
    }

    for (status, ids) in status_groups(specs) {
        println!("  {} ({})", style.bold(status.label()), ids.len());
        // Active and in-progress specs are already listed below with their
        // coverage detail, so listing their ids here would just duplicate that.
        if !matches!(status, Status::Active | Status::InProgress) {
            for id in ids {
                println!("    {}", style.cyan(id));
            }
        }
    }
    println!();
}

fn print_coverage(style: &Style, rows: &[SpecRow]) -> usize {
    let gapped: Vec<&SpecRow> = rows.iter().filter(|row| row.has_gap()).collect();
    let gaps = gapped.len();

    println!("{}", style.bold(&format!("Gaps ({gaps})")));
    if rows.is_empty() {
        println!("  {}", style.dim("no active or in-progress specs"));
        println!();
        return 0;
    }

    if gapped.is_empty() {
        println!("  {}", style.dim("none"));
    } else {
        let width = gapped.iter().map(|row| row.id.len()).max().unwrap_or(0);
        for row in &gapped {
            let id = style.cyan(&format!("{:<width$}", row.id, width = width));
            let ac_text = format!("{}/{} AC", row.ac_done(), row.ac_total());
            let ac = if row.ac_complete() {
                style.dim(&ac_text)
            } else {
                style.red(&ac_text)
            };

            let layers = missing_summary(row);
            if layers.is_empty() {
                println!("  {}  {id}  {ac}", style.red("✗"));
            } else {
                println!(
                    "  {}  {id}  {ac}  {}",
                    style.red("✗"),
                    style.dim(&format!("missing: {layers}"))
                );
            }
        }
    }

    println!();
    gaps
}
