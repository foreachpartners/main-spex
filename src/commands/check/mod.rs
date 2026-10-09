use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Args;
use error_stack::ResultExt;

use crate::config::Repository;
use crate::config::spex::Spex;
use crate::error::{CliError, Result};
use crate::rule::{self, Check, CheckKind, Rule, RunScope};
use crate::spec::{AcId, Spec, SpecId};
use crate::support::repo_files::RepoFiles;

mod baseline;
mod conformance;
mod finding;
mod report;
mod runners;
mod select;
mod spec_id_tracing;

use baseline::fill_preexisting_errors;
use conformance::validate_conformance;
use finding::Finding;
use report::{CheckReport, ReportTarget, report};
use runners::{
    Outcome, locate_script, outcome_findings, run_ast_grep, run_glob, run_per_file_script,
    run_regexp, run_script_once, run_workspace_checks, scope_files,
};
use select::select_repos;
use spec_id_tracing::validate_annotations;

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

struct ScopedFile {
    abs: PathBuf,
    rel: PathBuf,
}

impl ScopedFile {
    fn collect(repo_path: &Path) -> Result<Vec<Self>> {
        let mut files = Vec::new();
        for result in RepoFiles::new(repo_path) {
            let abs = result.change_context_lazy(|| {
                CliError::Internal("could not walk workspace repository files".into())
            })?;
            let rel = abs.strip_prefix(repo_path).unwrap_or(&abs).to_path_buf();
            files.push(Self { abs, rel });
        }
        Ok(files)
    }

    fn provided(repo_path: &Path, files: &[String]) -> Vec<Self> {
        files
            .iter()
            .filter_map(|file| {
                let rel = PathBuf::from(file.trim_start_matches("./"));
                let abs = repo_path.join(&rel);
                abs.is_file().then_some(Self { abs, rel })
            })
            .collect()
    }

    fn rel_str(&self) -> String {
        self.rel.to_string_lossy().into_owned()
    }
}

// Read-only context for checking a single repository, threaded through the
// dispatch path so individual checks don't each take the same arguments.
struct RepoCtx<'a> {
    repo_path: &'a Path,
    spex_dir: &'a Path,
    candidates: &'a [ScopedFile],
}

impl RepoCtx<'_> {
    fn dispatch_check(
        &self,
        rule: &Rule,
        check: &Check,
        workspace_checks: &mut Vec<WorkspaceCheck>,
    ) -> Vec<Finding> {
        match &check.kind {
            CheckKind::Script {
                script,
                run: RunScope::Workspace,
            } => {
                WorkspaceCheck::collect(rule, check, script, workspace_checks);
                Vec::new()
            }
            _ => self.run_check(rule, check),
        }
    }

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
                    RunScope::PerRepo => run_script_once(&program, self.repo_path, self.spex_dir),
                    RunScope::Workspace => Ok(Vec::new()),
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

// A `workspace`-scoped script check, deferred until every repository has been
// scanned so it runs exactly once across the whole workspace. It receives no
// files; the script discovers its own from the spex directory.
struct WorkspaceCheck {
    rule_id: String,
    script_dir: PathBuf,
    script: String,
    check: Check,
}

impl WorkspaceCheck {
    // Defers a `workspace`-scoped check, deduplicating by `(rule_id, name)` so a
    // check inherited by several repositories still runs exactly once.
    fn collect(rule: &Rule, check: &Check, script: &str, workspace_checks: &mut Vec<Self>) {
        let already_pending = workspace_checks
            .iter()
            .any(|pending| pending.rule_id == rule.id.as_str() && pending.check.name == check.name);
        if !already_pending {
            workspace_checks.push(Self {
                rule_id: rule.id.to_string(),
                script_dir: rule.script_dir(),
                script: script.to_string(),
                check: check.clone(),
            });
        }
    }
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
    let mut workspace_checks: Vec<WorkspaceCheck> = Vec::new();
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

    drop_baselined_findings(&mut reports, &selected, &spex.dir);
    Ok(report(&spex.project_name, &reports))
}

fn drop_baselined_findings(reports: &mut [CheckReport], selected: &[&Repository], spex_dir: &Path) {
    for report in reports {
        match report.target {
            ReportTarget::Repository(repo) => report
                .findings
                .retain(|finding| !finding.is_preexisting(&repo.ignored_errors)),
            ReportTarget::Workspace { .. } => report
                .findings
                .retain(|finding| !finding.is_preexisting_workspace(selected, spex_dir)),
        }
    }
}

fn check_repo(
    args: CheckRepoArgs,
    workspace_checks: &mut Vec<WorkspaceCheck>,
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
        return Ok(vec![Finding::repo_error(
            &repo.repository_name,
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
            findings.push(Finding::repo_error(
                rule_id.as_str(),
                format!("rule `{rule_id}` not found in {}/", rule::RULES_DIR),
            ));
            continue;
        };

        for check in &rule.checks {
            if has_files && check.kind.negate() {
                continue;
            }
            findings.extend(ctx.dispatch_check(rule, check, workspace_checks));
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
