use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use clap::Args;
use error_stack::{Report, ResultExt};
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use regex::Regex;
use thiserror::Error;

use crate::config::{self, Config, Repository};
use crate::tech_spec::{self, Check, CheckKind, RunScope, Severity, TechSpec};

#[cfg(test)]
#[path = "check_test.rs"]
mod check_test;

// Cap files per script invocation so large repos don't blow past the OS arg limit.
const SCRIPT_BATCH_SIZE: usize = 1000;

type Result<T> = core::result::Result<T, Report<CheckError>>;

#[derive(Debug, Error)]
pub enum CheckError {
    #[error("could not determine the current directory")]
    CurrentDir,
    #[error("could not load spex configuration")]
    Config,
    #[error("could not load tech specs")]
    Specs,
    #[error("unknown workspace repository `{0}` — not listed in spex.yml")]
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
    let current_dir = std::env::current_dir().change_context(CheckError::CurrentDir)?;
    let spex_dir = config::find_dir(&current_dir).change_context(CheckError::Config)?;

    let config = config::load(&spex_dir).change_context(CheckError::Config)?;
    let specs = tech_spec::load_all(&spex_dir.join(tech_spec::TECH_SPECS_DIR))
        .change_context(CheckError::Specs)?;

    let selected = select_repos(&config, &args.repo)?;

    let mut reports = Vec::with_capacity(selected.len());
    for repo in selected {
        let mut findings = Vec::new();
        check_repo(repo, &spex_dir, &specs, &args.files, &mut findings)?;
        reports.push(RepoReport {
            name: repo.name.clone(),
            findings,
        });
    }

    Ok(report(&config, &reports))
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
) -> Result<()> {
    let repo_path = spex_dir.join(&repo.path);
    if !repo_path.is_dir() {
        findings.push(Finding {
            severity: Severity::Error,
            spec_id: repo.name.clone(),
            check: String::new(),
            location: None,
            message: format!(
                "workspace repository path not found: {}",
                repo_path.display()
            ),
        });
        return Ok(());
    }

    let candidates = if files.is_empty() {
        collect_files(&repo_path)?
    } else {
        provided_files(&repo_path, files)
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
            findings.push(Finding {
                severity: Severity::Error,
                spec_id: spec_id.clone(),
                check: String::new(),
                location: None,
                message: format!(
                    "tech spec `{spec_id}` not found in {}/",
                    tech_spec::TECH_SPECS_DIR
                ),
            });
            continue;
        };

        for check in &spec.checks {
            inherited.insert((spec_id.as_str(), check.name.as_str()), check);
            if overridden.contains(&(spec_id.as_str(), check.name.as_str())) {
                continue;
            }
            run_check(&spec.id, check, &repo_path, spex_dir, &candidates, findings);
        }
    }

    for over in &repo.checks_override {
        let Some(base) = inherited.get(&(over.spec.as_str(), over.name.as_str())) else {
            findings.push(Finding {
                severity: Severity::Error,
                spec_id: repo.name.clone(),
                check: String::new(),
                location: None,
                message: format!(
                    "checks_override targets `{}/{}`, which is not an inherited check",
                    over.spec, over.name
                ),
            });
            continue;
        };

        let effective = over.apply(base);
        let script_base = if over.script.is_some() {
            &repo_path
        } else {
            spex_dir
        };
        run_check(
            &over.spec,
            &effective,
            &repo_path,
            script_base,
            &candidates,
            findings,
        );
    }

    for additional in &repo.checks_additional {
        if !repo.tech_specs.iter().any(|spec| spec == &additional.spec) {
            findings.push(Finding {
                severity: Severity::Error,
                spec_id: repo.name.clone(),
                check: String::new(),
                location: None,
                message: format!(
                    "checks_additional `{}/{}` references spec `{}`, which is not in this repository's tech-specs",
                    additional.spec, additional.check.name, additional.spec
                ),
            });
            continue;
        }
        run_check(
            &additional.spec,
            &additional.check,
            &repo_path,
            &repo_path,
            &candidates,
            findings,
        );
    }

    Ok(())
}

