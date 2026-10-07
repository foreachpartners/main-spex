use std::path::Path;
use std::process::ExitCode;

use clap::Args;
use error_stack::Report;
use strum::IntoEnumIterator;

use crate::config::spex::Spex;
use crate::error::{CliError, Result};
use crate::spec::{CoverageLayer, Spec, Status};
use crate::support::fs::file_url;
use crate::support::style::{Style, visible_width};
use crate::support::util::wrap_text;
use crate::tracing::annotations::{AcCoverage, Found};

#[cfg(test)]
#[path = "coverage_test.rs"]
mod coverage_test;

#[derive(Args)]
pub struct CoverageArgs {
    #[arg(
        long,
        num_args = 1..,
        value_name = "REPOSITORY_SHORT_NAME",
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

struct SpecRow<'a> {
    spec: &'a Spec,
    contract_missing: bool,
    ac_rows: Vec<AcCoverage>,
}

impl<'a> SpecRow<'a> {
    fn new(spec: &'a Spec, found: &Found) -> Self {
        let tracked = spec.status.is_tracked();
        Self {
            spec,
            contract_missing: tracked && found.contract_missing(spec),
            ac_rows: if tracked {
                found.ac_coverage(spec)
            } else {
                Vec::new()
            },
        }
    }

    fn all(specs: &'a [Spec], found: &Found) -> Vec<Self> {
        specs.iter().map(|spec| Self::new(spec, found)).collect()
    }

    fn is_tracked(&self) -> bool {
        self.spec.status.is_tracked()
    }

    fn ac_done(&self) -> usize {
        self.ac_rows.iter().filter(|ac| ac.impl_ok()).count()
    }

    fn ac_total(&self) -> usize {
        self.ac_rows.len()
    }

    fn ac_complete(&self) -> bool {
        self.ac_done() == self.ac_total()
    }

    fn has_gap(&self) -> bool {
        self.contract_missing || self.ac_rows.iter().any(AcCoverage::is_gap)
    }

    fn has_pending(&self) -> bool {
        self.ac_rows.iter().any(AcCoverage::is_pending)
    }

    fn missing_summary(&self) -> String {
        let mut parts = Vec::new();

        if self.contract_missing {
            parts.push("contract".to_string());
        }

        let layers = ac_layer_summary(&self.gapped_ac());
        if !layers.is_empty() {
            parts.push(layers);
        }

        parts.join(", ")
    }

    fn pending_summary(&self) -> String {
        ac_layer_summary(&self.pending_ac())
    }

    fn gapped_ac(&self) -> Vec<&AcCoverage> {
        self.ac_rows.iter().filter(|ac| ac.is_gap()).collect()
    }

    fn pending_ac(&self) -> Vec<&AcCoverage> {
        self.ac_rows.iter().filter(|ac| ac.is_pending()).collect()
    }
}

pub fn run(args: CoverageArgs) -> Result<ExitCode> {
    let spex = Spex::discover_and_warn()?;
    let specs = spex.specs()?;

    for short_name in &args.scope {
        if !spex
            .config
            .repositories
            .iter()
            .any(|repo| &repo.repository_short_name == short_name)
        {
            return Err(Report::new(CliError::UnknownScope(short_name.clone())));
        }
    }

    let specs: Vec<Spec> = specs
        .into_iter()
        .filter(|spec| spec.scope_selected(&args.scope))
        .collect();

    let found = Found::scan_workspace(&spex.dir, &spex.config)?;

    Ok(report(&spex.dir, &specs, &found, args.verbose))
}

