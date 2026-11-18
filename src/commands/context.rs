use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io;
use std::path::Path;
use std::process::ExitCode;

use clap::Args;
use eros::{Context, bail};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::config::spex::Spex;
use crate::config::workspace::RevisionStatus;
use crate::config::{ContractConfig, Package};
use crate::error::{Result, could_not_read};
use crate::rule::Rule;
use crate::spec::{AcceptanceCriterion, CoverageIgnore, Spec, Status};
use crate::support::fs::relative_to;
use crate::support::output::print_json;

#[derive(Args)]
pub struct ContextArgs {
    #[arg(long, required = true, num_args = 1.., help = "Select specs by ID")]
    id: Vec<String>,
    #[arg(long, num_args = 1.., help = "Include additional packages by name")]
    package: Vec<String>,
    #[arg(
        long,
        help = "Include explicitly selected deprecated specs or blockers"
    )]
    include_deprecated: bool,
}

#[derive(Serialize)]
struct ContextOutput<'a> {
    schema_version: u32,
    context_digest: String,
    context: TaskContext<'a>,
}

#[derive(Serialize)]
struct TaskContext<'a> {
    project: &'a str,
    description: Option<String>,
    prd: String,
    spex_revision: Option<String>,
    spex_dirty: Option<bool>,
    specs: Vec<SpecContext<'a>>,
    packages: Vec<PackageContext<'a>>,
    rules: Vec<RuleContext<'a>>,
}

#[derive(Serialize)]
struct SpecContext<'a> {
    id: &'a str,
    path: &'a Path,
    status: &'static str,
    description: &'a str,
    requested: bool,
    required_by: BTreeSet<&'a str>,
    packages: BTreeSet<&'a str>,
    contract: &'static str,
    acceptance_criteria: Vec<CriterionContext<'a>>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    coverage_ignore: &'a [CoverageIgnore],
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    package_coverage_ignore: &'a BTreeMap<String, Vec<CoverageIgnore>>,
}

#[derive(Serialize)]
struct CriterionContext<'a> {
    id: &'a str,
    commitment: &'static str,
    text: &'a str,
    line: usize,
    is_done: bool,
}

#[derive(Serialize)]
struct PackageContext<'a> {
    name: &'a str,
    short_name: &'a str,
    path: String,
    description: Option<String>,
    revision: Option<String>,
    dirty: Option<bool>,
    requested: bool,
    required_by: BTreeSet<&'a str>,
    rules: BTreeSet<&'a str>,
    contracts: Vec<ContractContext>,
}

struct PackageSource {
    path: String,
    description: Option<String>,
    contracts: Vec<ContractConfig>,
}

#[derive(Serialize)]
struct ContractContext {
    path: String,
    description: Option<String>,
    sha256: String,
}

#[derive(Serialize)]
struct RuleContext<'a> {
    id: &'a str,
    path: &'a Path,
    packages: BTreeSet<&'a str>,
    acceptance_criteria: Vec<&'a str>,
}

pub fn run(args: ContextArgs, project: Option<&Path>) -> Result<ExitCode> {
    let (dir, config) = Spex::load_project_config(project)?;
    let description = config.description.clone();
    let prd = config.prd.clone();
    let package_sources = config
        .packages
        .iter()
        .map(|entry| PackageSource {
            path: entry.path.clone(),
            description: entry.description.clone(),
            contracts: entry
                .contracts
                .iter()
                .map(|contract| ContractConfig {
                    path: contract.path.clone(),
                    description: contract.description.clone(),
                })
                .collect(),
        })
        .collect();
    let spex = Spex::from_config(dir, config).with_package_configs()?;
    let specs = spex.specs()?;
    let rules = spex.rules()?;
    let specs = select_specs(&specs, &args, &spex.dir)?;
    let packages = select_packages(&spex, package_sources, &specs, &args.package)?;
    let rules = select_rules(&rules, &packages, &spex.dir)?;
    let revision = RevisionStatus::inspect(&spex.dir);

    let context = TaskContext {
        project: &spex.project_name,
        description,
        prd,
        spex_revision: revision.head,
        spex_dirty: revision.dirty,
        specs,
        packages,
        rules,
    };
    let contents = serde_json::to_vec(&context).context("could not serialize task context")?;
    print_json(&ContextOutput {
        schema_version: 1,
        context_digest: format!("{:x}", Sha256::digest(contents)),
        context,
    })?;
    Ok(ExitCode::SUCCESS)
}