fn collect_files(repo_path: &Path) -> Result<Vec<ScopedFile>> {
    let mut files = Vec::new();
    let walker = WalkBuilder::new(repo_path)
        .hidden(false) // include dotfiles (e.g. .env) that aren't gitignored
        .git_ignore(true) // respect .gitignore
        .git_global(true) // respect global gitignore
        .filter_entry(|entry| entry.file_name() != std::ffi::OsStr::new(".git"))
        .build();
    for result in walker {
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

fn run_check(
    spec_id: &str,
    check: &Check,
    repo_path: &Path,
    script_base: &Path,
    files: &[ScopedFile],
    findings: &mut Vec<Finding>,
) {
    let scoped = match scope_files(check, files) {
        Ok(scoped) => scoped,
        Err(message) => {
            findings.push(config_error(spec_id, &check.name, message));
            return;
        }
    };

    match check.kind {
        CheckKind::Regexp => run_regexp(spec_id, check, &scoped, findings),
        CheckKind::Glob => run_glob(spec_id, check, &scoped, findings),
        CheckKind::Script => run_script(spec_id, check, repo_path, script_base, &scoped, findings),
    }
}

fn scope_files<'a>(
    check: &Check,
    files: &'a [ScopedFile],
) -> core::result::Result<Vec<&'a ScopedFile>, String> {
    let include =
        build_globset(&check.include).map_err(|err| format!("invalid include glob: {err}"))?;
    let exclude =
        build_globset(&check.exclude).map_err(|err| format!("invalid exclude glob: {err}"))?;

    Ok(files
        .iter()
        .filter(|file| {
            if let Some(include) = &include
                && !include.is_match(&file.rel)
            {
                return false;
            }
            if let Some(exclude) = &exclude
                && exclude.is_match(&file.rel)
            {
                return false;
            }
            true
        })
        .collect())
}

fn build_globset(patterns: &[String]) -> core::result::Result<Option<GlobSet>, globset::Error> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    Ok(Some(builder.build()?))
}

fn run_regexp(spec_id: &str, check: &Check, files: &[&ScopedFile], findings: &mut Vec<Finding>) {
    let Some(pattern) = &check.pattern else {
        findings.push(config_error(
            spec_id,
            &check.name,
            "regexp check requires `pattern`".to_string(),
        ));
        return;
    };
    let regex = match Regex::new(pattern) {
        Ok(regex) => regex,
        Err(err) => {
            findings.push(config_error(
                spec_id,
                &check.name,
                format!("invalid regexp: {err}"),
            ));
            return;
        }
    };

    for file in files {
        let Ok(content) = std::fs::read_to_string(&file.abs) else {
            continue;
        };

        if check.negate {
            if !content.lines().any(|line| regex.is_match(line)) {
                findings.push(violation(
                    check,
                    spec_id,
                    Some(rel_str(file)),
                    message(check, "required pattern not found"),
                ));
            }
        } else {
            for (index, line) in content.lines().enumerate() {
                if regex.is_match(line) {
                    findings.push(violation(
                        check,
                        spec_id,
                        Some(format!("{}:{}", rel_str(file), index + 1)),
                        message(check, "matches forbidden pattern"),
                    ));
                }
            }
        }
    }
}

fn run_glob(spec_id: &str, check: &Check, files: &[&ScopedFile], findings: &mut Vec<Finding>) {
    let Some(pattern) = &check.pattern else {
        findings.push(config_error(
            spec_id,
            &check.name,
            "glob check requires `pattern`".to_string(),
        ));
        return;
    };
    let matcher = match Glob::new(pattern) {
        Ok(glob) => glob.compile_matcher(),
        Err(err) => {
            findings.push(config_error(
                spec_id,
                &check.name,
                format!("invalid glob: {err}"),
            ));
            return;
        }
    };

    let matched: Vec<&&ScopedFile> = files
        .iter()
        .filter(|file| matcher.is_match(&file.rel))
        .collect();

    if check.negate {
        if matched.is_empty() {
            findings.push(violation(
                check,
                spec_id,
                None,
                message(check, &format!("required path not found: {pattern}")),
            ));
        }
    } else {
        for file in matched {
            findings.push(violation(
                check,
                spec_id,
                Some(rel_str(file)),
                message(check, "forbidden path"),
            ));
        }
    }
}

