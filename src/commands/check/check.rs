use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::ExitCode;

use clap::Args;

use crate::config::Package;
use crate::config::spex::Spex;
use crate::error::Result;
use crate::rule::{self, Check, CheckKind, Rule, RunScope};
use crate::spec::{AcId, Spec, SpecId};

use super::baseline::{drop_baselined_findings, fill_preexisting_errors};
use super::conformance::validate_conformance;
use super::finding::{Finding, Source};
use super::report::{CheckReport, ReportTarget, report};
use super::runners::{
    Outcome, locate_script, outcome_findings, run_ast_grep, run_glob, run_per_file_script,
    run_regexp, run_script_once, scope_files,
};
use super::scoped_file::ScopedFile;
use super::select::select_packages;
use super::spec_id_tracing::validate_annotations;

#[cfg(test)]
#[path = "check_test.rs"]
mod check_test;

#[derive(Args)]
pub struct CheckArgs {
    #[arg(long, num_args = 1.., help = "package names to include (default: all)")]
    package: Vec<String>,
    #[arg(long, num_args = 1.., help = "Files to check (default: all non-ignored files)")]
    files: Vec<String>,
    #[arg(
        long,
        help = "Record the current error findings as a per-package baseline in spex.yaml"
    )]
    fill_preexisting_errors: bool,
}

pub(super) struct PackageCtx<'a> {
    pub(super) package_path: &'a Path,
    pub(super) spex_dir: &'a Path,
    pub(super) candidates: &'a [ScopedFile],
}

type WorkspaceChecks<'a> = Vec<(&'a Rule, &'a Check)>;

impl PackageCtx<'_> {
    fn run_check(&self, rule: &Rule, check: &Check) -> Vec<Finding> {
        outcome_findings(rule.id.as_str(), check, self.check_hits(rule, check))
    }

    fn check_hits(&self, rule: &Rule, check: &Check) -> Outcome {
        let files = scope_files(check, self.candidates)?;

        match &check.kind {
            CheckKind::Regexp { pattern, negate } => run_regexp(check, pattern, *negate, &files),
            CheckKind::Glob { pattern, negate } => run_glob(check, pattern, *negate, &files),
            CheckKind::AstGrep {
                pattern,
                rule,
                language,
                negate,
            } => run_ast_grep(
                check,
                pattern.as_deref(),
                rule.as_ref(),
                language.as_deref(),
                *negate,
                &files,
            ),
            CheckKind::Script { script, run } => {
                let program = locate_script(&rule.script_dir(), script)?;
                match run {
                    RunScope::PerFile => {
                        run_per_file_script(&program, self.package_path, self.spex_dir, &files)
                    }
                    RunScope::PerPackage | RunScope::Workspace => {
                        run_script_once(&program, self.package_path, self.spex_dir)
                    }
                }
            }
        }
    }
}

struct CheckPackageArgs<'a> {
    package: &'a Package,
    spex_dir: &'a Path,
    rules: &'a [Rule],
    known_ids: &'a HashMap<SpecId, HashSet<AcId>>,
    files: &'a [String],
}

fn defer_workspace_check<'a>(
    workspace_checks: &mut WorkspaceChecks<'a>,
    rule: &'a Rule,
    check: &'a Check,
) {
    let already_deferred = workspace_checks.iter().any(|(deferred_rule, deferred)| {
        deferred_rule.id == rule.id && deferred.name == check.name
    });
    if !already_deferred {
        workspace_checks.push((rule, check));
    }
}

fn run_workspace_checks(spex_dir: &Path, workspace_checks: &WorkspaceChecks) -> Vec<Finding> {
    let ctx = PackageCtx {
        package_path: spex_dir,
        spex_dir,
        candidates: &[],
    };
    workspace_checks
        .iter()
        .flat_map(|(rule, check)| ctx.run_check(rule, check))
        .collect()
}

