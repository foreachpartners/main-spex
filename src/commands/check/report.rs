use std::path::Path;
use std::process::ExitCode;

use anstream::println;
use owo_colors::OwoColorize;

use crate::config::Package;
use crate::rule::Severity;
use crate::support::fs::file_url;
use crate::support::style::{hyperlink, version_line};
use crate::support::text::plural;

use super::finding::{Finding, Location};

#[cfg(test)]
#[path = "report_test.rs"]
mod report_test;

pub(super) enum ReportTarget<'a> {
    Package(&'a Package),
    Workspace { spex_dir: &'a Path },
    Spex { spex_dir: &'a Path },
}

impl ReportTarget<'_> {
    pub(super) fn name(&self) -> &str {
        match self {
            Self::Package(package) => &package.package_name,
            Self::Workspace { .. } => "workspace",
            Self::Spex { .. } => "spex",
        }
    }

    pub(super) fn base_dir(&self) -> &Path {
        match self {
            Self::Package(package) => &package.path,
            Self::Workspace { spex_dir } | Self::Spex { spex_dir } => spex_dir,
        }
    }

    pub(super) fn is_spex_dir(&self) -> bool {
        matches!(self, Self::Package(package) if package.is_spex_dir)
    }
}

pub(super) struct CheckReport<'a> {
    pub(super) target: ReportTarget<'a>,
    pub(super) findings: Vec<Finding>,
}

pub(super) fn report(project_name: &str, reports: &[CheckReport]) -> ExitCode {
    println!("{}", version_line());
    println!();
    let mut total_errors = 0usize;
    let mut total_warnings = 0usize;

    for CheckReport { target, findings } in reports {
        let errors = findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .count();
        let warnings = findings.len() - errors;
        total_errors += errors;
        total_warnings += warnings;

        println!("{}", target.name().bold());
        if findings.is_empty() {
            println!("  {}", "no issues".green());
        } else {
            for finding in findings {
                print_finding(target.base_dir(), finding);
            }
            println!("  {}", count_summary(errors, warnings).dimmed());
        }
        println!();
    }

    let total = count_summary(total_errors, total_warnings);
    println!("{}: {}", project_name.bold(), total.bold());

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
    let location = finding
        .location
        .as_ref()
        .map(|location| format!("  {}", location_link(base, location)))
        .unwrap_or_default();
    println!("  {severity}  {}{location}", finding.source.cyan());
    for line in finding.message.lines() {
        println!("      {line}");
    }
}

fn location_link(base: &Path, location: &Location) -> String {
    let text = format!("({location})").dimmed().to_string();
    hyperlink(&location_url(base, location), &text)
}

fn location_url(base: &Path, location: &Location) -> String {
    file_url(&base.join(&location.file))
}

fn count_summary(errors: usize, warnings: usize) -> String {
    format!(
        "{}, {}",
        plural(errors, "error"),
        plural(warnings, "warning")
    )
}