fn select_specs<'a>(
    specs: &'a [Spec],
    args: &ContextArgs,
    spex_dir: &'a Path,
) -> Result<Vec<SpecContext<'a>>> {
    let mut by_id = BTreeMap::new();
    for spec in specs {
        if by_id.insert(spec.id.as_str(), spec).is_some() {
            bail!("ambiguous spec ID `{}`", spec.id);
        }
    }
    let mut pending = BTreeSet::new();
    for id in &args.id {
        let Some(spec) = by_id.get(id.as_str()) else {
            bail!("unknown spec `{id}`");
        };
        pending.insert(spec.id.as_str());
    }

    let requested = pending.clone();
    let mut selected = BTreeSet::new();
    let mut required_by = BTreeMap::<&str, BTreeSet<&str>>::new();
    while let Some(id) = pending.pop_first() {
        if !selected.insert(id) {
            continue;
        }
        let spec = by_id[id];
        if spec.status == Status::Deprecated && !args.include_deprecated {
            bail!("spec `{id}` is deprecated — pass --include-deprecated to include it");
        }
        for dependency in &spec.blocked_by {
            let dep = dependency.as_str();
            if !by_id.contains_key(dep) {
                bail!("spec `{id}` depends on unknown spec `{dep}`");
            }
            required_by.entry(dep).or_default().insert(id);
            pending.insert(dep);
        }
    }

    Ok(selected
        .into_iter()
        .map(|id| {
            let spec = by_id[id];
            SpecContext {
                id,
                path: relative_to(&spec.path, spex_dir),
                status: spec.status.label(),
                description: &spec.description,
                requested: requested.contains(id),
                required_by: required_by.remove(id).unwrap_or_default(),
                packages: spec.packages.iter().map(String::as_str).collect(),
                contract: spec.contract.into(),
                acceptance_criteria: spec
                    .acceptance_criteria
                    .iter()
                    .map(CriterionContext::from)
                    .collect(),
                coverage_ignore: &spec.coverage_ignore,
                package_coverage_ignore: &spec.package_coverage_ignore,
            }
        })
        .collect())
}

impl<'a> From<&'a AcceptanceCriterion> for CriterionContext<'a> {
    fn from(criterion: &'a AcceptanceCriterion) -> Self {
        Self {
            id: criterion.id.as_str(),
            commitment: criterion.statement.commitment().label(),
            text: criterion.statement.text(),
            line: criterion.line,
            is_done: criterion.is_done,
        }
    }
}

fn select_packages<'a>(
    spex: &'a Spex,
    package_sources: Vec<PackageSource>,
    specs: &[SpecContext<'a>],
    names: &[String],
) -> Result<Vec<PackageContext<'a>>> {
    spex.validate_package_names(names)?;
    let mut required_by = BTreeMap::<&str, BTreeSet<&str>>::new();
    for spec in specs {
        for package in &spec.packages {
            required_by.entry(package).or_default().insert(spec.id);
        }
    }

    let mut packages = Vec::new();
    for (package, source) in spex.packages.iter().zip(package_sources) {
        let reasons = required_by.remove(package.package_short_name.as_str());
        let requested = names
            .iter()
            .any(|name| package.matches_any_package_name(name));
        if requested || reasons.is_some() {
            packages.push(package_context(
                package,
                source,
                requested,
                reasons.unwrap_or_default(),
            )?);
        }
    }
    if let Some((name, specs)) = required_by.first_key_value() {
        bail!(
            "unknown package `{name}` required by {}",
            specs.iter().copied().collect::<Vec<_>>().join(", ")
        );
    }
    packages.sort_by_key(|package| package.name);
    Ok(packages)
}

fn package_context<'a>(
    package: &'a Package,
    source: PackageSource,
    requested: bool,
    required_by: BTreeSet<&'a str>,
) -> Result<PackageContext<'a>> {
    if !package.path.is_dir() {
        bail!(
            "package `{}` directory is unavailable: `{}`",
            package.package_name,
            package.path.display()
        );
    }
    let revision = RevisionStatus::inspect(&package.path);
    let mut contracts = package
        .contracts
        .iter()
        .zip(source.contracts)
        .map(|(file, contract)| {
            Ok(ContractContext {
                path: contract.path,
                description: contract.description,
                sha256: file_digest(file)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    contracts.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(PackageContext {
        name: &package.package_name,
        short_name: &package.package_short_name,
        path: source.path,
        description: source.description,
        revision: revision.head,
        dirty: revision.dirty,
        requested,
        required_by,
        rules: package.rules.iter().map(|rule| rule.as_str()).collect(),
        contracts,
    })
}

fn select_rules<'a>(
    rules: &'a [Rule],
    packages: &[PackageContext<'a>],
    spex_dir: &'a Path,
) -> Result<Vec<RuleContext<'a>>> {
    let mut required_by = BTreeMap::<&str, BTreeSet<&str>>::new();
    for package in packages {
        for rule in &package.rules {
            required_by.entry(rule).or_default().insert(package.name);
        }
    }
    let mut selected = BTreeMap::new();
    for rule in rules {
        let id = rule.id.as_str();
        if let Some(packages) = required_by.get(id) {
            let context = RuleContext {
                id,
                path: relative_to(&rule.path, spex_dir),
                packages: packages.clone(),
                acceptance_criteria: rule
                    .acceptance_criteria
                    .iter()
                    .map(|criterion| criterion.text.as_str())
                    .collect(),
            };
            if selected.insert(id, context).is_some() {
                bail!("ambiguous rule ID `{id}`");
            }
        }
    }
    if let Some(id) = required_by.keys().find(|id| !selected.contains_key(*id)) {
        bail!("unknown rule `{id}` declared by a selected package");
    }
    Ok(selected.into_values().collect())
}

fn file_digest(path: &Path) -> Result<String> {
    if !path.is_file() {
        bail!("declared contract is unavailable: `{}`", path.display());
    }
    let mut file = File::open(path).with_context(|| could_not_read(path))?;
    let mut digest = Sha256::new();
    io::copy(&mut file, &mut digest).with_context(|| could_not_read(path))?;
    Ok(format!("{:x}", digest.finalize()))
}