pub fn run(args: CheckArgs) -> Result<ExitCode> {
    let spex = Spex::discover()?;
    let rules = spex.rules()?;
    let specs = spex.specs()?;
    let known_ids: HashMap<SpecId, HashSet<AcId>> = specs
        .iter()
        .map(|spec| {
            let ac_ids = spec
                .acceptance_criteria
                .iter()
                .map(|ac| ac.id.clone())
                .collect();
            (spec.id.clone(), ac_ids)
        })
        .collect();

    let selected = select_packages(&spex, &args.package, &args.files)?;

    let mut reports = Vec::with_capacity(selected.len());
    let mut workspace_checks = WorkspaceChecks::new();
    for &package in &selected {
        let findings = check_package(
            CheckPackageArgs {
                package,
                spex_dir: &spex.dir,
                rules: &rules,
                known_ids: &known_ids,
                files: &args.files,
            },
            &mut workspace_checks,
        )?;
        reports.push(CheckReport {
            target: ReportTarget::Package(package),
            findings,
        });
    }

    if !workspace_checks.is_empty() {
        reports.push(CheckReport {
            target: ReportTarget::Workspace {
                spex_dir: &spex.dir,
            },
            findings: run_workspace_checks(&spex.dir, &workspace_checks),
        });
    }

    validate_spex_dir(&spex, &specs, &rules, &mut reports)?;

    if args.fill_preexisting_errors {
        return fill_preexisting_errors(&spex.dir, &selected, &reports);
    }

    drop_baselined_findings(&mut reports, &selected);
    Ok(report(&spex.project_name, &reports))
}

fn check_package<'a>(
    args: CheckPackageArgs<'a>,
    workspace_checks: &mut WorkspaceChecks<'a>,
) -> Result<Vec<Finding>> {
    let CheckPackageArgs {
        package,
        spex_dir,
        rules,
        known_ids,
        files,
    } = args;

    let package_path = &package.path;
    if !package_path.is_dir() {
        return Ok(vec![Finding::error(
            Source::UnreachablePackage {
                name: package.package_name.clone(),
            },
            None,
            format!("package path not found: {}", package_path.display()),
        )]);
    }

    let candidates = if files.is_empty() {
        ScopedFile::collect(package_path)?
    } else {
        ScopedFile::provided(package_path, files)
    };

    let ctx = PackageCtx {
        package_path,
        spex_dir,
        candidates: &candidates,
    };

    let mut findings = validate_annotations(&ctx, known_ids);

    let skip_negated_checks = !files.is_empty();

    for rule_id in &package.rules {
        let Some(rule) = rules.iter().find(|rule| rule.id == *rule_id) else {
            findings.push(Finding::error(
                Source::UnknownRule {
                    rule_id: rule_id.to_string(),
                },
                None,
                format!("rule `{rule_id}` not found in {}/", rule::RULES_DIR),
            ));
            continue;
        };

        for check in &rule.checks {
            if skip_negated_checks && check.kind.negate() {
                continue;
            }
            if check.kind.is_workspace_scoped() {
                defer_workspace_check(workspace_checks, rule, check);
            } else {
                findings.extend(ctx.run_check(rule, check));
            }
        }
    }

    Ok(findings)
}

fn validate_spex_dir<'a>(
    spex: &'a Spex,
    specs: &[Spec],
    rules: &[Rule],
    reports: &mut Vec<CheckReport<'a>>,
) -> Result<()> {
    let spex_report = reports
        .iter_mut()
        .find(|report| report.target.is_spex_dir());
    if spex_report.is_none() && spex.layout.is_multi_repo() {
        return Ok(());
    }

    let findings = validate_conformance(&spex.dir, &spex.packages, specs, rules)?;
    match spex_report {
        Some(report) => report.findings.extend(findings),
        None => reports.push(CheckReport {
            target: ReportTarget::Spex {
                spex_dir: &spex.dir,
            },
            findings,
        }),
    }
    Ok(())
}
