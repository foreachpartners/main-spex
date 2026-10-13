use std::path::Path;
use std::process::ExitCode;

use clap::Args;
use eros::{Context, IntoAnyUnion, bail};

use crate::config::Package;
use crate::config::spex::Spex;
use crate::error::Result;
use crate::rule::RULES_DIR;
use crate::support::ai_view::ai_view;
use crate::support::fs::write_file;
use crate::support::yaml_dir::load_yaml_dir;

#[cfg(test)]
#[path = "rules_test.rs"]
mod rules_test;

const FIELDS_AI_DOES_NOT_NEED: [&str; 2] = ["sync_with_upstream", "checks"];

#[derive(Args)]
pub struct RulesArgs {
    #[arg(long, help = "Drop fields AI doesn't need, add ones it does")]
    ai: bool,
    #[arg(long, num_args = 1.., help = "Filter by rule ids")]
    id: Vec<String>,
    #[arg(long, num_args = 1.., help = "Filter by package names")]
    package: Vec<String>,
    #[arg(long, help = "Write the output to this file")]
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
    let packages = spex.select_packages(&args.package)?;

    let rules: Vec<serde_json::Value> =
        load_yaml_dir::<serde_json::Value>(&spex.dir.join(RULES_DIR))?
            .iter()
            .map(|file| {
                if args.ai {
                    ai_view(file, &FIELDS_AI_DOES_NOT_NEED)
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
            bail!("unknown rule id `{id}` — not found in {RULES_DIR}");
        }
    }

    let output: Vec<&serde_json::Value> = rules
        .iter()
        .filter(|rule| {
            packages
                .iter()
                .any(|package| package_has_rule(package, rule))
        })
        .filter(|rule| id_selected(rule, &args.id))
        .collect();

    if output.is_empty() {
        return serde_yaml_ng::to_string(&output)
            .context("could not serialize rules")
            .any_union();
    }

    output
        .iter()
        .map(|rule| {
            serde_yaml_ng::to_string(&[rule])
                .context("could not serialize rules")
                .any_union()
        })
        .collect::<Result<Vec<_>>>()
        .map(|docs| docs.join("\n"))
}

fn package_has_rule(package: &Package, rule: &serde_json::Value) -> bool {
    rule.get("id")
        .and_then(|v| v.as_str())
        .is_some_and(|rule_id| package.rules.iter().any(|id| id.as_str() == rule_id))
}

fn id_selected(value: &serde_json::Value, ids: &[String]) -> bool {
    ids.is_empty()
        || value
            .get("id")
            .and_then(|v| v.as_str())
            .is_some_and(|value_id| ids.iter().any(|id| id == value_id))
}
