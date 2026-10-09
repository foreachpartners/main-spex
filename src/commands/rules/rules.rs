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

const FIELDS_AI_DOES_NOT_NEED: [&str; 2] = ["sync_with_upstream", "checks"];

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
            .repositories
            .iter()
            .any(|repo| repo.matches_any_repo_name(name))
        {
            return Err(Report::new(CliError::UnknownRepo(name.clone())));
        }
    }

    let rules: Vec<serde_json::Value> =
        util::load_yaml_dir::<serde_json::Value>(&spex.dir.join(RULES_DIR))?
            .iter()
            .map(|file| {
                if args.ai {
                    util::ai_view(file, &FIELDS_AI_DOES_NOT_NEED)
                } else {
                    file.item.clone()
                }
            })
            .collect();

    for id in &args.id {
        if !rules
            .iter()
            .any(|rule| rule.get("id").and_then(|v| v.as_str()) == Some(id.as_str()))
        {
            return Err(Report::new(CliError::UnknownRuleId(id.clone())));
        }
    }

    let repositories: Vec<&Repository> = spex
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
        return serde_yaml_ng::to_string(&output)
            .change_context_lazy(|| CliError::Internal("could not serialize rules".into()));
    }

    output
        .iter()
        .map(|rule| {
            serde_yaml_ng::to_string(&[rule])
                .change_context_lazy(|| CliError::Internal("could not serialize rules".into()))
        })
        .collect::<Result<Vec<_>>>()
        .map(|docs| docs.join("\n"))
}

fn repo_has_rule(repo: &Repository, rule: &serde_json::Value) -> bool {
    rule.get("id")
        .and_then(|v| v.as_str())
        .is_some_and(|rule_id| repo.rules.iter().any(|id| id.as_str() == rule_id))
}

fn id_selected(value: &serde_json::Value, ids: &[String]) -> bool {
    ids.is_empty()
        || value
            .get("id")
            .and_then(|v| v.as_str())
            .is_some_and(|value_id| ids.iter().any(|id| id == value_id))
}
