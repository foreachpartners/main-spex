use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use clap::Args;
use error_stack::{Report, ResultExt};
use thiserror::Error;

use crate::config::{self, Config, PreexistingEntry, PreexistingError, Repository};
use crate::rule::{self, Check, CheckKind, Rule, RunScope, Severity};
use crate::spec::{self, Spec, Status};
use crate::style::{Log, Style};
use crate::util::{self, plural};
use crate::workspace::Workspace;

mod conformance;
mod report;
mod runners;

use conformance::validate_conformance;
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
    #[error("could not update `{}`", config::SPEX_CONFIG_FILE)]
    Rewrite,
}

#[derive(Args)]
pub struct CheckArgs {
    #[arg(long, num_args = 1.., help = "Workspace repository names to check (default: all)")]
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

// Read-only context for checking a single repository, threaded through the
// dispatch path so individual checks don't each take the same arguments.
struct RepoCtx<'a> {
    repo_path: &'a Path,
    spex_dir: &'a Path,
    candidates: &'a [ScopedFile],
    log: &'a Log,
}

// A `workspace`-scoped script check, deferred until every repository has been
// scanned so it runs exactly once across the whole workspace. It receives no
// files; the script discovers its own from the spex directory.
struct WorkspaceCheck {
    spec_id: String,
    check: Check,
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

pub fn run(args: CheckArgs, verbose: bool) -> Result<ExitCode> {
    let log = Log::new(verbose);
    let workspace = Workspace::discover().change_context(CheckError::Workspace)?;
    let rules = workspace.rules().change_context(CheckError::Workspace)?;

    let selected = select_repos(&workspace.config, &workspace.dir, &args.repo, &args.files)?;

    // The generator must see every current error, including ones an existing
    // baseline would otherwise suppress, so it can rewrite the full baseline.
    let suppress = !args.fill_preexisting_errors;

    let mut reports = Vec::with_capacity(selected.len());
    let mut workspace_checks: Vec<WorkspaceCheck> = Vec::new();
    for repo in &selected {
        let repo = *repo;
        let started = Instant::now();
        log.step(&format!("checking repository `{}`", repo.name));
        let mut findings = Vec::new();
        check_repo(
            repo,
            &workspace.dir,
            &rules,
            &args.files,
            &log,
            &mut findings,
            &mut workspace_checks,
        )?;
        log.step(&format!(
            "finished repository `{}` in {:.2?}",
            repo.name,
            started.elapsed()
        ));
        if suppress {
            findings.retain(|finding| !is_preexisting(finding, &repo.ignored_errors));
        }
        reports.push(RepoReport {
            name: repo.name.clone(),
            findings,
        });
    }

    if !workspace_checks.is_empty() {
        let mut findings = Vec::new();
        run_workspace_checks(&workspace.dir, &workspace_checks, &log, &mut findings);
        if suppress {
            findings.retain(|finding| !is_preexisting_workspace(finding, &selected));
        }
        reports.push(RepoReport {
            name: "workspace".to_string(),
            findings,
        });
    }

    validate_specs(&workspace, &rules, &selected, &mut reports)?;

    if args.fill_preexisting_errors {
        return fill_preexisting_errors(&workspace.dir, &selected, &reports);
    }

    Ok(report(&workspace.config, &workspace.dir, &reports))
}

// Either the whole check is baselined (a file-less error) or a specific set of
// files is. A file-less error widens the entry to the whole check.
enum Baseline {
    WholeCheck,
    Files(BTreeSet<String>),
}

impl Baseline {
    fn add_file(&mut self, file: Option<String>) {
        match (self, file) {
            (Baseline::WholeCheck, _) => {}
            (slot @ Baseline::Files(_), None) => *slot = Baseline::WholeCheck,
            (Baseline::Files(files), Some(file)) => {
                files.insert(file);
            }
        }
    }
}

// Records the current error findings into each selected repository's
// `ignored_errors` block in `spex.yaml`. File-less errors widen to a
// whole-check entry; repository-level errors with no check are skipped because
// they cannot be keyed. Out-of-scope repositories are left untouched.
fn fill_preexisting_errors(
    spex_dir: &Path,
    selected: &[&Repository],
    reports: &[RepoReport],
) -> Result<ExitCode> {
    // repository name -> (spec, check) -> baselined files
    let mut grouped: HashMap<String, BTreeMap<(String, String), Baseline>> = selected
        .iter()
        .map(|repo| (repo.name.clone(), BTreeMap::new()))
        .collect();

    for report in reports {
        for finding in &report.findings {
            if finding.severity != Severity::Error || finding.check.is_empty() {
                continue;
            }
            let (repo_name, file) = if report.name == "workspace" {
                match owning_repo(finding, selected) {
                    Some(resolved) => resolved,
                    None => continue,
                }
            } else {
                (report.name.clone(), finding_file(finding))
            };
            let Some(checks) = grouped.get_mut(&repo_name) else {
                continue;
            };
            checks
                .entry((finding.spec_id.clone(), finding.check.clone()))
                .or_insert_with(|| Baseline::Files(BTreeSet::new()))
                .add_file(file);
        }
    }

    let by_repo: HashMap<String, Vec<PreexistingEntry>> = grouped
        .into_iter()
        .map(|(name, checks)| {
            let entries = checks
                .into_iter()
                .map(|((rule_id, check), baseline)| PreexistingEntry {
                    rule_id,
                    check,
                    files: match baseline {
                        Baseline::WholeCheck => Vec::new(),
                        Baseline::Files(files) => files.into_iter().collect(),
                    },
                })
                .collect();
            (name, entries)
        })
        .collect();

    let config_path = spex_dir.join(config::SPEX_CONFIG_FILE);
    let raw = std::fs::read_to_string(&config_path).change_context(CheckError::Rewrite)?;
    let updated = config::set_preexisting_errors(&raw, &by_repo);
    let changed = updated != raw;
    if changed {
        std::fs::write(&config_path, &updated).change_context(CheckError::Rewrite)?;
    }

    report_fill(selected, &by_repo, changed);
    Ok(ExitCode::SUCCESS)
}

// Resolves a workspace finding to its owning repository and the repository-
// relative file, by matching its location against each repository's path.
fn owning_repo(finding: &Finding, repos: &[&Repository]) -> Option<(String, Option<String>)> {
    let location = finding.location.as_deref()?;
    repos.iter().find_map(|repo| {
        let prefix = format!("{}/", repo.path);
        location
            .strip_prefix(&prefix)
            .map(|rel| (repo.name.clone(), Some(normalize_location(rel))))
    })
}

fn report_fill(
    selected: &[&Repository],
    by_repo: &HashMap<String, Vec<PreexistingEntry>>,
    changed: bool,
) {
    let style = Style::detect();
    println!("{}", style.bold("spex check --fill-preexisting-errors"));
    println!();
    for repo in selected {
        let count = by_repo.get(&repo.name).map_or(0, Vec::len);
        let summary = if count == 0 {
            style.green("no pre-existing errors")
        } else {
            style.dim(&format!("baselined {}", plural(count, "check")))
        };
        println!("  {}  {summary}", style.bold(&repo.name));
    }
    println!();
    if changed {
        println!("Updated {}", config::SPEX_CONFIG_FILE);
    } else {
        println!("{} unchanged", config::SPEX_CONFIG_FILE);
    }
}

// Drops a finding when it matches a repository's pre-existing-error baseline: an
// `error`-severity finding whose `(rule_id, check)` matches an entry and whose
// location file is listed (or the entry lists no files, suppressing the whole
// check). Warnings and unmatched errors always survive.
fn is_preexisting(finding: &Finding, baseline: &[PreexistingError]) -> bool {
    finding.severity == Severity::Error
        && matches_baseline(
            baseline,
            &finding.spec_id,
            &finding.check,
            finding_file(finding).as_deref(),
        )
}

// Workspace findings carry a location prefixed by the owning repository's path
// (relative to the spex directory), so resolve each to its repository and match
// against that repository's baseline. There is no workspace-level baseline.
fn is_preexisting_workspace(finding: &Finding, repos: &[&Repository]) -> bool {
    if finding.severity != Severity::Error {
        return false;
    }
    let Some(location) = finding.location.as_deref() else {
        return false;
    };
    repos.iter().any(|repo| {
        let prefix = format!("{}/", repo.path);
        location.strip_prefix(&prefix).is_some_and(|rel| {
            matches_baseline(
                &repo.ignored_errors,
                &finding.spec_id,
                &finding.check,
                Some(&normalize_location(rel)),
            )
        })
    })
}

fn matches_baseline(
    baseline: &[PreexistingError],
    spec_id: &str,
    check: &str,
    file: Option<&str>,
) -> bool {
    baseline.iter().any(|entry| {
        entry.rule_id == spec_id
            && entry.check == check
            && (entry.files.is_empty()
                || file.is_some_and(|file| entry.files.iter().any(|listed| listed == file)))
    })
}

fn finding_file(finding: &Finding) -> Option<String> {
    finding.location.as_deref().map(normalize_location)
}

// Reduces a finding location to its file path by dropping a trailing `:<line>`,
// so a regexp finding's `file:line` matches a baseline entry keyed on `file`.
fn normalize_location(location: &str) -> String {
    if let Some((path, line)) = location.rsplit_once(':')
        && !line.is_empty()
        && line.bytes().all(|byte| byte.is_ascii_digit())
    {
        return path.to_string();
    }
    location.to_string()
}

// Selects the repositories to check, preserving `spex.yaml` order. Without
// `--files`, that is the `--repo` names (or every repository). With `--files`,
// it is every repository that owns at least one provided file, unioned with any
// explicitly named `--repo`, so a `per-repo` script runs only where the files
// live (plus any repository the user named outright).
fn select_repos<'a>(
    config: &'a Config,
    spex_dir: &Path,
    names: &[String],
    files: &[String],
) -> Result<Vec<&'a Repository>> {
    for name in names {
        if !config.repositories.iter().any(|repo| repo.matches(name)) {
            return Err(Report::new(CheckError::UnknownRepo(name.clone())));
        }
    }

    if files.is_empty() {
        return Ok(config
            .repositories
            .iter()
            .filter(|repo| names.is_empty() || names.iter().any(|name| repo.matches(name)))
            .collect());
    }

    Ok(config
        .repositories
        .iter()
        .filter(|repo| {
            names.iter().any(|name| repo.matches(name))
                || !provided_files(&spex_dir.join(&repo.path), files).is_empty()
        })
        .collect())
}

