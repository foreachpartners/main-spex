use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use error_stack::{Report, ResultExt};
use regex::Regex;
use thiserror::Error;

use crate::config::Config;
use crate::spec::{CoverageLayer, Spec, Status};
use crate::style::Style;
use crate::util::{self, plural};
use crate::workspace::Workspace;

#[cfg(test)]
#[path = "coverage_test.rs"]
mod coverage_test;

type Result<T> = core::result::Result<T, Report<CoverageError>>;

#[derive(Debug, Error)]
pub enum CoverageError {
    #[error("could not load the spex workspace")]
    Workspace,
    #[error("could not walk workspace repository files")]
    Walk,
}

// Spec-id annotations found across the workspace, keyed by spec id.
type Found = HashMap<String, HashSet<CoverageLayer>>;

struct SpecRow {
    id: String,
    present: HashSet<CoverageLayer>,
    ignored: HashSet<CoverageLayer>,
    ac_done: usize,
    ac_total: usize,
}

impl SpecRow {
    fn is_present(&self, layer: CoverageLayer) -> bool {
        self.present.contains(&layer)
    }

    fn is_required(&self, layer: CoverageLayer) -> bool {
        !self.ignored.contains(&layer)
    }

    fn ac_complete(&self) -> bool {
        self.ac_done == self.ac_total
    }

    // A gap is a required layer with no annotation, or an unimplemented
    // acceptance criterion. Layers listed in `coverage_ignore` are treated as
    // ok even when missing; the acceptance-criteria checkboxes are not relaxed.
    fn has_gap(&self) -> bool {
        !self.ac_complete()
            || CoverageLayer::ALL
                .iter()
                .any(|&layer| self.is_required(layer) && !self.is_present(layer))
    }

    fn fully_covered(&self) -> bool {
        self.ac_complete()
            && CoverageLayer::ALL
                .iter()
                .all(|&layer| self.is_present(layer))
    }
}

pub fn run() -> Result<ExitCode> {
    let workspace = Workspace::discover().change_context(CoverageError::Workspace)?;
    let specs = workspace.specs().change_context(CoverageError::Workspace)?;

    let found = scan_workspace(&workspace.dir, &workspace.config)?;

    Ok(report(&workspace.config, &specs, &found))
}

// Scans every workspace repository except spex itself for spec-id annotations.
fn scan_workspace(spex_dir: &Path, config: &Config) -> Result<Found> {
    let scanner = Scanner::new();
    let spex_canon = canonical(spex_dir);
    let mut found = Found::new();

    for repo in &config.repositories {
        // Interface contracts live inside the spex repository's `contracts/`
        // folder, so scan them explicitly even though the spex dir itself is
        // skipped below; their `x-spec` annotations are the contract layer.
        for contract in &repo.contracts {
            let contract_path = spex_dir.join(&contract.path);
            if let Ok(content) = fs::read_to_string(&contract_path) {
                scanner.scan(&content, &mut found);
            }
        }

        let repo_path = spex_dir.join(&repo.path);
        if !repo_path.is_dir() || canonical(&repo_path) == spex_canon {
            continue;
        }
        scan_repo(&scanner, &repo_path, &mut found)?;
    }

    Ok(found)
}

fn scan_repo(scanner: &Scanner, repo_path: &Path, found: &mut Found) -> Result<()> {
    for result in util::repo_walker(repo_path) {
        let entry = result.change_context(CoverageError::Walk)?;
        if !entry.file_type().is_some_and(|kind| kind.is_file()) {
            continue;
        }
        // Skip files that aren't valid UTF-8 (binaries); they carry no annotations.
        let Ok(content) = fs::read_to_string(entry.path()) else {
            continue;
        };
        scanner.scan(&content, found);
    }

    Ok(())
}

struct Scanner {
    spec: Regex,
    xspec: Regex,
    id: Regex,
}

