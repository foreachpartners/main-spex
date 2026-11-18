use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use anstream::println;
use clap::Args;

use crate::config::Package;
use crate::config::spex::Spex;
use crate::error::Result;
use crate::rule::{self, Check, CheckKind, Rule, RunScope};
use crate::spec::{AcId, SpecId};
use crate::support::output::{OutputFormat, print_json};

use super::baseline::{BaselineFile, drop_baselined_findings, fill_preexisting_errors};
use super::conformance::validate_conformance;
use super::evidence::{RuleExecution, report_json};
use super::finding::{Finding, Source};
use super::report::{CheckReport, ReportTarget, report};
use super::runners::{
    Outcome, locate_script, outcome_findings, require_commands, run_ast_grep, run_glob,
    run_per_file_script, run_regexp, run_script_once, scope_files,
};
use super::scoped_file::ScopedFile;
use super::select::select_packages;
use super::snapshot::Snapshot;
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
    #[arg(long, value_enum, default_value = "text", help = "Report format")]
    format: OutputFormat,
    #[arg(
        long,
        conflicts_with = "fill_preexisting_errors",
        help = "Gate only findings absent from a JSON baseline"
    )]
    baseline: Option<PathBuf>,
    #[arg(
        long,
        conflicts_with_all = ["baseline", "fill_preexisting_errors", "files"],
        help = "Write a JSON baseline to a new file"
    )]
    write_baseline: Option<PathBuf>,
    #[arg(
        long,
        conflicts_with = "format",
        help = "Record the current error findings as a per-package baseline in spex.yaml, or in the spex.package.yaml of each selected package with --package"
    )]
    fill_preexisting_errors: bool,
}

pub(super) struct PackageCtx<'a> {
    pub(super) package_path: &'a Path,
    pub(super) spex_dir: &'a Path,
    pub(super) candidates: &'a [ScopedFile],
}

type ProjectChecks<'a> = Vec<(&'a Rule, &'a Check)>;

impl PackageCtx<'_> {
    fn run_check(
        &self,
        rule: &Rule,
        check: &Check,
        executions: &mut Option<Vec<RuleExecution>>,
    ) -> Vec<Finding> {
        let files = scope_files(check, self.candidates);
        let outcome = files
            .as_ref()
            .map_err(Clone::clone)
            .and_then(|files| self.check_hits(rule, check, files));
        if let Some(executions) = executions {
            executions.push(RuleExecution::new(
                self.package_path,
                format!("{}/{}", rule.id, check.name),
                &check.kind,
                files.as_deref().unwrap_or_default(),
                &outcome,
            ));
        }
        outcome_findings(rule.id.as_str(), check, outcome)
    }

    fn check_hits(&self, rule: &Rule, check: &Check, files: &[&ScopedFile]) -> Outcome {
        match &check.kind {
            CheckKind::Regexp { pattern, negate } => run_regexp(check, pattern, *negate, files),
            CheckKind::Glob { pattern, negate } => run_glob(check, pattern, *negate, files),
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
                files,
            ),
            CheckKind::Script {
                script,
                run,
                requires,
                timeout_seconds,
            } => {
                if *run == RunScope::PerFile && files.is_empty() {
                    return Ok(Vec::new());
                }
                let program = locate_script(&rule.script_dir(), script)?;
                require_commands(requires)?;
                let timeout = timeout_seconds.map(|seconds| Duration::from_secs(seconds.get()));
                match run {
                    RunScope::PerFile => run_per_file_script(
                        &program,
                        self.package_path,
                        self.spex_dir,
                        files,
                        timeout,
                    ),
                    RunScope::PerPackage | RunScope::Project => {
                        run_script_once(&program, self.package_path, self.spex_dir, timeout)
                    }
                }
            }
        }
    }
}

struct CheckPackageArgs<'a> {
    package: &'a Package,
    spex_dir: &'a Path,
    agent_root: &'a Path,
    rules: &'a [Rule],
    known_ids: &'a HashMap<SpecId, HashSet<AcId>>,
    files: &'a [String],
}

fn defer_project_check<'a>(
    project_checks: &mut ProjectChecks<'a>,
    rule: &'a Rule,
    check: &'a Check,
) {
    let already_deferred = project_checks.iter().any(|(deferred_rule, deferred)| {
        deferred_rule.id == rule.id && deferred.name == check.name
    });
    if !already_deferred {
        project_checks.push((rule, check));
    }
}

