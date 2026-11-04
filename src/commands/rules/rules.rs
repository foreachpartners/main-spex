use std::process::ExitCode;

use clap::Args;
use eros::bail;

use crate::config::Package;
use crate::config::spex::Spex;
use crate::error::Result;
use crate::rule::RULES_DIR;
use crate::support::entry_view::{
    Keywords, ensure_schema_fields, entries_to_yaml, keep_fields, print_entries, with_path,
};
use crate::support::yaml_dir::load_yaml_dir;
use crate::templates::RULE_SCHEMA_FILENAME;

#[cfg(test)]
#[path = "rules_test.rs"]
mod rules_test;

#[derive(Args)]
pub struct RulesArgs {
    #[arg(long, num_args = 1.., help = "Filter by rule ids")]
    id: Vec<String>,
    #[arg(long, num_args = 1.., help = "Filter by package names")]
    package: Vec<String>,
    #[arg(long, num_args = 1.., help = "Print only rules containing any of these keywords")]
    search: Vec<String>,
    #[arg(long, num_args = 1.., help = "Print only these fields of each rule")]
    fields: Vec<String>,
    #[arg(long, help = "Write the output to this file")]
    output: Option<String>,
}

pub fn run(args: RulesArgs) -> Result<ExitCode> {
    let spex = Spex::discover()?;
    let yaml = render(&spex, &args)?;
    print_entries(&yaml, args.output.as_deref())?;
    Ok(ExitCode::SUCCESS)
}

fn render(spex: &Spex, args: &RulesArgs) -> Result<String> {
    let packages = spex.select_packages(&args.package)?;
    ensure_schema_fields(&args.fields, "rule", RULE_SCHEMA_FILENAME)?;
    let keywords = Keywords::new(&args.search);

    let files = load_yaml_dir::<serde_json::Value>(&spex.dir.join(RULES_DIR))?;

    for id in &args.id {
        if !files
            .iter()
            .any(|file| file.item.get("id").and_then(|v| v.as_str()) == Some(id.as_str()))
        {
            bail!("unknown rule id `{id}` — not found in {RULES_DIR}");
        }
    }

    let output: Vec<serde_json::Value> = files
        .iter()
        .filter(|file| {
            packages
                .iter()
                .any(|package| package_has_rule(package, &file.item))
        })
        .filter(|file| id_selected(&file.item, &args.id))
        .filter(|file| keywords.matches(&file.item))
        .map(|file| keep_fields(with_path(file), &args.fields))
        .collect();

    entries_to_yaml(&output, "rules")
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
