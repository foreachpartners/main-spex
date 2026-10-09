use std::path::Path;
use std::process::ExitCode;

use anstream::println;
use owo_colors::OwoColorize;

use crate::config::spex::Spex;
use crate::rule::Severity;
use crate::support::fs::file_url;
use crate::support::style::{hyperlink, version_line};
use crate::support::util::plural;

use super::{Finding, RepoReport};

#[cfg(test)]
#[path = "report_test.rs"]
mod report_test;

pub(super) fn report(spex: &Spex, reports: &[RepoReport]) -> ExitCode {
    println!("{}", version_line());
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

        let base = spex.repo_base_dir(&repo.name);

        println!("{}", repo.name.bold());
        if repo.findings.is_empty() {
            println!("  {}", "no issues".green());
        } else {
            for finding in &repo.findings {
                print_finding(base, finding);
            }
            println!("  {}", count_summary(errors, warnings).dimmed());
        }
        println!();
    }

    let total = count_summary(total_errors, total_warnings);
    println!("{}: {}", spex.project_name.bold(), total.bold());

    if total_errors > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn print_finding(base: &Path, finding: &Finding) {
    let label: &str = finding.severity.into();
    let severity = match finding.severity {
        Severity::Error => label.red().to_string(),
        Severity::Warn => label.yellow().to_string(),
    };
    let check = if finding.check.is_empty() {
        finding.rule_id.clone()
    } else {
        format!("{}/{}", finding.rule_id, finding.check)
    };
    let location = finding
        .location
        .as_deref()
        .map(|location| format!("  {}", location_link(base, location)))
        .unwrap_or_default();
    println!("  {severity}  {}{location}", check.cyan());
    for line in finding.message.lines() {
        println!("      {line}");
    }
}

// Renders a finding's location as a terminal hyperlink to the absolute file
// (display text stays the short repository-relative form), so it opens
// correctly regardless of the shell's current directory.
fn location_link(base: &Path, location: &str) -> String {
    let text = format!("({location})").dimmed().to_string();
    hyperlink(&location_url(base, location), &text)
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