fn run_project_checks(
    spex_dir: &Path,
    project_checks: &ProjectChecks,
    executions: &mut Option<Vec<RuleExecution>>,
) -> Vec<Finding> {
    let ctx = PackageCtx {
        package_path: spex_dir,
        spex_dir,
        candidates: &[],
    };
    project_checks
        .iter()
        .flat_map(|(rule, check)| ctx.run_check(rule, check, executions))
        .collect()
}

pub fn run(args: CheckArgs, spex: Spex) -> Result<ExitCode> {
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
    let snapshot = args
        .baseline
        .as_deref()
        .map(|path| Snapshot::load(path, &spex.project_name, &selected))
        .transpose()?;
    let mut executions = matches!(args.format, OutputFormat::Json).then(Vec::new);
    let mut conformance =
        validate_conformance(&spex.dir, &spex.prd, &spex.packages, &specs, &rules)?;

    let mut reports = Vec::with_capacity(selected.len());
    let mut project_checks = ProjectChecks::new();
    for &package in &selected {
        let mut findings = check_package(
            CheckPackageArgs {
                package,
                spex_dir: &spex.dir,
                agent_root: &spex.agent_root,
                rules: &rules,
                known_ids: &known_ids,
                files: &args.files,
            },
            &mut project_checks,
            &mut executions,
        )?;
        if package.is_spex_dir {
            findings.append(&mut conformance);
        }
        reports.push(CheckReport {
            target: ReportTarget::Package(package),
            findings,
        });
    }

    if !selected.iter().any(|package| package.is_spex_dir)
        && let Some(spex_package) = spex.packages.iter().find(|package| package.is_spex_dir)
    {
        reports.push(CheckReport {
            target: ReportTarget::Package(spex_package),
            findings: conformance,
        });
    }

    if !project_checks.is_empty() {
        reports.push(CheckReport {
            target: ReportTarget::Project {
                spex_dir: &spex.dir,
            },
            findings: run_project_checks(&spex.dir, &project_checks, &mut executions),
        });
    }

    if args.fill_preexisting_errors {
        let file = if args.package.is_empty() {
            BaselineFile::SpexYaml
        } else {
            BaselineFile::PackageYaml
        };
        return fill_preexisting_errors(&spex.dir, &selected, &reports, file);
    }

    if let Some(path) = args.write_baseline {
        Snapshot::save(&path, &spex.project_name, &selected, &reports)?;
        match args.format {
            OutputFormat::Text => println!("Wrote baseline {}", path.display()),
            OutputFormat::Json => print_json(&serde_json::json!({
                "schema_version": 1,
                "command": "check",
                "baseline_written": path,
            }))?,
        }
        return Ok(ExitCode::SUCCESS);
    }

    let explicit_baseline = snapshot.is_some();
    let inherited = match snapshot {
        Some(snapshot) => snapshot.separate(&mut reports),
        None => drop_baselined_findings(&mut reports, &selected),
    };
    if let Some(executions) = executions {
        return report_json(
            &spex,
            &selected,
            &args.files,
            &reports,
            &inherited,
            &executions,
        );
    }
    let inherited_count =
        explicit_baseline.then(|| inherited.iter().map(|report| report.findings.len()).sum());
    Ok(report(&spex.project_name, &reports, inherited_count))
}

fn check_package<'a>(
    args: CheckPackageArgs<'a>,
    project_checks: &mut ProjectChecks<'a>,
    executions: &mut Option<Vec<RuleExecution>>,
) -> Result<Vec<Finding>> {
    let CheckPackageArgs {
        package,
        spex_dir,
        agent_root,
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
        ScopedFile::collect(package)?
    } else {
        ScopedFile::provided_not_excluded(package, spex_dir, files)?
    };

    let ctx = PackageCtx {
        package_path,
        spex_dir,
        candidates: &candidates,
    };

    let mut findings = validate_annotations(&ctx, agent_root, known_ids);

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
                if let Some(executions) = executions {
                    executions.push(RuleExecution::skipped(
                        package_path,
                        format!("{}/{}", rule.id, check.name),
                    ));
                }
                continue;
            }
            if check.kind.is_project_scoped() {
                defer_project_check(project_checks, rule, check);
            } else {
                findings.extend(ctx.run_check(rule, check, executions));
            }
        }
    }

    Ok(findings)
}
