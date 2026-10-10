use std::fmt::Display;

use owo_colors::{OwoColorize, Style};
use strum::IntoEnumIterator;

use crate::spec::{CoverageLayer, Spec, Status};
use crate::support::fs::file_url;
use crate::support::style::{DIM, RED, hyperlink, version_line, visible_width};
use crate::support::text::wrap_text;

use super::report::CoverageReport;
use super::spec_coverage::{CriterionCoverage, SpecCoverage, SpecHealth};

#[cfg(test)]
#[path = "render_test.rs"]
mod render_test;

pub fn render(report: &CoverageReport, width: usize, verbose: bool) -> String {
    let mut writer = ReportWriter {
        width,
        verbose,
        id_width: id_width(report),
        text: String::new(),
    };

    writer.line(version_line());
    writer.blank();
    writer.specs_section(report);
    writer.gaps_section(report);
    writer.line(summary_line(report));
    writer.text
}

fn id_width(report: &CoverageReport) -> usize {
    report
        .specs
        .iter()
        .filter(|coverage| coverage.health() != SpecHealth::Complete)
        .map(|coverage| coverage.spec.id.as_str().len())
        .max()
        .unwrap_or(0)
}

fn summary_line(report: &CoverageReport) -> String {
    let gaps = report.gapped().len();
    let gaps_style = if gaps == 0 { Style::new().green() } else { RED };
    let gaps_text = format!("gaps ({gaps})").style(gaps_style).to_string();
    format!(
        "planned ({}), in-progress ({}), active ({}), {gaps_text}",
        report.count(Status::Planned),
        report.count(Status::InProgress),
        report.count(Status::Active),
    )
}

fn spec_link(spec: &Spec) -> String {
    hyperlink(&file_url(&spec.path), &spec.id.as_str().cyan().to_string())
}

fn health_marker(coverage: &SpecCoverage) -> String {
    match coverage.health() {
        SpecHealth::Gap => "✗".red().to_string(),
        SpecHealth::Pending | SpecHealth::Complete => "●".blue().to_string(),
    }
}

fn progress_text(coverage: &SpecCoverage) -> String {
    let text = format!(
        "{}/{} AC",
        coverage.implemented_count(),
        coverage.criteria.len()
    );
    if coverage.is_fully_implemented() {
        text.dimmed().to_string()
    } else if coverage.has_gap() {
        text.red().to_string()
    } else {
        text.blue().to_string()
    }
}

fn missing_summary(coverage: &SpecCoverage) -> String {
    let mut parts = Vec::new();

    if coverage.has_contract_gap() {
        parts.push("contract".to_string());
    }

    let layers = layer_summary(&coverage.gapped_criteria());
    if !layers.is_empty() {
        parts.push(layers);
    }

    parts.join(", ")
}

fn pending_summary(coverage: &SpecCoverage) -> String {
    let mut parts = Vec::new();

    if coverage.has_contract_pending() {
        parts.push("contract".to_string());
    }

    let layers = layer_summary(&coverage.pending_criteria());
    if !layers.is_empty() {
        parts.push(layers);
    }

    parts.join(", ")
}

