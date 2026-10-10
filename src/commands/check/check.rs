use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::ExitCode;

use clap::Args;

use crate::config::Repository;
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
use super::select::select_repos;
use super::spec_id_tracing::validate_annotations;

#[cfg(test)]
#[path = "check_test.rs"]
mod check_test;

#[derive(Args)]
pub struct CheckArgs {
    #[arg(long, num_args = 1.., help = "Workspace repository names to include (default: all)")]
    repo: Vec<String>,
    #[arg(long, num_args = 1.., help = "Files to check (default: all non-ignored files)")]
    files: Vec<String>,
    #[arg(
        long,
        help = "Record the current error findings as a per-repository baseline in spex.yaml"
    )]
    fill_preexisting_errors: bool,
}

// Read-only context for checking a single repository, threaded through the
// dispatch path so individual checks don't each take the same arguments.
pub(super) struct RepoCtx<'a> {
    pub(super) repo_path: &'a Path,
    pub(super) spex_dir: &'a Path,
    pub(super) candidates: &'a [ScopedFile],
}

type WorkspaceChecks<'a> = Vec<(&'a Rule, &'a Check)>;

impl RepoCtx<'_> {
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
                        run_per_file_script(&program, self.repo_path, self.spex_dir, &files)
                    }
                    RunScope::PerRepo | RunScope::Workspace => {
                        run_script_once(&program, self.repo_path, self.spex_dir)
                    }
                }
            }
        }
    }
}

// Read-only inputs for `check_repo`, grouped so callers can't accidentally
// transpose same-typed positional arguments.
struct CheckRepoArgs<'a> {
    repo: &'a Repository,
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
    let ctx = RepoCtx {
        repo_path: spex_dir,
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

    let selected = select_repos(&spex, &args.repo, &args.files)?;

    let mut reports = Vec::with_capacity(selected.len());
    let mut workspace_checks = WorkspaceChecks::new();
    for &repo in &selected {
        let findings = check_repo(
            CheckRepoArgs {
                repo,
                spex_dir: &spex.dir,
                rules: &rules,
                known_ids: &known_ids,
                files: &args.files,
            },
            &mut workspace_checks,
        )?;
        reports.push(CheckReport {
            target: ReportTarget::Repository(repo),
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

    validate_specs(&spex, &specs, &rules, &mut reports)?;

    if args.fill_preexisting_errors {
        return fill_preexisting_errors(&spex.dir, &selected, &reports);
    }

    drop_baselined_findings(&mut reports, &selected);
    Ok(report(&spex.project_name, &reports))
}

fn check_repo<'a>(
    args: CheckRepoArgs<'a>,
    workspace_checks: &mut WorkspaceChecks<'a>,
) -> Result<Vec<Finding>> {
    let CheckRepoArgs {
        repo,
        spex_dir,
        rules,
        known_ids,
        files,
    } = args;

    let repo_path = &repo.path;
    if !repo_path.is_dir() {
        return Ok(vec![Finding::error(
            Source::UnreachableRepository {
                name: repo.repository_name.clone(),
            },
            None,
            format!(
                "workspace repository path not found: {}",
                repo_path.display()
            ),
        )]);
    }

    let candidates = if files.is_empty() {
        ScopedFile::collect(repo_path)?
    } else {
        ScopedFile::provided(repo_path, files)
    };

    let ctx = RepoCtx {
        repo_path,
        spex_dir,
        candidates: &candidates,
    };

    let mut findings = validate_annotations(&ctx, repo.is_spex_repo, known_ids);

    // Negated checks assert that something must exist in the repository
    // (e.g. "dprint.json must be present"). When --files narrows the
    // candidate set to an explicit subset, a negated check cannot be
    // meaningfully evaluated, so we skip them to avoid false positives.
    let has_files = !files.is_empty();

    for rule_id in &repo.rules {
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
            if has_files && check.kind.negate() {
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

// Validates the spec set itself (statuses and `blocked_by` references) and
// attributes any findings to the spex repository, which owns the spec files.
// Runs only when the spex repository is in scope.
fn validate_specs(
    spex: &Spex,
    specs: &[Spec],
    rules: &[Rule],
    reports: &mut [CheckReport],
) -> Result<()> {
    let Some(spex_report) = reports
        .iter_mut()
        .find(|report| report.target.is_spex_repo())
    else {
        return Ok(());
    };

    spex_report.findings.extend(validate_conformance(
        &spex.dir,
        &spex.repositories,
        specs,
        rules,
    )?);
    Ok(())
}
