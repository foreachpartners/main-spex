use std::path::Path;
use std::process::ExitCode;

use clap::Args;
use error_stack::{Report, ResultExt};

use crate::config::Repository;
use crate::config::spex::Spex;
use crate::error::{CliError, Result};
use crate::rule::RULES_DIR;
use crate::support::fs::write_file;
use crate::support::util;

#[cfg(test)]
#[path = "rules_test.rs"]
mod rules_test;

#[derive(Args)]
pub struct RulesArgs {
    #[arg(long, help = "Drop fields AI doesn't need, add ones it does")]
    ai: bool,
    #[arg(long, num_args = 1.., help = "Rule IDs to include (default: all)")]
    id: Vec<String>,
    #[arg(long, num_args = 1.., help = "Workspace repository names to include (default: all)")]
    repo: Vec<String>,
    #[arg(
        long,
        help = "Write the output to this file instead of stdout — the full rules list is large and can get truncated when printed to a terminal or piped through a shell"
    )]
    output: Option<String>,
}

pub fn run(args: RulesArgs) -> Result<ExitCode> {
    let spex = Spex::discover()?;
    let yaml = render(&spex, &args)?;
    match &args.output {
        Some(path) => {
            write_file(Path::new(path), &yaml)?;
        }
        None => print!("{yaml}"),
    }
    Ok(ExitCode::SUCCESS)
}

fn render(spex: &Spex, args: &RulesArgs) -> Result<String> {
    for name in &args.repo {
        if !spex
            .config
            .repositories
            .iter()
            .any(|repo| repo.matches_any_repo_name(name))
        {
            return Err(Report::new(CliError::UnknownRepo(name.clone())));
        }
    }

    let mut rules = util::load_yaml_dir::<serde_json::Value>(&spex.dir.join(RULES_DIR))?;
    if args.ai {
        for file in &mut rules {
            if let Some(obj) = file.item.as_object_mut() {
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
                            serde_json::Value::String(format!("{RULES_DIR}/{}", file.rel_path)),
                        );
                    }
                }
                *obj = with_path;
            }
        }
    }
    let rules: Vec<serde_json::Value> = rules.into_iter().map(|file| file.item).collect();

    for id in &args.id {
        if !rules
            .iter()
            .any(|rule| rule.get("id").and_then(|v| v.as_str()) == Some(id.as_str()))
        {
            return Err(Report::new(CliError::UnknownRuleId(id.clone())));
        }
    }

    let repositories: Vec<&Repository> = spex
        .config
        .repositories
        .iter()
        .filter(|repo| {
            args.repo.is_empty()
                || args
                    .repo
                    .iter()
                    .any(|name| repo.matches_any_repo_name(name))
        })
        .collect();

    let output: Vec<&serde_json::Value> = rules
        .iter()
        .filter(|rule| repositories.iter().any(|repo| repo_has_rule(repo, rule)))
        .filter(|rule| id_selected(rule, &args.id))
        .collect();

    if output.is_empty() {
        return serde_yaml_ng::to_string(&output).change_context(CliError::Serialize);
    }

    output
        .iter()
        .map(|rule| serde_yaml_ng::to_string(&[rule]).change_context(CliError::Serialize))
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
