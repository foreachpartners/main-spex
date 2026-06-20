use std::collections::BTreeMap;
use std::fmt::Write;
use std::process::ExitCode;

use clap::Args;
use error_stack::{Report, ResultExt};
use thiserror::Error;

use crate::biz_spec::BizSpec;
use crate::tech_spec::TechSpec;
use crate::workspace::Workspace;

#[cfg(test)]
#[path = "spec_context_test.rs"]
mod spec_context_test;

type Result<T> = core::result::Result<T, Report<SpecContextError>>;

#[derive(Debug, Error)]
pub enum SpecContextError {
    #[error("could not load the spex workspace")]
    Workspace,
    #[error("unknown spec id: {0}")]
    Unknown(String),
}

#[derive(Args)]
pub struct SpecContextArgs {
    #[arg(
        num_args = 1..,
        required = true,
        help = "Spec IDs (BIZ or TECH) to gather context for"
    )]
    ids: Vec<String>,
}

pub fn run(args: SpecContextArgs) -> Result<ExitCode> {
    let workspace = Workspace::discover().change_context(SpecContextError::Workspace)?;
    let biz_specs = workspace
        .biz_specs()
        .change_context(SpecContextError::Workspace)?;
    let tech_specs = workspace
        .tech_specs()
        .change_context(SpecContextError::Workspace)?;

    let mut unknown = Vec::new();
    let sections: Vec<String> = args
        .ids
        .iter()
        .filter_map(|id| {
            if let Some(spec) = biz_specs.iter().find(|spec| &spec.id == id) {
                Some(render_biz(spec))
            } else if let Some(spec) = tech_specs.get(id) {
                Some(render_tech(spec))
            } else {
                unknown.push(id.clone());
                None
            }
        })
        .collect();

    if !unknown.is_empty() {
        return Err(Report::new(SpecContextError::Unknown(unknown.join(", "))));
    }

    print!("{}", sections.join("\n"));
    Ok(ExitCode::SUCCESS)
}

fn render_biz(spec: &BizSpec) -> String {
    let mut out = header(&spec.id, &spec.title, &spec.description);
    write_criteria(&mut out, &spec.acceptance_criteria);
    out
}

fn render_tech(spec: &TechSpec) -> String {
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
