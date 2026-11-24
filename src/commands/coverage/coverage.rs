use std::process::ExitCode;

use anstream::print;
use clap::Args;
use eros::bail;

use crate::config::spex::Spex;
use crate::error::Result;
use crate::spec::Spec;
use crate::support::output::{OutputFormat, print_json};
use crate::support::style::terminal_width;

use super::annotation_index::AnnotationIndex;
use super::evidence::CoverageEvidence;
use super::render::render;
use super::report::CoverageReport;

#[derive(Args)]
pub struct CoverageArgs {
    #[arg(
        long,
        num_args = 1..,
        help = "Filter by package names or short names"
    )]
    package: Vec<String>,
    #[arg(long, num_args = 1.., help = "Filter by spec IDs and show their trace locations")]
    id: Vec<String>,
    #[arg(
        long,
        help = "Require complete inputs and ignore traces from ineligible files"
    )]
    strict: bool,
    #[arg(long, value_enum, default_value = "text", help = "Report format")]
    format: OutputFormat,
    #[arg(
        long,
        short,
        help = "Also show spec descriptions for planned specs, and spec/acceptance-criterion descriptions for each gap"
    )]
    verbose: bool,
}

pub fn run(args: CoverageArgs, spex: Spex) -> Result<ExitCode> {
    let short_names = spex.package_short_names(&args.package)?;

    let specs = spex.specs()?;
    for id in &args.id {
        if !specs.iter().any(|spec| spec.id.as_str() == id) {
            bail!("unknown spec id `{id}`");
        }
    }
    let specs: Vec<Spec> = specs
        .into_iter()
        .filter(|spec| spec.package_selected(&short_names))
        .filter(|spec| args.id.is_empty() || args.id.iter().any(|id| spec.id.as_str() == id))
        .collect();
    let index = AnnotationIndex::scan_project(&spex, args.strict)?;

    let report = CoverageReport::new(&specs, &index);
    let evidence = CoverageEvidence::new(
        &index,
        &report,
        &short_names,
        args.strict,
        !args.id.is_empty(),
    );
    match args.format {
        OutputFormat::Text => print!(
            "{}",
            render(&report, terminal_width(), args.verbose, Some(&evidence))
        ),
        OutputFormat::Json => print_json(&evidence.json(&spex.project_name, &report))?,
    }
    Ok(if evidence.has_blocking_inputs() {
        ExitCode::FAILURE
    } else {
        report.exit_code()
    })
}
