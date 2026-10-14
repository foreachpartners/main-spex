use std::process::ExitCode;

use anstream::print;
use clap::Args;

use crate::config::spex::Spex;
use crate::error::Result;
use crate::spec::Spec;
use crate::support::style::terminal_width;

use super::annotation_index::AnnotationIndex;
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
    #[arg(
        long,
        short,
        help = "Also show spec descriptions for planned specs, and spec/acceptance-criterion descriptions for each gap"
    )]
    verbose: bool,
}

pub fn run(args: CoverageArgs) -> Result<ExitCode> {
    let spex = Spex::discover()?;
    let short_names = spex.package_short_names(&args.package)?;

    let specs: Vec<Spec> = spex
        .specs()?
        .into_iter()
        .filter(|spec| spec.package_selected(&short_names))
        .collect();
    let index = AnnotationIndex::scan_project(&spex)?;

    let report = CoverageReport::new(&specs, &index);
    print!("{}", render(&report, terminal_width(), args.verbose));
    Ok(report.exit_code())
}