fn check_repo(
    repo: &Repository,
    spex_dir: &Path,
    rules: &HashMap<String, Rule>,
    files: &[String],
    log: &Log,
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
        repo_path: &repo_path,
        spex_dir,
        candidates: &candidates,
        log,
    };

    // Negated checks assert that something must exist in the repository
    // (e.g. "dprint.json must be present"). When --files narrows the
    // candidate set to an explicit subset, a negated check cannot be
    // meaningfully evaluated, so we skip them to avoid false positives.
    let has_files = !files.is_empty();

    for spec_id in &repo.rules {
        let Some(rule) = rules.get(spec_id) else {
            findings.push(repo_error(
                spec_id,
                format!("rule `{spec_id}` not found in {}/", rule::RULES_DIR),
            ));
            continue;
        };

        for check in &rule.checks {
            if has_files && check.kind.negate() {
                continue;
            }
            dispatch_check(&ctx, &rule.id, check, findings, workspace_checks);
        }
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
    if matches!(
        check.kind,
        CheckKind::Script {
            run: RunScope::Workspace,
            ..
        }
    ) {
        // Workspace checks only collect here; they run once in `run_workspace_checks`.
        collect_workspace_check(spec_id, check, workspace_checks);
    } else {
        let started = Instant::now();
        run_check(ctx, spec_id, check, findings);
        ctx.log.step(&format!(
            "  {spec_id}/{} ({:.2?})",
            check.name,
            started.elapsed()
        ));
    }
}

