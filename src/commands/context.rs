use std::collections::BTreeMap;
use std::fmt::Write;
use std::process::ExitCode;

use clap::Args;
use error_stack::{Report, ResultExt};
use thiserror::Error;

use crate::capability_spec::CapabilitySpec;
use crate::engineering_spec::EngineeringSpec;
use crate::workspace::Workspace;

#[cfg(test)]
#[path = "context_test.rs"]
mod context_test;

type Result<T> = core::result::Result<T, Report<ContextError>>;

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("could not load the spex workspace")]
    Workspace,
    #[error("unknown spec id: {0}")]
    Unknown(String),
}

#[derive(Args)]
pub struct ContextArgs {
    #[arg(
        num_args = 1..,
        required = true,
        help = "Spec IDs (CS or ES) to gather context for"
    )]
    ids: Vec<String>,
}

pub fn run(args: ContextArgs) -> Result<ExitCode> {
    let workspace = Workspace::discover().change_context(ContextError::Workspace)?;
    let cs_specs = workspace
        .cs_specs()
        .change_context(ContextError::Workspace)?;
    let es_specs = workspace
        .es_specs()
        .change_context(ContextError::Workspace)?;

    let mut unknown = Vec::new();
    let sections: Vec<String> = args
        .ids
        .iter()
        .filter_map(|id| {
            if let Some(spec) = cs_specs.iter().find(|spec| &spec.id == id) {
                Some(render_capability(spec))
            } else if let Some(spec) = es_specs.get(id) {
                Some(render_engineering(spec))
            } else {
                unknown.push(id.clone());
                None
            }
        })
        .collect();

    if !unknown.is_empty() {
        return Err(Report::new(ContextError::Unknown(unknown.join(", "))));
    }

    print!("{}", sections.join("\n"));
    Ok(ExitCode::SUCCESS)
}

fn render_capability(spec: &CapabilitySpec) -> String {
    let mut out = header(&spec.id, &spec.title, &spec.description);
    write_criteria(&mut out, &spec.acceptance_criteria);
    out
}

fn render_engineering(spec: &EngineeringSpec) -> String {
    let mut out = header(&spec.id, &spec.title, &spec.description);
    write_criteria(&mut out, &spec.acceptance_criteria);
    write_examples(&mut out, &spec.examples);
    out
}

fn header(id: &str, title: &str, description: &str) -> String {
    let mut out = String::new();
    if title.is_empty() {
        let _ = writeln!(out, "# {id}");
    } else {
        let _ = writeln!(out, "# {id} — {title}");
    }
    let description = description.trim();
    if !description.is_empty() {
        let _ = writeln!(out, "\n{description}");
    }
    out
}

fn write_criteria(out: &mut String, criteria: &[String]) {
    if criteria.is_empty() {
        return;
    }
    let _ = write!(out, "\n## Acceptance criteria\n\n");
    for criterion in criteria {
        let _ = writeln!(out, "- {criterion}");
    }
}

fn write_examples(out: &mut String, examples: &BTreeMap<String, String>) {
    if examples.is_empty() {
        return;
    }
    let _ = write!(out, "\n## Examples\n");
    for (label, body) in examples {
        let _ = write!(out, "\n### {label}\n\n```\n{}\n```\n", body.trim_end());
    }
}
