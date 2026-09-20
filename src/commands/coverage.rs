use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Args;
use error_stack::{Report, ResultExt};
use thiserror::Error;

use crate::annotations::{self, AcCoverage, Found};
use crate::spec::{Spec, Status};
use crate::style::Style;
use crate::util::file_url;
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
    file: PathBuf,
    description: String,
    status: Status,
    scope: Vec<String>,
    contract_required: bool,
    contract_present: bool,
    ac_rows: Vec<AcCoverage>,
}

impl SpecRow {
    // The `X/Y AC` counter tracks implementation, not testing — a criterion
    // still missing its test annotation still counts as done here, and shows
    // up separately in `missing`/`in-progress` instead.
    fn ac_done(&self) -> usize {
        self.ac_rows.iter().filter(|ac| ac.impl_ok()).count()
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

    // An acceptance criterion still missing a required impl or test
    // annotation, but not flagged `is_done: false` — that's not expected
    // (a real gap), unlike a pending one.
    fn has_real_ac_gap(&self) -> bool {
        self.ac_rows
            .iter()
            .any(|ac| !ac.is_ok() && !ac.is_pending())
    }

    // An acceptance criterion flagged `is_done: false` and not yet traced —
    // expected on an in-progress spec, so it's not a gap.
    fn has_pending(&self) -> bool {
        self.ac_rows.iter().any(|ac| ac.is_pending())
    }

    // A gap is a missing required contract annotation, or any acceptance
    // criterion still missing a required impl or test annotation that isn't
    // itself flagged `is_done: false`.
    fn has_gap(&self) -> bool {
        !self.contract_ok() || self.has_real_ac_gap()
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

    Ok(report(&workspace.dir, &specs, &found, args.verbose))
}

fn scope_selected(spec: &Spec, scopes: &[String]) -> bool {
    scopes.is_empty()
        || scopes
            .iter()
            .any(|short_name| spec.scope.contains(short_name))
}

fn tracked_rows(workspace: &Path, specs: &[Spec], found: &Found) -> Vec<SpecRow> {
    specs
        .iter()
        .filter(|spec| matches!(spec.status, Status::Active | Status::InProgress))
        .map(|spec| {
            let (contract_required, contract_present) = annotations::contract_coverage(spec, found);
            SpecRow {
                id: spec.id.clone(),
                file: workspace.join(spec.location()),
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

// Renders which of the given acceptance criteria still lack a required impl
// or test annotation, as `impl(ac-1), test(ac-1, ac-2)`.
fn ac_layer_summary(ac_rows: &[&AcCoverage]) -> String {
    let by_layer: [(&str, Vec<&str>); 2] = [
        (
            "impl",
            ac_rows
                .iter()
                .filter(|ac| ac.impl_required && !ac.impl_present)
                .map(|ac| ac.id.as_str())
                .collect(),
        ),
        (
            "test",
            ac_rows
                .iter()
                .filter(|ac| ac.test_required && !ac.test_present)
                .map(|ac| ac.id.as_str())
                .collect(),
        ),
    ];
    by_layer
        .into_iter()
        .filter(|(_, missing)| !missing.is_empty())
        .map(|(layer, missing)| format!("{layer}({})", missing.join(", ")))
        .collect::<Vec<_>>()
        .join(", ")
}

// Lists what's missing on a gapped row: the contract annotation, then which
// acceptance criteria still lack a required impl or test annotation, other
// than ones flagged `is_done: false` (those are pending, not a gap).
fn missing_summary(row: &SpecRow) -> String {
    let mut parts = Vec::new();

    if !row.contract_ok() {
        parts.push("contract".to_string());
    }

    let layers = ac_layer_summary(&gapped_ac(row));
    if !layers.is_empty() {
        parts.push(layers);
    }

    parts.join(", ")
}

// Lists which acceptance criteria are flagged `is_done: false` and not yet
// traced — expected on an in-progress spec, reported but not a gap.
fn pending_summary(row: &SpecRow) -> String {
    let pending: Vec<&AcCoverage> = row.ac_rows.iter().filter(|ac| ac.is_pending()).collect();
    ac_layer_summary(&pending)
}

fn report(workspace: &Path, specs: &[Spec], found: &Found, verbose: bool) -> ExitCode {
    let style = Style::detect();
    let rows = tracked_rows(workspace, specs, found);
    // Shared across the pending list (in the "Specs" totals) and the "Gaps"
    // list, so the `id`/`AC` columns line up between the two sections.
    let id_width = rows
        .iter()
        .filter(|row| row.has_gap() || row.has_pending())
        .map(|row| row.id.len())
        .max()
        .unwrap_or(0);

    print_status_totals(&style, workspace, specs, &rows, id_width, verbose);

    let gaps = print_coverage(&style, &rows, id_width, verbose);

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
    format!("planned ({planned}), in-progress ({in_progress}), active ({active}), {gaps_text}")
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

fn print_status_totals(
    style: &Style,
    workspace: &Path,
    specs: &[Spec],
    rows: &[SpecRow],
    id_width: usize,
    verbose: bool,
) {
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
        match status {
            // A gapped in-progress spec is already listed below with its
            // coverage detail; a fully covered one has nothing to show. Only
            // a pending-only one (an `is_done: false` criterion not yet
            // traced, and nothing else wrong) is listed here — it's not a
            // gap, just this spec's in-progress detail.
            Status::InProgress => {
                let pending: Vec<&SpecRow> = group
                    .iter()
                    .filter_map(|spec| rows.iter().find(|row| row.id == spec.id))
                    .filter(|row| row.has_pending() && !row.has_gap())
                    .collect();
                for row in pending {
                    print_gap_row(style, row, id_width, verbose);
                    printed_blank = verbose;
                }
            }
            // Active specs are already listed below with their coverage
            // detail, so listing their ids here would just duplicate that.
            Status::Active => {}
            _ => {
                for spec in group {
                    println!("    {}", spec_link(style, workspace, spec));
                    printed_blank = false;
                    if verbose && status == Status::Planned {
                        print_wrapped(style, 6, &spec.description);
                        println!();
                        printed_blank = true;
                    }
                }
            }
        }
    }
    if !printed_blank {
        println!();
    }
}

fn spec_link(style: &Style, workspace: &Path, spec: &Spec) -> String {
    style.hyperlink(
        &file_url(&workspace.join(spec.location())),
        &style.cyan(&spec.id),
    )
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

// The acceptance criteria still missing a required impl or test annotation,
// other than ones flagged `is_done: false` (those are pending, not a gap).
fn gapped_ac(row: &SpecRow) -> Vec<&AcCoverage> {
    row.ac_rows
        .iter()
        .filter(|ac| !ac.is_ok() && !ac.is_pending())
        .collect()
}

// The acceptance criteria flagged `is_done: false` and not yet traced.
fn pending_ac(row: &SpecRow) -> Vec<&AcCoverage> {
    row.ac_rows.iter().filter(|ac| ac.is_pending()).collect()
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
    print_wrapped_after_colored(style, prefix, prefix_len, text, Style::dim);
}

// Like `print_wrapped_after`, but colors each wrapped line with `color`
// instead of always dimming it — used to highlight a "missing: ..." message
// red.
fn print_wrapped_after_colored(
    style: &Style,
    prefix: &str,
    prefix_len: usize,
    text: &str,
    color: fn(&Style, &str) -> String,
) {
    let avail = terminal_width().saturating_sub(prefix_len).max(20);
    let pad = " ".repeat(prefix_len);
    let mut lines = wrap_text(text, avail).into_iter();
    println!(
        "{prefix}{}",
        color(style, &lines.next().unwrap_or_default())
    );
    for line in lines {
        println!("{pad}{}", color(style, &line));
    }
}

// Describes which layers a single gapped acceptance criterion is missing, as
// `missing impl & test`, `missing impl`, or `missing test`.
fn ac_gap_label(ac: &AcCoverage) -> String {
    let missing_impl = ac.impl_required && !ac.impl_present;
    let missing_test = ac.test_required && !ac.test_present;
    match (missing_impl, missing_test) {
        (true, true) => "missing impl & test".to_string(),
        (true, false) => "missing impl".to_string(),
        (false, true) => "missing test".to_string(),
        (false, false) => String::new(),
    }
}

fn print_gap_row(style: &Style, row: &SpecRow, width: usize, verbose: bool) {
    let id = format!(
        "{}{}",
        style.hyperlink(&file_url(&row.file), &style.cyan(&row.id)),
        " ".repeat(width.saturating_sub(row.id.len())),
    );
    let ac_text = format!("{}/{} AC", row.ac_done(), row.ac_total());
    let ac = if row.ac_complete() {
        style.dim(&ac_text)
    } else if row.has_gap() {
        style.red(&ac_text)
    } else {
        style.blue(&ac_text)
    };
    let prefix_len = 4 + 1 + 2 + width + 2 + ac_text.len() + 2;

    // In verbose mode the per-criterion gap is already shown inline below, so
    // the header only needs to call out a missing contract, which isn't
    // per-AC.
    let header_summary = if verbose {
        (!row.contract_ok()).then(|| "contract".to_string())
    } else {
        let layers = missing_summary(row);
        (!layers.is_empty()).then_some(layers)
    };

    if row.has_gap() {
        match header_summary {
            None => println!("    {}  {id}  {ac}", style.red("✗")),
            Some(layers) => {
                let prefix = format!("    {}  {id}  {ac}  ", style.red("✗"));
                print_wrapped_after_colored(
                    style,
                    &prefix,
                    prefix_len,
                    &format!("missing: {layers}"),
                    Style::red,
                );
            }
        }
    } else if verbose {
        println!("    {}  {id}  {ac}", style.blue("●"));
    } else {
        let prefix = format!("    {}  {id}  {ac}  ", style.blue("●"));
        print_wrapped_after(
            style,
            &prefix,
            prefix_len,
            &format!("in-progress: {}", pending_summary(row)),
        );
    }

    if row.has_gap() && row.has_pending() && !verbose {
        let pad = " ".repeat(prefix_len);
        print_wrapped_after(
            style,
            &pad,
            prefix_len,
            &format!("in-progress: {}", pending_summary(row)),
        );
    }

    if verbose {
        print_wrapped(style, 8, &row.description);
        for ac in gapped_ac(row) {
            let label = ac_gap_label(ac);
            let prefix = format!("        {} ({}): ", style.cyan(&ac.id), style.red(&label));
            let prefix_len = 8 + ac.id.len() + 3 + label.len() + 3;
            print_wrapped_after(style, &prefix, prefix_len, &ac.text);
        }
        // Pending detail only belongs here for a pending-only row (listed
        // under its spec's in-progress status group); a row shown here
        // because of a real gap must not have its pending criteria mistaken
        // for gaps.
        if !row.has_gap() {
            for ac in pending_ac(row) {
                let prefix = format!("        {}: ", style.cyan(&ac.id));
                let prefix_len = 8 + ac.id.len() + 2;
                print_wrapped_after(style, &prefix, prefix_len, &ac.text);
            }
        }
        println!();
    }
}

fn print_coverage(style: &Style, rows: &[SpecRow], id_width: usize, verbose: bool) -> usize {
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
        for (scope, scope_rows) in scope_groups(&gapped) {
            println!("  {}", style.bold(scope.unwrap_or("unscoped")));
            printed_blank = false;
            for row in scope_rows {
                print_gap_row(style, row, id_width, verbose);
                printed_blank = verbose;
            }
        }
    }

    if !printed_blank {
        println!();
    }
    gaps
}
