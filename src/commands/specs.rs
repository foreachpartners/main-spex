use std::path::Path;
use std::process::ExitCode;

use clap::Args;
use error_stack::{Report, ResultExt};
use thiserror::Error;

use crate::spec::SPECS_DIR;
use crate::util;
use crate::workspace::Workspace;

#[cfg(test)]
#[path = "specs_test.rs"]
mod specs_test;

type Result<T> = core::result::Result<T, Report<SpecsError>>;

#[derive(Debug, Error)]
pub enum SpecsError {
    #[error("could not load the spex workspace")]
    Workspace,
    #[error("could not read `{0}`")]
    ReadFile(String),
    #[error("could not parse `{0}`")]
    Parse(String),
    #[error("could not serialize output")]
    Serialize,
    #[error("unknown repository_short_name `{0}` — not listed in spex.yaml")]
    UnknownScope(String),
    #[error("could not write `{0}`")]
    WriteFile(String),
}

#[derive(Args)]
pub struct SpecsArgs {
    #[arg(long, help = "Add a path field with the spec file's location")]
    ai: bool,
    #[arg(long, num_args = 1.., help = "Spec IDs to include (default: all)")]
    id: Vec<String>,
    #[arg(
        long,
        num_args = 1..,
        help = "Repository short names to include by spec `scope` (default: all)"
    )]
    scope: Vec<String>,
    #[arg(
        long,
        help = "Write the rendered YAML to this file instead of stdout — the full specs list is large and can get truncated when printed to a terminal or piped through a shell"
    )]
    output: Option<String>,
}

pub fn run(args: SpecsArgs) -> Result<ExitCode> {
    let workspace = Workspace::discover().change_context(SpecsError::Workspace)?;
    let yaml = render(&workspace, &args)?;
    match &args.output {
        Some(path) => {
            std::fs::write(path, &yaml)
                .change_context_lazy(|| SpecsError::WriteFile(path.clone()))?;
        }
        None => print!("{yaml}"),
    }
    Ok(ExitCode::SUCCESS)
}

fn render(workspace: &Workspace, args: &SpecsArgs) -> Result<String> {
    for short_name in &args.scope {
        if !workspace
            .config
            .repositories
            .iter()
            .any(|repo| &repo.short_name == short_name)
        {
            return Err(Report::new(SpecsError::UnknownScope(short_name.clone())));
        }
    }

    let mut specs = load_docs(&workspace.dir.join(SPECS_DIR))?;
    if args.ai {
        for (spec, path) in &mut specs {
            if let Some(obj) = spec.as_object_mut() {
                let mut with_path = serde_json::Map::with_capacity(obj.len() + 1);
                for (key, value) in std::mem::take(obj) {
                    let is_id = key == "id";
                    with_path.insert(key, value);
                    if is_id {
                        with_path.insert(
                            "path".to_string(),
                            serde_json::Value::String(format!("{SPECS_DIR}/{path}")),
                        );
                    }
                }
                *obj = with_path;
            }
        }
    }
    let specs: Vec<serde_json::Value> = specs.into_iter().map(|(spec, _)| spec).collect();

    let output: Vec<&serde_json::Value> = specs
        .iter()
        .filter(|spec| scope_selected(spec, &args.scope))
        .filter(|spec| id_selected(spec, &args.id))
        .collect();

    if output.is_empty() {
        return serde_yaml_ng::to_string(&output).change_context(SpecsError::Serialize);
    }

    output
        .iter()
        .map(|spec| serde_yaml_ng::to_string(&[spec]).change_context(SpecsError::Serialize))
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

// Loads every spec under `dir` as generic JSON, so the full document — every
// field the schema allows, not just the subset `Spec` deserializes for
// coverage and checks — is available to filter and print.
fn load_docs(dir: &Path) -> Result<Vec<(serde_json::Value, String)>> {
    let mut docs = Vec::new();
    for path in util::yaml_files(dir) {
        let rel_name = path
            .strip_prefix(dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .into_owned();
        let raw = std::fs::read_to_string(&path)
            .change_context_lazy(|| SpecsError::ReadFile(rel_name.clone()))?;
        let yaml: serde_yaml_ng::Value = serde_yaml_ng::from_str(&raw)
            .change_context_lazy(|| SpecsError::Parse(rel_name.clone()))?;
        let json = serde_json::to_value(yaml)
            .change_context_lazy(|| SpecsError::Parse(rel_name.clone()))?;
        docs.push((json, rel_name));
    }
    Ok(docs)
}
