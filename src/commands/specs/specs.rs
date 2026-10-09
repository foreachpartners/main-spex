use std::path::Path;
use std::process::ExitCode;

use clap::Args;
use error_stack::{Report, ResultExt};

use crate::config::spex::Spex;
use crate::error::{CliError, Result};
use crate::spec::SPECS_DIR;
use crate::support::fs::write_file;
use crate::support::util;

#[cfg(test)]
#[path = "specs_test.rs"]
mod specs_test;

#[derive(Args)]
pub struct SpecsArgs {
    #[arg(long, help = "Drop fields AI doesn't need, add ones it does")]
    ai: bool,
    #[arg(long, num_args = 1.., help = "Spec IDs to include (default: all)")]
    id: Vec<String>,
    #[arg(
        long,
        num_args = 1..,
        value_name = "REPOSITORY_SHORT_NAME",
        help = "Repository short names to include by spec `scope` (default: all)"
    )]
    scope: Vec<String>,
    #[arg(
        long,
        help = "Write the output to this file instead of stdout — the full specs list is large and can get truncated when printed to a terminal or piped through a shell"
    )]
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
    for short_name in &args.scope {
        if !spex
            .repositories
            .iter()
            .any(|repo| &repo.repository_short_name == short_name)
        {
            return Err(Report::new(CliError::UnknownScope(short_name.clone())));
        }
    }

    let specs: Vec<serde_json::Value> =
        util::load_yaml_dir::<serde_json::Value>(&spex.dir.join(SPECS_DIR))?
            .iter()
            .map(|file| {
                if args.ai {
                    util::ai_view(file, &[])
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
            return Err(Report::new(CliError::UnknownSpecId(id.clone())));
        }
    }

    let output: Vec<&serde_json::Value> = specs
        .iter()
        .filter(|spec| scope_selected(spec, &args.scope))
        .filter(|spec| id_selected(spec, &args.id))
        .collect();

    if output.is_empty() {
        return serde_yaml_ng::to_string(&output)
            .change_context_lazy(|| CliError::Internal("could not serialize specs".into()));
    }

    output
        .iter()
        .map(|spec| {
            serde_yaml_ng::to_string(&[spec])
                .change_context_lazy(|| CliError::Internal("could not serialize specs".into()))
        })
        .collect::<Result<Vec<_>>>()
        .map(|docs| docs.join("\n"))
}

fn scope_selected(spec: &serde_json::Value, scopes: &[String]) -> bool {
    if scopes.is_empty() {
        return true;
    }
    let spec_scope = spec
        .get("scope")
        .and_then(|v| v.as_array())
        .map(Vec::as_slice)
        .unwrap_or_default();
    scopes.iter().any(|short_name| {
        spec_scope
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
