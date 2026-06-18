use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use error_stack::{Report, ResultExt};
use ignore::WalkBuilder;
use regex::Regex;
use thiserror::Error;

use crate::biz_spec::{self, BizSpec, Status};
use crate::config::{self, Config};
use crate::style::Style;

#[cfg(test)]
#[path = "coverage_test.rs"]
mod coverage_test;

type Result<T> = core::result::Result<T, Report<CoverageError>>;

#[derive(Debug, Error)]
pub enum CoverageError {
    #[error("could not determine the current directory")]
    CurrentDir,
    #[error("could not load spex configuration")]
    Config,
    #[error("could not load business specs")]
    Specs,
    #[error("could not walk workspace repository files")]
    Walk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Layer {
    Contract,
    Impl,
    Test,
}

// Spec-id annotations found across the workspace, keyed by spec id.
type Found = HashMap<String, HashSet<Layer>>;

struct SpecRow {
    id: String,
    has_contract: bool,
    has_impl: bool,
    has_test: bool,
}

impl SpecRow {
    // A contract is optional, so a spec only has a real gap when an impl or a
    // test annotation is missing.
    fn has_gap(&self) -> bool {
        !self.has_impl || !self.has_test
    }

    fn fully_covered(&self) -> bool {
        self.has_contract && self.has_impl && self.has_test
    }
}

pub fn run() -> Result<ExitCode> {
    let current_dir = std::env::current_dir().change_context(CoverageError::CurrentDir)?;
    let spex_dir = config::find_dir(&current_dir).change_context(CoverageError::Config)?;

    let config = config::load(&spex_dir).change_context(CoverageError::Config)?;
    let specs = biz_spec::load_all(&spex_dir.join(biz_spec::BIZ_SPECS_DIR))
        .change_context(CoverageError::Specs)?;

    let found = scan_workspace(&spex_dir, &config)?;

    Ok(report(&config, &specs, &found))
}

// Scans every workspace repository except spex itself for spec-id annotations.
fn scan_workspace(spex_dir: &Path, config: &Config) -> Result<Found> {
    let scanner = Scanner::new();
    let spex_canon = canonical(spex_dir);
    let mut found = Found::new();

    for repo in &config.repositories {
        let repo_path = spex_dir.join(&repo.path);
        if !repo_path.is_dir() || canonical(&repo_path) == spex_canon {
            continue;
        }
        scan_repo(&scanner, &repo_path, &mut found)?;
    }

    Ok(found)
}

fn scan_repo(scanner: &Scanner, repo_path: &Path, found: &mut Found) -> Result<()> {
    let walker = WalkBuilder::new(repo_path)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .filter_entry(|entry| entry.file_name() != std::ffi::OsStr::new(".git"))
        .build();

    for result in walker {
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
        let id = r"(?:BIZ|TECH)-[A-Z]+(?:-[A-Z]+)*";
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
                "contract" => Layer::Contract,
                "impl" => Layer::Impl,
                "test" => Layer::Test,
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
                .insert(Layer::Contract);
        }
    }
}

fn canonical(path: &Path) -> std::path::PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn active_rows(specs: &[BizSpec], found: &Found) -> Vec<SpecRow> {
    specs
        .iter()
        .filter(|spec| spec.status == Status::Active)
        .map(|spec| {
            let layers = found.get(&spec.id);
            let has = |layer: Layer| layers.is_some_and(|set| set.contains(&layer));
            SpecRow {
                id: spec.id.clone(),
                has_contract: has(Layer::Contract),
                has_impl: has(Layer::Impl),
                has_test: has(Layer::Test),
            }
        })
        .collect()
}

fn missing_summary(row: &SpecRow) -> String {
    let mut parts = Vec::new();
    if !row.has_impl {
        parts.push("impl".to_string());
    }
    if !row.has_test {
        parts.push("test".to_string());
    }
    if !row.has_contract {
        parts.push("contract (optional)".to_string());
    }
    parts.join(", ")
}

fn report(config: &Config, specs: &[BizSpec], found: &Found) -> ExitCode {
    let style = Style::detect();

    println!(
        "{} {}",
        style.bold("spex coverage"),
        style.dim(&format!("· {}", config.project_name))
    );
    println!();

    print_status_totals(&style, specs);

    let rows = active_rows(specs, found);
    print_coverage(&style, &rows);

    let gaps = rows.iter().filter(|row| row.has_gap()).count();
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

fn print_status_totals(style: &Style, specs: &[BizSpec]) {
    println!(
        "{}",
        style.bold(&format!("Business specs ({})", specs.len()))
    );
    if specs.is_empty() {
        println!("  {}", style.dim("none found"));
        println!();
        return;
    }

    let width = Status::ALL
        .iter()
        .map(|status| status.label().len())
        .max()
        .unwrap_or(0);
    for status in Status::ALL {
        let count = specs.iter().filter(|spec| spec.status == status).count();
        if count == 0 {
            continue;
        }
        println!("  {:<width$}  {count}", status.label(), width = width);
    }
    println!();
}

fn print_coverage(style: &Style, rows: &[SpecRow]) {
    println!(
        "{}",
        style.bold(&format!("Active spec coverage ({})", rows.len()))
    );
    if rows.is_empty() {
        println!("  {}", style.dim("no active specs"));
        println!();
        return;
    }

    let width = rows.iter().map(|row| row.id.len()).max().unwrap_or(0);
    let mut covered = 0usize;
    for row in rows {
        let (symbol, fully) = if row.has_gap() {
            (style.red("✗"), false)
        } else if row.fully_covered() {
            covered += 1;
            (style.green("✓"), true)
        } else {
            // Required layers present, only the optional contract is missing.
            (style.yellow("⚠"), false)
        };

        let id = style.cyan(&format!("{:<width$}", row.id, width = width));
        if fully {
            println!("  {symbol}  {id}");
        } else {
            println!(
                "  {symbol}  {id}  {}",
                style.dim(&format!("missing: {}", missing_summary(row)))
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
}

fn plural(count: usize, noun: &str) -> String {
    if count == 1 {
        format!("1 {noun}")
    } else {
        format!("{count} {noun}s")
    }
}
