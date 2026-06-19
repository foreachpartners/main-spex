use std::process::ExitCode;

use crate::style::Style;
use crate::util::plural;

use super::*;

pub(super) fn report(config: &Config, reports: &[RepoReport]) -> ExitCode {
    let style = Style::detect();
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

        println!("{}", style.bold(&repo.name));
        if repo.findings.is_empty() {
            println!("  {}", style.green("no issues"));
        } else {
            for finding in &repo.findings {
                print_finding(&style, finding);
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
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn print_finding(style: &Style, finding: &Finding) {
    let severity = match finding.severity {
        Severity::Error => style.red("error"),
        Severity::Warn => style.yellow("warn"),
    };
    let check = if finding.check.is_empty() {
        finding.spec_id.clone()
    } else {
        format!("{}/{}", finding.spec_id, finding.check)
    };
    let location = finding
        .location
        .as_deref()
        .map(|location| format!("  {}", style.dim(&format!("({location})"))))
        .unwrap_or_default();
    println!("  {severity}  {}{location}", style.cyan(&check));
    for line in finding.message.lines() {
        println!("      {line}");
    }
}

fn count_summary(errors: usize, warnings: usize) -> String {
    format!(
        "{}, {}",
        plural(errors, "error"),
        plural(warnings, "warning")
    )
}
