use std::path::Path;
use std::process::ExitCode;

use clap::Args;
use error_stack::{Report, ResultExt};
use thiserror::Error;

use crate::config::Repository;
use crate::rule::RULES_DIR;
use crate::util;
use crate::workspace::Workspace;

#[cfg(test)]
#[path = "rules_test.rs"]
mod rules_test;

type Result<T> = core::result::Result<T, Report<RulesError>>;

#[derive(Debug, Error)]
pub enum RulesError {
    #[error("could not load the spex workspace")]
    Workspace,
    #[error("could not read `{0}`")]
    ReadFile(String),
    #[error("could not parse `{0}`")]
    Parse(String),
    #[error("could not serialize output")]
    Serialize,
    #[error("unknown workspace repository `{0}` — not listed in spex.yaml")]
    UnknownRepo(String),
    #[error("could not write `{0}`")]
    WriteFile(String),
}

#[derive(Args)]
pub struct RulesArgs {
    #[arg(
        long,
        help = "Drop fields an agent doesn't need to follow a rule: sync_with_upstream and checks"
    )]
    ai: bool,
    #[arg(long, num_args = 1.., help = "Rule IDs to include (default: all)")]
    id: Vec<String>,
    #[arg(long, num_args = 1.., help = "Workspace repository names to include (default: all)")]
    repo: Vec<String>,
    #[arg(
        long,
        help = "Write the rendered YAML to this file instead of stdout — the full rules list is large and can get truncated when printed to a terminal or piped through a shell"
    )]
    output: Option<String>,
}

pub fn run(args: RulesArgs) -> Result<ExitCode> {
    let workspace = Workspace::discover().change_context(RulesError::Workspace)?;
    let yaml = render(&workspace, &args)?;
    match &args.output {
        Some(path) => {
            std::fs::write(path, &yaml)
                .change_context_lazy(|| RulesError::WriteFile(path.clone()))?;
        }
        None => print!("{yaml}"),
    }
    Ok(ExitCode::SUCCESS)
}

fn render(workspace: &Workspace, args: &RulesArgs) -> Result<String> {
    for name in &args.repo {
        if !workspace
            .config
            .repositories
            .iter()
            .any(|repo| repo.matches(name))
        {
            return Err(Report::new(RulesError::UnknownRepo(name.clone())));
        }
    }

    let mut rules = load_docs(&workspace.dir.join(RULES_DIR))?;
    if args.ai {
        for (rule, path) in &mut rules {
            if let Some(obj) = rule.as_object_mut() {
                let mut with_path = serde_json::Map::with_capacity(obj.len() + 1);
                for (key, value) in std::mem::take(obj) {
                    if key == "sync_with_upstream" || key == "checks" {
                        continue;
                    }
                    let is_id = key == "id";
                    with_path.insert(key, value);
                    if is_id {
                        with_path.insert(
                            "path".to_string(),
                            serde_json::Value::String(format!("{RULES_DIR}/{path}")),
                        );
                    }
                }
                *obj = with_path;
            }
        }
    }
    let rules: Vec<serde_json::Value> = rules.into_iter().map(|(rule, _)| rule).collect();

    let repositories: Vec<&Repository> = workspace
        .config
        .repositories
        .iter()
        .filter(|repo| args.repo.is_empty() || args.repo.iter().any(|name| repo.matches(name)))
        .collect();

    let output: Vec<&serde_json::Value> = rules
        .iter()
        .filter(|rule| repositories.iter().any(|repo| repo_has_rule(repo, rule)))
        .filter(|rule| id_selected(rule, &args.id))
        .collect();

    if output.is_empty() {
        return serde_yaml_ng::to_string(&output).change_context(RulesError::Serialize);
    }

    output
        .iter()
        .map(|rule| serde_yaml_ng::to_string(&[rule]).change_context(RulesError::Serialize))
        .collect::<Result<Vec<_>>>()
        .map(|docs| docs.join("\n"))
}

fn repo_has_rule(repo: &Repository, rule: &serde_json::Value) -> bool {
    rule.get("id")
        .and_then(|v| v.as_str())
        .is_some_and(|rule_id| repo.rules.iter().any(|id| id == rule_id))
}

fn id_selected(value: &serde_json::Value, ids: &[String]) -> bool {
    ids.is_empty()
        || value
            .get("id")
            .and_then(|v| v.as_str())
            .is_some_and(|value_id| ids.iter().any(|id| id == value_id))
}

// Loads every rule under `dir` as generic JSON (rules may nest in
// subdirectories, but the Rule ID is always just the filename), so the full
// document — every field the schema allows, not just the subset `Rule`
// deserializes for running checks — is available to filter and print.
fn load_docs(dir: &Path) -> Result<Vec<(serde_json::Value, String)>> {
    let mut docs = Vec::new();
    for path in util::yaml_files(dir) {
        let rel_name = path
            .strip_prefix(dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .into_owned();
        let raw = std::fs::read_to_string(&path)
            .change_context_lazy(|| RulesError::ReadFile(rel_name.clone()))?;
        let yaml: serde_yaml_ng::Value = serde_yaml_ng::from_str(&raw)
            .change_context_lazy(|| RulesError::Parse(rel_name.clone()))?;
        let json = serde_json::to_value(yaml)
            .change_context_lazy(|| RulesError::Parse(rel_name.clone()))?;
        docs.push((json, rel_name));
    }
    Ok(docs)
}
