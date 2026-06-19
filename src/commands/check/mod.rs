use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Args;
use error_stack::{Report, ResultExt};
use thiserror::Error;

use crate::config::{Config, Repository};
use crate::tech_spec::{self, Check, CheckKind, RunScope, Severity, TechSpec};
use crate::util;
use crate::workspace::Workspace;

mod report;
mod runners;

use report::report;
use runners::*;

#[cfg(test)]
#[path = "check_test.rs"]
mod check_test;

type Result<T> = core::result::Result<T, Report<CheckError>>;

#[derive(Debug, Error)]
pub enum CheckError {
    #[error("could not load the spex workspace")]
    Workspace,
    #[error("unknown workspace repository `{0}` — not listed in spex.yaml")]
    UnknownRepo(String),
    #[error("could not walk workspace repository files")]
    Walk,
}

#[derive(Args)]
pub struct CheckArgs {
    #[arg(long, num_args = 1.., help = "Workspace repository names to check (default: all)")]
    repo: Vec<String>,
    #[arg(long, num_args = 1.., help = "Files to check (default: all non-ignored files)")]
    files: Vec<String>,
}

struct ScopedFile {
    abs: PathBuf,
    rel: PathBuf,
}

// Read-only context for checking a single repository, threaded through the
// dispatch path so individual checks don't each take the same arguments.
struct RepoCtx<'a> {
    repo: &'a Repository,
    repo_path: &'a Path,
    spex_dir: &'a Path,
    candidates: &'a [ScopedFile],
}

// A `workspace`-scoped script check, deferred until every repository has been
// scanned so it runs once across the whole workspace. `files` are paths from the
// spex directory (the script's working directory) to each in-scope file.
struct WorkspaceCheck {
    spec_id: String,
    check: Check,
    files: Vec<PathBuf>,
}

struct Finding {
    severity: Severity,
    spec_id: String,
    check: String,
    location: Option<String>,
    message: String,
}

struct RepoReport {
    name: String,
    findings: Vec<Finding>,
}

pub fn run(args: CheckArgs) -> Result<ExitCode> {
    let workspace = Workspace::discover().change_context(CheckError::Workspace)?;
    let specs = workspace
        .tech_specs()
        .change_context(CheckError::Workspace)?;

    let selected = select_repos(&workspace.config, &args.repo)?;

    let mut reports = Vec::with_capacity(selected.len());
    let mut workspace_checks: Vec<WorkspaceCheck> = Vec::new();
    for repo in selected {
        let mut findings = Vec::new();
        check_repo(
            repo,
            &workspace.dir,
            &specs,
            &args.files,
            &mut findings,
            &mut workspace_checks,
        )?;
        reports.push(RepoReport {
            name: repo.name.clone(),
            findings,
        });
    }

    if !workspace_checks.is_empty() {
        let mut findings = Vec::new();
        run_workspace_checks(&workspace.dir, &workspace_checks, &mut findings);
        reports.push(RepoReport {
            name: "workspace".to_string(),
            findings,
        });
    }

    Ok(report(&workspace.config, &reports))
}

fn select_repos<'a>(config: &'a Config, names: &[String]) -> Result<Vec<&'a Repository>> {
    if names.is_empty() {
        return Ok(config.repositories.iter().collect());
    }

    let mut selected = Vec::with_capacity(names.len());
    for name in names {
        let repo = config
            .repositories
            .iter()
            .find(|repo| &repo.name == name)
            .ok_or_else(|| Report::new(CheckError::UnknownRepo(name.clone())))?;
        selected.push(repo);
    }
    Ok(selected)
}