impl Scanner {
    fn new() -> Self {
        let id = r"[a-z]+(?:-[a-z]+)*";
        Self {
            spec: Regex::new(&format!(
                r"@spec\s+(contract|impl|test)\s+({id}(?:\s*,\s*{id})*)"
            ))
            .expect("valid @spec regex"),
            xspec: Regex::new(&format!(r#"x-spec:\s*"?({id})"?"#)).expect("valid x-spec regex"),
            id: Regex::new(id).expect("valid id regex"),
        }
    }

    fn scan(&self, content: &str, found: &mut Found) {
        for caps in self.spec.captures_iter(content) {
            let layer = match &caps[1] {
                "contract" => CoverageLayer::Contract,
                "impl" => CoverageLayer::Impl,
                "test" => CoverageLayer::Test,
                _ => continue,
            };
            for id in self.id.find_iter(&caps[2]) {
                found
                    .entry(id.as_str().to_string())
                    .or_default()
                    .insert(layer);
            }
        }

        for caps in self.xspec.captures_iter(content) {
            found
                .entry(caps[1].to_string())
                .or_default()
                .insert(CoverageLayer::Contract);
        }
    }
}

fn canonical(path: &Path) -> std::path::PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn tracked_rows(specs: &[Spec], found: &Found) -> Vec<SpecRow> {
    specs
        .iter()
        .filter(|spec| matches!(spec.status, Status::Active | Status::InProgress))
        .map(|spec| {
            let (ac_done, ac_total) = spec.ac_counts();
            SpecRow {
                id: spec.id.clone(),
                present: found.get(&spec.id).cloned().unwrap_or_default(),
                ignored: spec.coverage_ignore.iter().copied().collect(),
                ac_done,
                ac_total,
            }
        })
        .collect()
}

fn missing_summary(row: &SpecRow) -> String {
    CoverageLayer::ALL
        .iter()
        .filter(|&&layer| !row.is_present(layer))
        .map(|&layer| {
            if row.is_required(layer) {
                layer.label().to_string()
            } else {
                format!("{} (ignored)", layer.label())
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn report(config: &Config, specs: &[Spec], found: &Found) -> ExitCode {
    let style = Style::detect();

    println!(
        "{} {}",
        style.bold("spex coverage"),
        style.dim(&format!("· {}", config.project_name))
    );
    println!();

    print_status_totals(&style, specs);

    let rows = tracked_rows(specs, found);
    let gaps = print_coverage(&style, &rows);

    let footer = if gaps == 0 {
        style.green("no gaps")
    } else {
        style.red(&plural(gaps, "gap"))
    };
    println!("{}: {footer}", style.bold(&config.project_name));

    if gaps == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

// Spec ids per status, in Status::ALL order, skipping empty statuses.
fn status_groups(specs: &[Spec]) -> Vec<(Status, Vec<&str>)> {
    Status::ALL
        .into_iter()
        .filter_map(|status| {
            let ids: Vec<&str> = specs
                .iter()
                .filter(|spec| spec.status == status)
                .map(|spec| spec.id.as_str())
                .collect();
            (!ids.is_empty()).then_some((status, ids))
        })
        .collect()
}

fn print_status_totals(style: &Style, specs: &[Spec]) {
    println!("{}", style.bold(&format!("Specs ({})", specs.len())));
    if specs.is_empty() {
        println!("  {}", style.dim("none found"));
        println!();
        return;
    }

    for (status, ids) in status_groups(specs) {
        println!("  {} ({})", style.bold(status.label()), ids.len());
        // Active and in-progress specs are already listed below with their
        // coverage detail, so listing their ids here would just duplicate that.
        if !matches!(status, Status::Active | Status::InProgress) {
            for id in ids {
                println!("    {}", style.cyan(id));
            }
        }
    }
    println!();
}

fn print_coverage(style: &Style, rows: &[SpecRow]) -> usize {
    println!(
        "{}",
        style.bold(&format!("Tracked spec coverage ({})", rows.len()))
    );
    if rows.is_empty() {
        println!("  {}", style.dim("no active or in-progress specs"));
        println!();
        return 0;
    }

    let width = rows.iter().map(|row| row.id.len()).max().unwrap_or(0);
    let mut covered = 0usize;
    for row in rows {
        let symbol = if row.has_gap() {
            style.red("✗")
        } else if row.fully_covered() {
            covered += 1;
            style.green("✓")
        } else {
            // No gap: every required layer is present and all acceptance
            // criteria are implemented, but an ignored layer is still missing,
            // so coverage is not complete.
            style.yellow("⚠")
        };

        let id = style.cyan(&format!("{:<width$}", row.id, width = width));
        let ac_text = format!("{}/{} AC", row.ac_done, row.ac_total);
        let ac = if row.ac_complete() {
            style.dim(&ac_text)
        } else {
            style.red(&ac_text)
        };

        let layers = missing_summary(row);
        if layers.is_empty() {
            println!("  {symbol}  {id}  {ac}");
        } else {
            println!(
                "  {symbol}  {id}  {ac}  {}",
                style.dim(&format!("missing: {layers}"))
            );
        }
    }

    let gaps = rows.iter().filter(|row| row.has_gap()).count();
    println!();
    println!(
        "  {}",
        style.dim(&format!(
            "{} fully covered · {}",
            covered,
            plural(gaps, "gap")
        ))
    );
    println!();
    gaps
}