fn run_script(
    spec_id: &str,
    check: &Check,
    repo_path: &Path,
    script_base: &Path,
    files: &[&ScopedFile],
    findings: &mut Vec<Finding>,
) {
    let Some(reference) = &check.script else {
        findings.push(config_error(
            spec_id,
            &check.name,
            "script check requires `script`".to_string(),
        ));
        return;
    };

    let Some(script) = resolve_script(reference, script_base, spec_id) else {
        let dir = script_base.join(tech_spec::SCRIPTS_DIR).join(spec_id);
        findings.push(config_error(
            spec_id,
            &check.name,
            format!("script `{reference}` not found in {}", dir.display()),
        ));
        return;
    };

    match check.run {
        RunScope::PerFile => run_per_file(spec_id, check, &script, repo_path, files, findings),
        RunScope::PerRepo => run_per_repo(spec_id, check, &script, repo_path, findings),
    }
}

fn run_per_file(
    spec_id: &str,
    check: &Check,
    script: &Path,
    repo_path: &Path,
    files: &[&ScopedFile],
    findings: &mut Vec<Finding>,
) {
    for batch in files.chunks(SCRIPT_BATCH_SIZE) {
        let args = batch.iter().map(|file| file.rel.as_os_str());

        let output = match Command::new(script)
            .args(args)
            .current_dir(repo_path)
            .output()
        {
            Ok(output) => output,
            Err(err) => {
                findings.push(run_error(spec_id, check, script, err));
                return;
            }
        };

        if !output.status.success() {
            findings.push(violation(check, spec_id, None, script_message(&output)));
        }
    }
}

fn run_per_repo(
    spec_id: &str,
    check: &Check,
    script: &Path,
    repo_path: &Path,
    findings: &mut Vec<Finding>,
) {
    let output = match Command::new(script).current_dir(repo_path).output() {
        Ok(output) => output,
        Err(err) => {
            findings.push(run_error(spec_id, check, script, err));
            return;
        }
    };

    if !output.status.success() {
        findings.push(violation(check, spec_id, None, script_message(&output)));
    }
}

// Scripts are stored at `<base>/scripts/<spec-id>/<reference>`, so the `script`
// field only carries the bare filename.
fn resolve_script(reference: &str, base: &Path, spec_id: &str) -> Option<PathBuf> {
    let script = base
        .join(tech_spec::SCRIPTS_DIR)
        .join(spec_id)
        .join(reference);
    script.is_file().then_some(script)
}

fn run_error(spec_id: &str, check: &Check, script: &Path, err: std::io::Error) -> Finding {
    config_error(
        spec_id,
        &check.name,
        format!("could not run script `{}`: {err}", script.display()),
    )
}

fn script_message(output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut parts = Vec::new();
    if !stdout.trim().is_empty() {
        parts.push(stdout.trim().to_string());
    }
    if !stderr.trim().is_empty() {
        parts.push(stderr.trim().to_string());
    }
    if parts.is_empty() {
        let code = output
            .status
            .code()
            .map_or_else(|| "signal".to_string(), |code| code.to_string());
        return format!("script exited with status {code}");
    }
    parts.join("\n")
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

fn report(config: &Config, reports: &[RepoReport]) -> ExitCode {
    let mut total_errors = 0usize;
    let mut total_warnings = 0usize;

    for repo in reports {
        let mut errors = 0usize;
        let mut warnings = 0usize;

        println!("{}:", repo.name);
        for finding in &repo.findings {
            match finding.severity {
                Severity::Error => errors += 1,
                Severity::Warn => warnings += 1,
            }
            println!("  {}", format_finding(finding));
        }
        println!("  {errors} error(s), {warnings} warning(s)");

        total_errors += errors;
        total_warnings += warnings;
    }

    println!(
        "{}: {total_errors} error(s), {total_warnings} warning(s)",
        config.project_name
    );

    if total_errors > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn format_finding(finding: &Finding) -> String {
    let severity = match finding.severity {
        Severity::Error => "error",
        Severity::Warn => "warn",
    };
    let check = if finding.check.is_empty() {
        finding.spec_id.clone()
    } else {
        format!("{}/{}", finding.spec_id, finding.check)
    };
    let location = finding
        .location
        .as_deref()
        .map(|location| format!("{location}: "))
        .unwrap_or_default();
    format!("{severity}: {check}: {location}{}", finding.message)
}
