use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anstream::println;
use eros::Context;
use owo_colors::OwoColorize;

use crate::config::{self, PreexistingError, Repository, spex_yaml_edit};
use crate::error::{Result, could_not_read};
use crate::rule::Severity;
use crate::support::fs::write_file;
use crate::support::text::plural;

use super::finding::Finding;
use super::report::{CheckReport, ReportTarget};

#[cfg(test)]
#[path = "baseline_test.rs"]
mod baseline_test;

enum Baseline {
    WholeCheck,
    Files(BTreeSet<String>),
}

impl Baseline {
    fn add_file(&mut self, file: Option<PathBuf>) {
        match (self, file) {
            (Baseline::WholeCheck, _) => {}
            (slot @ Baseline::Files(_), None) => *slot = Baseline::WholeCheck,
            (Baseline::Files(files), Some(file)) => {
                files.insert(file.to_string_lossy().into_owned());
            }
        }
    }
}

struct Owner<'a> {
    repo: &'a Repository,
    file: Option<PathBuf>,
}

fn owner<'a>(
    target: &ReportTarget<'a>,
    finding: &Finding,
    selected: &[&'a Repository],
) -> Option<Owner<'a>> {
    let file = finding
        .location
        .as_ref()
        .map(|location| location.file.as_path());
    match target {
        ReportTarget::Repository(repo) => Some(Owner {
            repo,
            file: file.map(Path::to_path_buf),
        }),
        ReportTarget::Workspace { spex_dir } => {
            let located = spex_dir.join(file?);
            selected.iter().find_map(|repo| {
                let relative = located.strip_prefix(&repo.path).ok()?;
                (!relative.starts_with("..")).then(|| Owner {
                    repo,
                    file: Some(relative.to_path_buf()),
                })
            })
        }
    }
}

fn is_baselined(finding: &Finding, owner: &Owner) -> bool {
    finding.severity == Severity::Error
        && finding
            .source
            .baseline_key()
            .is_some_and(|(rule_id, check)| {
                owner.repo.ignored_errors.iter().any(|entry| {
                    entry.rule_id == rule_id
                        && entry.check == check
                        && (entry.files.is_empty()
                            || owner.file.as_ref().is_some_and(|file| {
                                entry.files.iter().any(|listed| Path::new(listed) == file)
                            }))
                })
            })
}

pub(super) fn drop_baselined_findings<'a>(
    reports: &mut [CheckReport<'a>],
    selected: &[&'a Repository],
) {
    for report in reports {
        let target = &report.target;
        report.findings.retain(|finding| {
            !owner(target, finding, selected).is_some_and(|owner| is_baselined(finding, &owner))
        });
    }
}

pub(super) fn fill_preexisting_errors(
    spex_dir: &Path,
    selected: &[&Repository],
    reports: &[CheckReport],
) -> Result<ExitCode> {
    let mut baselines_by_repo: HashMap<String, BTreeMap<(String, String), Baseline>> = selected
        .iter()
        .map(|repo| (repo.repository_name.clone(), BTreeMap::new()))
        .collect();

    for report in reports {
        for finding in &report.findings {
            if finding.severity != Severity::Error {
                continue;
            }
            let Some((rule_id, check)) = finding.source.baseline_key() else {
                continue;
            };
            let Some(Owner { repo, file }) = owner(&report.target, finding, selected) else {
                continue;
            };
            let Some(checks) = baselines_by_repo.get_mut(&repo.repository_name) else {
                continue;
            };
            checks
                .entry((rule_id.to_string(), check.to_string()))
                .or_insert_with(|| Baseline::Files(BTreeSet::new()))
                .add_file(file);
        }
    }

    let by_repo: HashMap<String, Vec<PreexistingError>> = baselines_by_repo
        .into_iter()
        .map(|(name, checks)| {
            let entries = checks
                .into_iter()
                .map(|((rule_id, check), baseline)| PreexistingError {
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
    let raw =
        std::fs::read_to_string(&config_path).with_context(|| could_not_read(&config_path))?;
    let updated = spex_yaml_edit::set_preexisting_errors(&raw, &by_repo);
    let changed = updated != raw;
    if changed {
        write_file(&config_path, &updated)?;
    }

    report_fill(selected, &by_repo, changed);
    Ok(ExitCode::SUCCESS)
}

fn report_fill(
    selected: &[&Repository],
    by_repo: &HashMap<String, Vec<PreexistingError>>,
    changed: bool,
) {
    println!("{}", "spex check --fill-preexisting-errors".bold());
    println!();
    for repo in selected {
        let count = by_repo.get(&repo.repository_name).map_or(0, Vec::len);
        let summary = if count == 0 {
            "no pre-existing errors".green().to_string()
        } else {
            format!("baselined {}", plural(count, "check"))
                .dimmed()
                .to_string()
        };
        println!("  {}  {summary}", repo.repository_name.bold());
    }
    println!();
    if changed {
        println!("Updated {}", config::SPEX_CONFIG_FILE);
    } else {
        println!("{} unchanged", config::SPEX_CONFIG_FILE);
    }
}