// Defers a `workspace`-scoped check, deduplicating by `(spec_id, name)` so a
// check inherited by several repositories still runs exactly once.
fn collect_workspace_check(
    spec_id: &str,
    check: &Check,
    workspace_checks: &mut Vec<WorkspaceCheck>,
) {
    let already_pending = workspace_checks
        .iter()
        .any(|pending| pending.spec_id == spec_id && pending.check.name == check.name);
    if !already_pending {
        workspace_checks.push(WorkspaceCheck {
            spec_id: spec_id.to_string(),
            check: check.clone(),
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

fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn is_spex_repo(repo: &Repository, spex_dir: &Path) -> bool {
    canonical(&spex_dir.join(&repo.path)) == canonical(spex_dir)
}

// Validates the spec set itself (statuses and `blocked_by` references) and
// attributes any findings to the spex repository, which owns the spec files.
// Runs only when the spex repository is in scope.
fn validate_specs(
    workspace: &Workspace,
    rules: &HashMap<String, Rule>,
    selected: &[&Repository],
    reports: &mut [RepoReport],
) -> Result<()> {
    let Some(spex_repo) = selected
        .iter()
        .find(|repo| is_spex_repo(repo, &workspace.dir))
    else {
        return Ok(());
    };
    let Some(report) = reports
        .iter_mut()
        .find(|report| report.name == spex_repo.name)
    else {
        return Ok(());
    };

    let specs = workspace.specs().change_context(CheckError::Workspace)?;

    let mut status_by_id: HashMap<&str, Status> = HashMap::new();
    for spec in &specs {
        status_by_id.insert(spec.id.as_str(), spec.status);
    }
    // Rules are global and always apply, so they count as satisfied for
    // `blocked_by` purposes.
    for rule in rules.values() {
        status_by_id.insert(rule.id.as_str(), Status::Active);
    }

    for spec in &specs {
        if let Some(finding) = status_finding(spec) {
            report.findings.push(finding);
        }
        if let Some(finding) = active_incomplete_finding(spec) {
            report.findings.push(finding);
        }
    }

    for spec in &specs {
        validate_blocked_by(spec, &status_by_id, &mut report.findings);
        validate_scope(spec, &workspace.config.repositories, &mut report.findings);
    }

    validate_conformance(&workspace.dir, rules, &mut report.findings);

    Ok(())
}

// The spec file's location relative to the workspace root (e.g.
// `specs/admin/foo.yaml`), honoring however the spec is nested under `specs/`.
fn spec_location(spec: &Spec) -> String {
    format!("{}/{}", spec::SPECS_DIR, spec.path)
}

// A spec with every acceptance criterion checked off is implemented, so its
// status must say so. Specs with no criteria or any unchecked box are exempt.
fn status_finding(spec: &Spec) -> Option<Finding> {
    let (done, total) = spec.ac_counts();
    if total == 0 || done != total || spec.status.is_implemented() {
        return None;
    }
    Some(Finding {
        severity: Severity::Warn,
        spec_id: spec.id.clone(),
        check: "status".to_string(),
        location: Some(spec_location(spec)),
        message: format!(
            "all acceptance criteria are checked off but status is `{}` — set status to active or deprecated",
            spec.status.label()
        ),
    })
}

// `active` means fully implemented, so it requires every acceptance
// criterion checked off; a spec still being worked on belongs in
// `in-progress` until then.
fn active_incomplete_finding(spec: &Spec) -> Option<Finding> {
    let (done, total) = spec.ac_counts();
    if spec.status != Status::Active || done == total {
        return None;
    }
    Some(Finding {
        severity: Severity::Error,
        spec_id: spec.id.clone(),
        check: "status".to_string(),
        location: Some(spec_location(spec)),
        message: format!(
            "status is `active` but only {done}/{total} acceptance criteria are checked off — use `in-progress` until all are done"
        ),
    })
}

fn validate_blocked_by(
    spec: &Spec,
    status_by_id: &HashMap<&str, Status>,
    findings: &mut Vec<Finding>,
) {
    for blocker in &spec.blocked_by {
        let location = Some(spec_location(spec));
        match status_by_id.get(blocker.as_str()) {
            None => findings.push(Finding {
                severity: Severity::Error,
                spec_id: spec.id.clone(),
                check: "blocked-by".to_string(),
                location,
                message: format!("`blocked_by` references `{blocker}`, a non-existing spec"),
            }),
            Some(Status::Active) => findings.push(Finding {
                severity: Severity::Warn,
                spec_id: spec.id.clone(),
                check: "blocked-by".to_string(),
                location,
                message: format!(
                    "`blocked_by` references `{blocker}`, which is already active — remove satisfied blockers"
                ),
            }),
            Some(_) => {}
        }
    }
}

fn validate_scope(spec: &Spec, repositories: &[Repository], findings: &mut Vec<Finding>) {
    for short_name in &spec.scope {
        if repositories
            .iter()
            .any(|repo| &repo.short_name == short_name)
        {
            continue;
        }
        findings.push(Finding {
            severity: Severity::Error,
            spec_id: spec.id.clone(),
            check: "scope".to_string(),
            location: Some(spec_location(spec)),
            message: format!(
                "`scope` references `{short_name}`, which does not match any repository_short_name in spex.yaml"
            ),
        });
    }
}
