use std::fs;
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Args;
use eros::{Context, IntoAnyUnion};
use jaq_all::jaq_core::{Vars, unwrap_valr};
use jaq_all::load::{FileReports, FileReportsDisp};
use jaq_all::{compile_with, data, defs};
use jaq_json::Val;
use jaq_json::read;
use jaq_json::write::{self, Pp};

use crate::error::{Result, could_not_read};

#[cfg(test)]
#[path = "jq_test.rs"]
mod jq_test;

const PRETTY_INDENT: &str = "  ";

#[derive(Args)]
pub struct JqArgs {
    #[arg(help = "jq filter to run on the input, e.g. `.project_name`")]
    filter: String,
    #[arg(help = "JSON or YAML file to read; reads standard input when omitted")]
    file: Option<PathBuf>,
    #[arg(short, long, help = "Print string results without quotes")]
    raw_output: bool,
    #[arg(short, long, help = "Print each result on a single line")]
    compact_output: bool,
}

#[derive(Clone, Copy)]
struct Output {
    raw: bool,
    compact: bool,
}

pub fn run(args: JqArgs) -> Result<ExitCode> {
    let text = read_input(args.file.as_deref())?;
    let output = Output {
        raw: args.raw_output,
        compact: args.compact_output,
    };
    let mut stdout = BufWriter::new(io::stdout().lock());
    evaluate(&args.filter, &text, output, &mut stdout)?;
    stdout
        .flush()
        .context("could not write to standard output")?;
    Ok(ExitCode::SUCCESS)
}

fn read_input(file: Option<&Path>) -> Result<String> {
    match file {
        Some(path) => fs::read_to_string(path)
            .with_context(|| could_not_read(path))
            .any_union(),
        None => {
            let mut text = String::new();
            io::stdin()
                .read_to_string(&mut text)
                .context("could not read standard input")?;
            Ok(text)
        }
    }
}

fn evaluate(filter: &str, text: &str, output: Output, out: &mut dyn Write) -> Result<()> {
    let compiled = compile_with(filter, defs(), data::base_funs(), &[])
        .map_err(|reports| eros::error!("{}", describe_filter_errors(&reports)))?;
    let input = parse_input(text)?;
    let pp = printer(output);

    data::run(
        &data::Runner::default(),
        &compiled,
        Vars::default(),
        std::iter::once(Ok::<_, String>(input)),
        |message| eros::error!("{message}"),
        |result| {
            let value = unwrap_valr(result).map_err(|err| eros::error!("jq: {err}"))?;
            write_value(out, &pp, output, &value).context("could not write to standard output")?;
            Ok(())
        },
    )
}

fn parse_input(text: &str) -> Result<Val> {
    if let Ok(value) = read::parse_single(text.as_bytes()) {
        return Ok(value);
    }
    serde_yaml_ng::from_str(text)
        .context("could not parse the input as JSON or YAML")
        .any_union()
}

fn printer(output: Output) -> Pp {
    Pp {
        indent: (!output.compact).then(|| PRETTY_INDENT.to_string()),
        sep_space: !output.compact,
        ..Pp::default()
    }
}

fn write_value(out: &mut dyn Write, pp: &Pp, output: Output, value: &Val) -> io::Result<()> {
    match value {
        Val::TStr(text) if output.raw => out.write_all(&text[..])?,
        _ => write::write(out, pp, 0, value)?,
    }
    out.write_all(b"\n")
}

fn describe_filter_errors(reports: &[FileReports]) -> String {
    let rendered = reports
        .first()
        .map(|report| FileReportsDisp::new(report).to_string())
        .unwrap_or_default();
    let message = rendered
        .lines()
        .next()
        .unwrap_or_default()
        .trim_start_matches("Error: ");
    format!("invalid jq filter: {message}")
}
