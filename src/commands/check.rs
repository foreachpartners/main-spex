use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use clap::Args;
use error_stack::{Report, ResultExt};
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use regex::Regex;
use thiserror::Error;

use crate::arch_spec::{self, ArchSpec, Check, CheckKind, Severity};
use crate::config::{self, Config, Repository};

#[cfg(test)]
#[path = "check_test.rs"]
mod check_test;

const SCRIPTS_DIR: &str = "scripts";

type Result<T> = core::result::Result<T, Report<CheckError>>;

#[derive(Debug, Error)]
pub enum CheckError {
    #[error("could not determine the current directory")]
    CurrentDir,
    #[error("could not load spex configuration")]
    Config,
    #[error("could not load arch specs")]
    Specs,
    #[error("unknown repository `{0}` — not listed in spex.yml")]
    UnknownRepo(String),
    #[error("could not walk repository files")]
    Walk,
}

#[derive(Args)]
pub struct CheckArgs {
    #[arg(long, num_args = 1.., help = "Repository names to check (default: all)")]
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

pub fn run(args: CheckArgs) -> Result<ExitCode> {
    let current_dir = std::env::current_dir().change_context(CheckError::CurrentDir)?;
    let spex_dir = config::find_dir(&current_dir).change_context(CheckError::Config)?;

    let config = config::load(&spex_dir).change_context(CheckError::Config)?;
    let specs = arch_spec::load_all(&spex_dir.join(arch_spec::ARCH_SPECS_DIR))
        .change_context(CheckError::Specs)?;

    let selected = select_repos(&config, &args.repo)?;

    let mut findings = Vec::new();
    for repo in selected {
        check_repo(repo, &spex_dir, &specs, &args.files, &mut findings)?;
    }

    Ok(report(&config, &findings))
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
    specs: &std::collections::HashMap<String, ArchSpec>,
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
            message: format!("repository path not found: {}", repo_path.display()),
        });
        return Ok(());
    }

    let mut candidates = collect_files(&repo_path)?;
    if !files.is_empty() {
        candidates.retain(|file| matches_filter(&file.rel, files));
    }

    for spec_id in &repo.arch_specs {
        let Some(spec) = specs.get(spec_id) else {
            findings.push(Finding {
                severity: Severity::Error,
                spec_id: spec_id.clone(),
                check: String::new(),
                location: None,
                message: format!(
                    "arch spec `{spec_id}` not found in {}/",
                    arch_spec::ARCH_SPECS_DIR
                ),
            });
            continue;
        };

        for check in &spec.checks {
            run_check(&spec.id, check, &repo_path, spex_dir, &candidates, findings);
        }
    }

    Ok(())
}

fn collect_files(repo_path: &Path) -> Result<Vec<ScopedFile>> {
    let mut files = Vec::new();
    for result in WalkBuilder::new(repo_path).build() {
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

fn matches_filter(rel: &Path, filters: &[String]) -> bool {
    let rel_str = rel.to_string_lossy();
    let name = rel.file_name().map(|name| name.to_string_lossy());
    filters.iter().any(|filter| {
        let filter = filter.trim_start_matches("./");
        rel_str == filter
            || name.as_deref() == Some(filter)
            || rel_str.ends_with(&format!("/{filter}"))
    })
}

fn run_check(
    spec_id: &str,
    check: &Check,
    repo_path: &Path,
    spex_dir: &Path,
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
        CheckKind::Script => run_script(spec_id, check, repo_path, spex_dir, &scoped, findings),
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
    spex_dir: &Path,
    files: &[&ScopedFile],
    findings: &mut Vec<Finding>,
) {
    let Some(script) = resolve_script(&check.name, repo_path, spex_dir) else {
        findings.push(config_error(
            spec_id,
            &check.name,
            format!(
                "script `{}` not found in repository or spex `{SCRIPTS_DIR}/`",
                check.name
            ),
        ));
        return;
    };

    let args: Vec<String> = files
        .iter()
        .map(|file| file.rel.to_string_lossy().into_owned())
        .collect();

    let output = match Command::new(&script)
        .args(&args)
        .current_dir(repo_path)
        .output()
    {
        Ok(output) => output,
        Err(err) => {
            findings.push(config_error(
                spec_id,
                &check.name,
                format!("could not run script `{}`: {err}", script.display()),
            ));
            return;
        }
    };

    if output.status.success() {
        return;
    }

    findings.push(violation(check, spec_id, None, script_message(&output)));
}

fn resolve_script(name: &str, repo_path: &Path, spex_dir: &Path) -> Option<PathBuf> {
    let repo_script = repo_path.join(SCRIPTS_DIR).join(name);
    if repo_script.is_file() {
        return Some(repo_script);
    }
    let spex_script = spex_dir.join(SCRIPTS_DIR).join(name);
    if spex_script.is_file() {
        return Some(spex_script);
    }
    None
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

fn report(config: &Config, findings: &[Finding]) -> ExitCode {
    let mut errors = 0usize;
    let mut warnings = 0usize;

    for finding in findings {
        match finding.severity {
            Severity::Error => errors += 1,
            Severity::Warn => warnings += 1,
        }
        println!("{}", format_finding(finding));
    }

    println!("{}: {errors} error(s), {warnings} warning(s)", config.name);

    if errors > 0 {
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