fn check_repo(
    repo: &Repository,
    spex_dir: &Path,
    specs: &HashMap<String, TechSpec>,
    files: &[String],
    findings: &mut Vec<Finding>,
    workspace_checks: &mut Vec<WorkspaceCheck>,
) -> Result<()> {
    let repo_path = spex_dir.join(&repo.path);
    if !repo_path.is_dir() {
        findings.push(repo_error(
            &repo.name,
            format!(
                "workspace repository path not found: {}",
                repo_path.display()
            ),
        ));
        return Ok(());
    }

    let candidates = if files.is_empty() {
        collect_files(&repo_path)?
    } else {
        provided_files(&repo_path, files)
    };

    let ctx = RepoCtx {
        repo,
        repo_path: &repo_path,
        spex_dir,
        candidates: &candidates,
    };

    let overridden: HashSet<(&str, &str)> = repo
        .checks_override
        .iter()
        .map(|over| (over.spec.as_str(), over.name.as_str()))
        .collect();

    // Inherited checks identified by `(spec_id, name)`, used to resolve overrides.
    let mut inherited: HashMap<(&str, &str), &Check> = HashMap::new();

    for spec_id in &repo.tech_specs {
        let Some(spec) = specs.get(spec_id) else {
            findings.push(repo_error(
                spec_id,
                format!(
                    "tech spec `{spec_id}` not found in {}/",
                    tech_spec::TECH_SPECS_DIR
                ),
            ));
            continue;
        };

        for check in &spec.checks {
            inherited.insert((spec_id.as_str(), check.name.as_str()), check);
            if overridden.contains(&(spec_id.as_str(), check.name.as_str())) {
                continue;
            }
            dispatch_check(&ctx, &spec.id, check, findings, workspace_checks);
        }
    }

    for over in &repo.checks_override {
        let Some(base) = inherited.get(&(over.spec.as_str(), over.name.as_str())) else {
            findings.push(repo_error(
                &repo.name,
                format!(
                    "checks_override targets `{}/{}`, which is not an inherited check",
                    over.spec, over.name
                ),
            ));
            continue;
        };

        if over.disable {
            continue;
        }

        let effective = over.apply(base);
        dispatch_check(&ctx, &over.spec, &effective, findings, workspace_checks);
    }

    for additional in &repo.checks_additional {
        if !repo.tech_specs.iter().any(|spec| spec == &additional.spec) {
            findings.push(repo_error(
                &repo.name,
                format!(
                    "checks_additional `{}/{}` references spec `{}`, which is not in this repository's tech_specs",
                    additional.spec, additional.check.name, additional.spec
                ),
            ));
            continue;
        }
        dispatch_check(
            &ctx,
            &additional.spec,
            &additional.check,
            findings,
            workspace_checks,
        );
    }

    Ok(())
}

fn collect_files(repo_path: &Path) -> Result<Vec<ScopedFile>> {
    let mut files = Vec::new();
    for result in util::repo_walker(repo_path) {
        let entry = result.change_context(CheckError::Walk)?;
        if !entry.file_type().is_some_and(|kind| kind.is_file()) {
            continue;
        }
        let abs = entry.into_path();
        let rel = abs.strip_prefix(repo_path).unwrap_or(&abs).to_path_buf();
        files.push(ScopedFile { abs, rel });
    }
    Ok(files)
}

fn provided_files(repo_path: &Path, files: &[String]) -> Vec<ScopedFile> {
    files
        .iter()
        .filter_map(|file| {
            let rel = PathBuf::from(file.trim_start_matches("./"));
            let abs = repo_path.join(&rel);
            abs.is_file().then_some(ScopedFile { abs, rel })
        })
        .collect()
}

fn dispatch_check(
    ctx: &RepoCtx,
    spec_id: &str,
    check: &Check,
    findings: &mut Vec<Finding>,
    workspace_checks: &mut Vec<WorkspaceCheck>,
) {
    if check.kind == CheckKind::Script && check.run == RunScope::Workspace {
        collect_workspace_check(ctx, spec_id, check, findings, workspace_checks);
    } else {
        run_check(ctx, spec_id, check, findings);
    }
}

// Defers a `workspace`-scoped check: scopes this repository's files, then merges
// them (as paths from the spex directory) into the pending check identified by
// `(spec_id, name)` so the script runs once over the whole workspace.
fn collect_workspace_check(
    ctx: &RepoCtx,
    spec_id: &str,
    check: &Check,
    findings: &mut Vec<Finding>,
    workspace_checks: &mut Vec<WorkspaceCheck>,
) {
    let scoped = match scope_files(check, ctx.candidates) {
        Ok(scoped) => scoped,
        Err(message) => {
            findings.push(config_error(spec_id, &check.name, message));
            return;
        }
    };

    let prefix = Path::new(&ctx.repo.path);
    let scoped_files = scoped.iter().map(|file| prefix.join(&file.rel));

    if let Some(pending) = workspace_checks
        .iter_mut()
        .find(|pending| pending.spec_id == spec_id && pending.check.name == check.name)
    {
        pending.files.extend(scoped_files);
    } else {
        workspace_checks.push(WorkspaceCheck {
            spec_id: spec_id.to_string(),
            check: check.clone(),
            files: scoped_files.collect(),
        });
    }
}

fn message(check: &Check, default: &str) -> String {
    check
        .description
        .clone()
        .unwrap_or_else(|| default.to_string())
}

fn rel_str(file: &ScopedFile) -> String {
    file.rel.to_string_lossy().into_owned()
}

fn violation(check: &Check, spec_id: &str, location: Option<String>, message: String) -> Finding {
    Finding {
        severity: check.severity,
        spec_id: spec_id.to_string(),
        check: check.name.clone(),
        location,
        message,
    }
}

fn config_error(spec_id: &str, check: &str, message: String) -> Finding {
    Finding {
        severity: Severity::Error,
        spec_id: spec_id.to_string(),
        check: check.to_string(),
        location: None,
        message,
    }
}

// A repository-level error not tied to a specific named check (e.g. a missing
// spec or a misconfigured override).
fn repo_error(spec_id: &str, message: String) -> Finding {
    config_error(spec_id, "", message)
}