fn layer_summary(criteria: &[&CriterionCoverage]) -> String {
    CoverageLayer::iter()
        .filter_map(|layer| {
            let ids: Vec<&str> = criteria
                .iter()
                .filter(|criterion| criterion.is_missing(layer))
                .map(|criterion| criterion.id.as_str())
                .collect();
            (!ids.is_empty()).then(|| format!("{}({})", layer.label(), ids.join(", ")))
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn missing_label(criterion: &CriterionCoverage) -> String {
    let layers: Vec<&str> = criterion
        .missing
        .iter()
        .map(|layer| layer.label())
        .collect();
    format!("missing {}", layers.join(" & "))
}

struct ReportWriter {
    width: usize,
    verbose: bool,
    id_width: usize,
    text: String,
}

impl ReportWriter {
    fn line(&mut self, text: impl Display) {
        self.text.push_str(&text.to_string());
        self.text.push('\n');
    }

    fn blank(&mut self) {
        self.text.push('\n');
    }

    fn end_section(&mut self) {
        self.text.truncate(self.text.trim_end_matches('\n').len());
        self.text.push_str("\n\n");
    }

    fn wrapped_after(&mut self, prefix: &str, text: &str, paint: Style) {
        let prefix_width = visible_width(prefix);
        let available = self.width.saturating_sub(prefix_width).max(20);
        let padding = " ".repeat(prefix_width);
        let mut lines = wrap_text(text, available).into_iter();
        let first = lines.next().unwrap_or_default();
        self.line(format!("{prefix}{}", first.style(paint)));
        for line in lines {
            self.line(format!("{padding}{}", line.style(paint)));
        }
    }

    fn description(&mut self, indent: usize, text: &str) {
        self.wrapped_after(&" ".repeat(indent), text, DIM);
    }

    fn specs_section(&mut self, report: &CoverageReport) {
        self.line(format!("Specs ({})", report.specs.len()).bold());
        if report.specs.is_empty() {
            self.line(format!("  {}", "none found".dimmed()));
            self.end_section();
            return;
        }

        for (status, group) in report.status_groups() {
            self.line(format!("  {} ({})", status.label().bold(), group.len()));
            match status {
                Status::Active => {}
                Status::InProgress => {
                    for coverage in group
                        .into_iter()
                        .filter(|coverage| coverage.health() == SpecHealth::Pending)
                    {
                        self.spec_progress(coverage);
                    }
                }
                Status::Planned | Status::Blocked | Status::Deprecated => {
                    for coverage in group {
                        self.listed_spec(coverage);
                    }
                }
            }
        }
        self.end_section();
    }

    fn listed_spec(&mut self, coverage: &SpecCoverage) {
        self.line(format!("    {}", spec_link(coverage.spec)));
        if self.verbose && coverage.spec.status == Status::Planned {
            self.description(6, &coverage.spec.description);
            self.blank();
        }
    }

    fn gaps_section(&mut self, report: &CoverageReport) {
        self.line(format!("Gaps ({})", report.gapped().len()).bold());
        if !report.has_tracked_specs() {
            self.line(format!("  {}", "no active or in-progress specs".dimmed()));
        } else if report.gapped().is_empty() {
            self.line(format!("  {}", "none".dimmed()));
        } else {
            for (scope, group) in report.gapped_by_scope() {
                self.line(format!("  {}", scope.unwrap_or("unscoped").bold()));
                for coverage in group {
                    self.spec_progress(coverage);
                }
            }
        }
        self.end_section();
    }

    fn spec_progress(&mut self, coverage: &SpecCoverage) {
        let lead = self.progress_lead(coverage);
        if self.verbose {
            self.verbose_progress(coverage, &lead);
        } else {
            self.compact_progress(coverage, &lead);
        }
    }

    fn progress_lead(&self, coverage: &SpecCoverage) -> String {
        let padding = " ".repeat(
            self.id_width
                .saturating_sub(coverage.spec.id.as_str().len()),
        );
        format!(
            "    {}  {}{padding}  {}  ",
            health_marker(coverage),
            spec_link(coverage.spec),
            progress_text(coverage),
        )
    }

    fn compact_progress(&mut self, coverage: &SpecCoverage, lead: &str) {
        let in_progress = format!("in-progress: {}", pending_summary(coverage));
        if !coverage.has_gap() {
            self.wrapped_after(lead, &in_progress, DIM);
            return;
        }

        self.wrapped_after(
            lead,
            &format!("missing: {}", missing_summary(coverage)),
            RED,
        );
        if coverage.has_pending() {
            let continuation = " ".repeat(visible_width(lead));
            self.wrapped_after(&continuation, &in_progress, DIM);
        }
    }

    fn verbose_progress(&mut self, coverage: &SpecCoverage, lead: &str) {
        if coverage.has_contract_gap() {
            self.wrapped_after(lead, "missing: contract", RED);
        } else {
            self.line(lead.trim_end());
        }

        self.description(8, &coverage.spec.description);
        for criterion in coverage.gapped_criteria() {
            let prefix = format!(
                "        {} ({}): ",
                criterion.id.as_str().cyan(),
                missing_label(criterion).red()
            );
            self.wrapped_after(&prefix, &criterion.text, DIM);
        }
        if !coverage.has_gap() {
            for criterion in coverage.pending_criteria() {
                let prefix = format!("        {}: ", criterion.id.as_str().cyan());
                self.wrapped_after(&prefix, &criterion.text, DIM);
            }
        }
        self.blank();
    }
}
