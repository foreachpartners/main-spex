use std::process::ExitCode;

use clap::Args;
use eros::bail;

use crate::config::spex::Spex;
use crate::error::Result;
use crate::spec::SPECS_DIR;
use crate::support::entry_view::{
    Keywords, ensure_schema_fields, entries_to_yaml, keep_fields, print_entries, with_path,
};
use crate::support::yaml_dir::load_yaml_dir;
use crate::templates::SPEC_SCHEMA_FILENAME;

#[cfg(test)]
#[path = "specs_test.rs"]
mod specs_test;

#[derive(Args)]
pub struct SpecsArgs {
    #[arg(long, num_args = 1.., help = "Filter by spec ids")]
    id: Vec<String>,
    #[arg(long, num_args = 1.., help = "Filter by package name or short name")]
    package: Vec<String>,
    #[arg(long, num_args = 1.., help = "Print only specs containing any of these keywords")]
    search: Vec<String>,
    #[arg(long, num_args = 1.., help = "Print only these fields of each spec")]
    fields: Vec<String>,
    #[arg(long, help = "Write the output to this file")]
    output: Option<String>,
}

pub fn run(args: SpecsArgs) -> Result<ExitCode> {
    let spex = Spex::discover()?;
    let yaml = render(&spex, &args)?;
    print_entries(&yaml, args.output.as_deref())?;
    Ok(ExitCode::SUCCESS)
}

fn render(spex: &Spex, args: &SpecsArgs) -> Result<String> {
    let short_names = spex.package_short_names(&args.package)?;
    ensure_schema_fields(&args.fields, "spec", SPEC_SCHEMA_FILENAME)?;
    let keywords = Keywords::new(&args.search);

    let files = load_yaml_dir::<serde_json::Value>(&spex.dir.join(SPECS_DIR))?;

    for id in &args.id {
        if !files
            .iter()
            .any(|file| file.item.get("id").and_then(|v| v.as_str()) == Some(id.as_str()))
        {
            bail!("unknown spec id `{id}` — not found in {SPECS_DIR}");
        }
    }

    let output: Vec<serde_json::Value> = files
        .iter()
        .filter(|file| package_selected(&file.item, &short_names))
        .filter(|file| id_selected(&file.item, &args.id))
        .filter(|file| keywords.matches(&file.item))
        .map(|file| keep_fields(with_path(file), &args.fields))
        .collect();

    entries_to_yaml(&output, "specs")
}

fn package_selected(spec: &serde_json::Value, short_names: &[&str]) -> bool {
    if short_names.is_empty() {
        return true;
    }
    let spec_packages = spec
        .get("packages")
        .and_then(|v| v.as_array())
        .map(Vec::as_slice)
        .unwrap_or_default();
    short_names.iter().any(|short_name| {
        spec_packages
            .iter()
            .any(|name| name.as_str() == Some(short_name))
    })
}

fn id_selected(value: &serde_json::Value, ids: &[String]) -> bool {
    ids.is_empty()
        || value
            .get("id")
            .and_then(|v| v.as_str())
            .is_some_and(|value_id| ids.iter().any(|id| id == value_id))
}
