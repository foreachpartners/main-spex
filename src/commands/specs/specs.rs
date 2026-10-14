use std::path::Path;
use std::process::ExitCode;

use clap::Args;
use eros::{Context, IntoAnyUnion, bail};

use crate::config::spex::Spex;
use crate::error::Result;
use crate::spec::SPECS_DIR;
use crate::support::ai_view::ai_view;
use crate::support::fs::write_file;
use crate::support::yaml_dir::load_yaml_dir;

#[cfg(test)]
#[path = "specs_test.rs"]
mod specs_test;

#[derive(Args)]
pub struct SpecsArgs {
    #[arg(long, help = "Drop fields AI doesn't need, add ones it does")]
    ai: bool,
    #[arg(long, num_args = 1.., help = "Filter by spec ids")]
    id: Vec<String>,
    #[arg(long, num_args = 1.., help = "Filter by package name or short name")]
    package: Vec<String>,
    #[arg(long, help = "Write the output to this file")]
    output: Option<String>,
}

pub fn run(args: SpecsArgs) -> Result<ExitCode> {
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

fn render(spex: &Spex, args: &SpecsArgs) -> Result<String> {
    let short_names = spex.package_short_names(&args.package)?;

    let specs: Vec<serde_json::Value> =
        load_yaml_dir::<serde_json::Value>(&spex.dir.join(SPECS_DIR))?
            .iter()
            .map(|file| {
                if args.ai {
                    ai_view(file, &[])
                } else {
                    file.item.clone()
                }
            })
            .collect();

    for id in &args.id {
        if !specs
            .iter()
            .any(|spec| spec.get("id").and_then(|v| v.as_str()) == Some(id.as_str()))
        {
            bail!("unknown spec id `{id}` — not found in {SPECS_DIR}");
        }
    }

    let output: Vec<&serde_json::Value> = specs
        .iter()
        .filter(|spec| package_selected(spec, &short_names))
        .filter(|spec| id_selected(spec, &args.id))
        .collect();

    if output.is_empty() {
        return serde_yaml_ng::to_string(&output)
            .context("could not serialize specs")
            .any_union();
    }

    output
        .iter()
        .map(|spec| {
            serde_yaml_ng::to_string(&[spec])
                .context("could not serialize specs")
                .any_union()
        })
        .collect::<Result<Vec<_>>>()
        .map(|docs| docs.join("\n"))
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