fn ac_layer_summary(ac_rows: &[&AcCoverage]) -> String {
    CoverageLayer::iter()
        .filter_map(|layer| {
            let ids: Vec<&str> = ac_rows
                .iter()
                .filter(|ac| ac.is_missing(layer))
                .map(|ac| ac.id.as_str())
                .collect();
            (!ids.is_empty()).then(|| format!("{}({})", layer.label(), ids.join(", ")))
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn report(spex_dir: &Path, specs: &[Spec], found: &Found, verbose: bool) -> ExitCode {
    let style = Style::detect();
    let rows = SpecRow::all(specs, found);
    print!(
        "{}",
        render(&style, terminal_width(), verbose, spex_dir, &rows)
    );

    if active_gaps(&rows) == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn render(style: &Style, width: usize, verbose: bool, spex_dir: &Path, rows: &[SpecRow]) -> String {
    let id_width = rows
        .iter()
        .filter(|row| row.has_gap() || row.has_pending())
        .map(|row| row.spec.id.len())
        .max()
        .unwrap_or(0);
    let mut printer = Printer {
        style,
        width,
        verbose,
        id_width,
        spex_dir,
        text: String::new(),
    };

    printer.line(style.version_line());
    printer.blank();
    printer.status_totals(rows);
    let gaps = printer.coverage(rows);
    printer.line(summary_line(style, rows, gaps));
    printer.text
}

fn active_gaps(rows: &[SpecRow]) -> usize {
    rows.iter()
        .filter(|row| row.spec.status == Status::Active && row.has_gap())
        .count()
}

fn count_status(rows: &[SpecRow], status: Status) -> usize {
    rows.iter().filter(|row| row.spec.status == status).count()
}

fn summary_line(style: &Style, rows: &[SpecRow], gaps: usize) -> String {
    let in_progress = count_status(rows, Status::InProgress);
    let planned = count_status(rows, Status::Planned);
    let active = count_status(rows, Status::Active);
    let gaps_text = if gaps == 0 {
        style.green(&format!("gaps ({gaps})"))
    } else {
        style.red(&format!("gaps ({gaps})"))
    };
    format!("planned ({planned}), in-progress ({in_progress}), active ({active}), {gaps_text}")
}

fn status_groups<'a>(rows: &'a [SpecRow]) -> Vec<(Status, Vec<&'a SpecRow<'a>>)> {
    Status::iter()
        .filter_map(|status| {
            let group: Vec<&SpecRow> = rows
                .iter()
                .filter(|row| row.spec.status == status)
                .collect();
            (!group.is_empty()).then_some((status, group))
        })
        .collect()
}

fn spec_link(style: &Style, spex_dir: &Path, spec: &Spec) -> String {
    style.hyperlink(
        &file_url(&spex_dir.join(spec.location())),
        &style.cyan(&spec.id),
    )
}

fn scope_groups<'a>(gapped: &[&'a SpecRow<'a>]) -> Vec<(Option<&'a str>, Vec<&'a SpecRow<'a>>)> {
    let mut scopes: Vec<&str> = gapped
        .iter()
        .flat_map(|row| row.spec.scope.iter().map(String::as_str))
        .collect();
    scopes.sort_unstable();
    scopes.dedup();

    let mut groups: Vec<(Option<&str>, Vec<&SpecRow>)> = scopes
        .into_iter()
        .map(|scope| {
            let rows = gapped
                .iter()
                .filter(|row| row.spec.scope.iter().any(|s| s == scope))
                .copied()
                .collect();
            (Some(scope), rows)
        })
        .collect();

    let unscoped: Vec<&SpecRow> = gapped
        .iter()
        .filter(|row| row.spec.scope.is_empty())
        .copied()
        .collect();
    if !unscoped.is_empty() {
        groups.push((None, unscoped));
    }

    groups
}

fn terminal_width() -> usize {
    terminal_size::terminal_size().map_or(100, |(terminal_size::Width(w), _)| w as usize)
}

fn ac_gap_label(ac: &AcCoverage) -> String {
    if ac.missing.is_empty() {
        return String::new();
    }
    let layers: Vec<&str> = ac.missing.iter().map(|layer| layer.label()).collect();
    format!("missing {}", layers.join(" & "))
}

type Paint = fn(&Style, &str) -> String;

struct Printer<'a> {
    style: &'a Style,
    width: usize,
    verbose: bool,
    id_width: usize,
    spex_dir: &'a Path,
    text: String,
}

impl Printer<'_> {
    fn line(&mut self, text: impl AsRef<str>) {
        self.text.push_str(text.as_ref());
        self.text.push('\n');
    }

    fn blank(&mut self) {
        self.text.push('\n');
    }

    fn wrapped(&mut self, indent: usize, text: &str) {
        let avail = self.width.saturating_sub(indent).max(20);
        let pad = " ".repeat(indent);
        for line in wrap_text(text, avail) {
            self.line(format!("{pad}{}", self.style.dim(&line)));
        }
    }

    fn wrapped_after(&mut self, prefix: &str, text: &str, paint: Paint) {
        let prefix_len = visible_width(prefix);
        let avail = self.width.saturating_sub(prefix_len).max(20);
        let pad = " ".repeat(prefix_len);
        let mut lines = wrap_text(text, avail).into_iter();
        let first = lines.next().unwrap_or_default();
        self.line(format!("{prefix}{}", paint(self.style, &first)));
        for line in lines {
            self.line(format!("{pad}{}", paint(self.style, &line)));
        }
    }

    fn status_totals(&mut self, rows: &[SpecRow]) {
        let style = self.style;
        self.line(style.bold(&format!("Specs ({})", rows.len())));
        if rows.is_empty() {
            self.line(format!("  {}", style.dim("none found")));
            self.blank();
            return;
        }

        let mut printed_blank = false;
        for (status, group) in status_groups(rows) {
            self.line(format!(
                "  {} ({})",
                style.bold(status.label()),
                group.len()
            ));
            printed_blank = false;
            match status {
                Status::InProgress => {
                    for row in group
                        .into_iter()
                        .filter(|row| row.has_pending() && !row.has_gap())
                    {
                        self.gap_row(row);
                        printed_blank = self.verbose;
                    }
                }
                Status::Active => {}
                _ => {
                    for row in group {
                        self.line(format!("    {}", spec_link(style, self.spex_dir, row.spec)));
                        printed_blank = false;
                        if self.verbose && status == Status::Planned {
                            self.wrapped(6, &row.spec.description);
                            self.blank();
                            printed_blank = true;
                        }
                    }
                }
            }
        }
        if !printed_blank {
            self.blank();
        }
    }

    fn gap_row(&mut self, row: &SpecRow) {
        let style = self.style;
        let id = format!(
            "{}{}",
            spec_link(style, self.spex_dir, row.spec),
            " ".repeat(self.id_width.saturating_sub(row.spec.id.len())),
        );
        let ac_text = format!("{}/{} AC", row.ac_done(), row.ac_total());
        let ac = if row.ac_complete() {
            style.dim(&ac_text)
        } else if row.has_gap() {
            style.red(&ac_text)
        } else {
            style.blue(&ac_text)
        };
        let marker = if row.has_gap() {
            style.red("✗")
        } else {
            style.blue("●")
        };
        let lead = format!("    {marker}  {id}  {ac}  ");

        let detail: Option<(String, Paint)> = if row.has_gap() {
            let layers = if self.verbose {
                row.contract_missing.then(|| "contract".to_string())
            } else {
                Some(row.missing_summary()).filter(|layers| !layers.is_empty())
            };
            layers.map(|layers| (format!("missing: {layers}"), Style::red as Paint))
        } else if self.verbose {
            None
        } else {
            Some((
                format!("in-progress: {}", row.pending_summary()),
                Style::dim as Paint,
            ))
        };
        match detail {
            None => self.line(lead.trim_end()),
            Some((text, paint)) => self.wrapped_after(&lead, &text, paint),
        }

        if row.has_gap() && row.has_pending() && !self.verbose {
            let pad = " ".repeat(visible_width(&lead));
            self.wrapped_after(
                &pad,
                &format!("in-progress: {}", row.pending_summary()),
                Style::dim,
            );
        }

        if self.verbose {
            self.wrapped(8, &row.spec.description);
            for ac in row.gapped_ac() {
                let label = ac_gap_label(ac);
                let prefix = format!("        {} ({}): ", style.cyan(&ac.id), style.red(&label));
                self.wrapped_after(&prefix, &ac.text, Style::dim);
            }
            if !row.has_gap() {
                for ac in row.pending_ac() {
                    let prefix = format!("        {}: ", style.cyan(&ac.id));
                    self.wrapped_after(&prefix, &ac.text, Style::dim);
                }
            }
            self.blank();
        }
    }

    fn coverage(&mut self, rows: &[SpecRow]) -> usize {
        let style = self.style;
        let gapped: Vec<&SpecRow> = rows.iter().filter(|row| row.has_gap()).collect();
        let gaps = gapped.len();

        self.line(style.bold(&format!("Gaps ({gaps})")));
        if !rows.iter().any(SpecRow::is_tracked) {
            self.line(format!("  {}", style.dim("no active or in-progress specs")));
            self.blank();
            return 0;
        }

        let mut printed_blank = false;
        if gapped.is_empty() {
            self.line(format!("  {}", style.dim("none")));
        } else {
            for (scope, scope_rows) in scope_groups(&gapped) {
                self.line(format!("  {}", style.bold(scope.unwrap_or("unscoped"))));
                printed_blank = false;
                for row in scope_rows {
                    self.gap_row(row);
                    printed_blank = self.verbose;
                }
            }
        }

        if !printed_blank {
            self.blank();
        }
        gaps
    }
}
