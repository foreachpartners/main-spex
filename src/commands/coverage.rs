use std::process::ExitCode;

use clap::Args;
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
    #[error("unknown repository short name `{0}` — not listed in spex.yaml")]
    UnknownScope(String),
}

#[derive(Args)]
pub struct CoverageArgs {
    #[arg(
        long,
        num_args = 1..,
        help = "Repository short names to include by spec `scope` (default: all)"
    )]
    scope: Vec<String>,
    #[arg(
        long,
        short,
        help = "Also show spec descriptions for planned specs, and spec/acceptance-criterion descriptions for each gap"
    )]
    verbose: bool,
}

struct SpecRow {
    id: String,
    description: String,
    status: Status,
    scope: Vec<String>,
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

pub fn run(args: CoverageArgs) -> Result<ExitCode> {
    let workspace = Workspace::discover().change_context(CoverageError::Workspace)?;
    let specs = workspace.specs().change_context(CoverageError::Workspace)?;

    for short_name in &args.scope {
        if !workspace
            .config
            .repositories
            .iter()
            .any(|repo| &repo.short_name == short_name)
        {
            return Err(Report::new(CoverageError::UnknownScope(short_name.clone())));
        }
    }

    let specs: Vec<Spec> = specs
        .into_iter()
        .filter(|spec| scope_selected(spec, &args.scope))
        .collect();

    let found = annotations::scan_workspace(&workspace.dir, &workspace.config)
        .change_context(CoverageError::Walk)?;

    Ok(report(&specs, &found, args.verbose))
}

fn scope_selected(spec: &Spec, scopes: &[String]) -> bool {
    scopes.is_empty()
        || scopes
            .iter()
            .any(|short_name| spec.scope.contains(short_name))
}

fn tracked_rows(specs: &[Spec], found: &Found) -> Vec<SpecRow> {
    specs
        .iter()
        .filter(|spec| matches!(spec.status, Status::Active | Status::InProgress))
        .map(|spec| {
            let (contract_required, contract_present) = annotations::contract_coverage(spec, found);
            SpecRow {
                id: spec.id.clone(),
                description: spec.description.clone(),
                status: spec.status,
                scope: spec.scope.clone(),
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

fn report(specs: &[Spec], found: &Found, verbose: bool) -> ExitCode {
    let style = Style::detect();

    print_status_totals(&style, specs, verbose);

    let rows = tracked_rows(specs, found);
    let gaps = print_coverage(&style, &rows, verbose);

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

// Specs per status, in Status::ALL order, skipping empty statuses.
fn status_groups(specs: &[Spec]) -> Vec<(Status, Vec<&Spec>)> {
    Status::ALL
        .into_iter()
        .filter_map(|status| {
            let specs: Vec<&Spec> = specs.iter().filter(|spec| spec.status == status).collect();
            (!specs.is_empty()).then_some((status, specs))
        })
        .collect()
}

fn print_status_totals(style: &Style, specs: &[Spec], verbose: bool) {
    println!("{}", style.bold(&format!("Specs ({})", specs.len())));
    if specs.is_empty() {
        println!("  {}", style.dim("none found"));
        println!();
        return;
    }

    let mut printed_blank = false;
    for (status, group) in status_groups(specs) {
        println!("  {} ({})", style.bold(status.label()), group.len());
        printed_blank = false;
        // Active and in-progress specs are already listed below with their
        // coverage detail, so listing their ids here would just duplicate that.
        if !matches!(status, Status::Active | Status::InProgress) {
            for spec in group {
                println!("    {}", style.cyan(&spec.id));
                printed_blank = false;
                if verbose && status == Status::Planned {
                    print_wrapped(style, 6, &spec.description);
                    println!();
                    printed_blank = true;
                }
            }
        }
    }
    if !printed_blank {
        println!();
    }
}

// Groups gapped rows by scope, sorted alphabetically; a row with more than
// one scope is duplicated under each of its scopes. Rows with no scope are
// grouped last under `None`.
fn scope_groups<'a>(gapped: &[&'a SpecRow]) -> Vec<(Option<&'a str>, Vec<&'a SpecRow>)> {
    let mut scopes: Vec<&str> = gapped
        .iter()
        .flat_map(|row| row.scope.iter().map(String::as_str))
        .collect();
    scopes.sort_unstable();
    scopes.dedup();

    let mut groups: Vec<(Option<&str>, Vec<&SpecRow>)> = scopes
        .into_iter()
        .map(|scope| {
            let rows = gapped
                .iter()
                .filter(|row| row.scope.iter().any(|s| s == scope))
                .copied()
                .collect();
            (Some(scope), rows)
        })
        .collect();

    let unscoped: Vec<&SpecRow> = gapped
        .iter()
        .filter(|row| row.scope.is_empty())
        .copied()
        .collect();
    if !unscoped.is_empty() {
        groups.push((None, unscoped));
    }

    groups
}

// The acceptance criteria still missing a required impl or test annotation.
fn gapped_ac(row: &SpecRow) -> Vec<&AcCoverage> {
    row.ac_rows.iter().filter(|ac| !ac.is_ok()).collect()
}

fn terminal_width() -> usize {
    terminal_size::terminal_size().map_or(100, |(terminal_size::Width(w), _)| w as usize)
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

// Prints `text` dimmed and word-wrapped to the terminal width, indented by
// `indent` spaces on every line.
fn print_wrapped(style: &Style, indent: usize, text: &str) {
    let avail = terminal_width().saturating_sub(indent).max(20);
    let pad = " ".repeat(indent);
    for line in wrap_text(text, avail) {
        println!("{pad}{}", style.dim(&line));
    }
}

// Prints `prefix` followed by `text` dimmed and word-wrapped to the terminal
// width; continuation lines are indented to align under `prefix`, whose
// rendered (non-ANSI) width is `prefix_len`.
fn print_wrapped_after(style: &Style, prefix: &str, prefix_len: usize, text: &str) {
    let avail = terminal_width().saturating_sub(prefix_len).max(20);
    let pad = " ".repeat(prefix_len);
    let mut lines = wrap_text(text, avail).into_iter();
    println!("{prefix}{}", style.dim(&lines.next().unwrap_or_default()));
    for line in lines {
        println!("{pad}{}", style.dim(&line));
    }
}

fn print_gap_row(style: &Style, row: &SpecRow, width: usize, verbose: bool) {
    let id = style.cyan(&format!("{:<width$}", row.id, width = width));
    let ac_text = format!("{}/{} AC", row.ac_done(), row.ac_total());
    let ac = if row.ac_complete() {
        style.dim(&ac_text)
    } else {
        style.red(&ac_text)
    };

    let layers = missing_summary(row);
    if layers.is_empty() {
        println!("    {}  {id}  {ac}", style.red("✗"));
    } else {
        let prefix_len = 4 + 1 + 2 + width + 2 + ac_text.len() + 2;
        let prefix = format!("    {}  {id}  {ac}  ", style.red("✗"));
        print_wrapped_after(style, &prefix, prefix_len, &format!("missing: {layers}"));
    }

    if verbose {
        print_wrapped(style, 8, &row.description);
        for ac in gapped_ac(row) {
            let prefix_len = 8 + ac.id.len() + 2;
            let prefix = format!("        {}: ", style.cyan(&ac.id));
            print_wrapped_after(style, &prefix, prefix_len, &ac.text);
        }
        println!();
    }
}

fn print_coverage(style: &Style, rows: &[SpecRow], verbose: bool) -> usize {
    let gapped: Vec<&SpecRow> = rows.iter().filter(|row| row.has_gap()).collect();
    let gaps = gapped.len();

    println!("{}", style.bold(&format!("Gaps ({gaps})")));
    if rows.is_empty() {
        println!("  {}", style.dim("no active or in-progress specs"));
        println!();
        return 0;
    }

    let mut printed_blank = false;
    if gapped.is_empty() {
        println!("  {}", style.dim("none"));
    } else {
        let width = gapped.iter().map(|row| row.id.len()).max().unwrap_or(0);
        for (scope, scope_rows) in scope_groups(&gapped) {
            println!("  {}", style.bold(scope.unwrap_or("unscoped")));
            printed_blank = false;
            for row in scope_rows {
                print_gap_row(style, row, width, verbose);
                printed_blank = verbose;
            }
        }
    }

    if !printed_blank {
        println!();
    }
    gaps
}
