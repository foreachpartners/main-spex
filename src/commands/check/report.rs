use std::path::Path;
use std::process::ExitCode;

use crate::config::Config;
use crate::rule::Severity;
use crate::style::Style;
use crate::util::{file_url, plural};

use super::{Finding, RepoReport};

#[cfg(test)]
#[path = "report_test.rs"]
mod report_test;

pub(super) fn report(config: &Config, spex_dir: &Path, reports: &[RepoReport]) -> ExitCode {
    let style = Style::detect();
    println!("{}", style.version_line());
    println!();
    let mut total_errors = 0usize;
    let mut total_warnings = 0usize;

    for repo in reports {
        let errors = repo
            .findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .count();
        let warnings = repo.findings.len() - errors;
        total_errors += errors;
        total_warnings += warnings;

        let base = config.repo_base_dir(spex_dir, &repo.name);

        println!("{}", style.bold(&repo.name));
        if repo.findings.is_empty() {
            println!("  {}", style.green("no issues"));
        } else {
            for finding in &repo.findings {
                print_finding(&style, &base, finding);
            }
            println!("  {}", style.dim(&count_summary(errors, warnings)));
        }
        println!();
    }

    let total = count_summary(total_errors, total_warnings);
    println!(
        "{}: {}",
        style.bold(&config.project_name),
        style.bold(&total)
    );

    if total_errors > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn print_finding(style: &Style, base: &Path, finding: &Finding) {
    let severity = match finding.severity {
        Severity::Error => style.red("error"),
        Severity::Warn => style.yellow("warn"),
    };
    let check = if finding.check.is_empty() {
        finding.rule_id.clone()
    } else {
        format!("{}/{}", finding.rule_id, finding.check)
    };
    let location = finding
        .location
        .as_deref()
        .map(|location| format!("  {}", location_link(style, base, location)))
        .unwrap_or_default();
    println!("  {severity}  {}{location}", style.cyan(&check));
    for line in finding.message.lines() {
        println!("      {line}");
    }
}

// Renders a finding's location as a terminal hyperlink to the absolute file
// (display text stays the short repository-relative form), so it opens
// correctly regardless of the shell's current directory.
fn location_link(style: &Style, base: &Path, location: &str) -> String {
    let text = style.dim(&format!("({location})"));
    style.hyperlink(&location_url(base, location), &text)
}

fn location_url(base: &Path, location: &str) -> String {
    file_url(&base.join(Finding::normalize_location(location)))
}

fn count_summary(errors: usize, warnings: usize) -> String {
    format!(
        "{}, {}",
        plural(errors, "error"),
        plural(warnings, "warning")
    )
}
