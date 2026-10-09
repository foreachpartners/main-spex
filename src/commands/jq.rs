use std::fs;
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Args;
use error_stack::{Report, ResultExt};
use jaq_all::jaq_core::{Vars, unwrap_valr};
use jaq_all::load::{FileReports, FileReportsDisp};
use jaq_all::{compile_with, data, defs};
use jaq_json::Val;
use jaq_json::read;
use jaq_json::write::{self, Pp};

use crate::error::{CliError, Result};

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
        .change_context_lazy(|| CliError::Internal("could not write to standard output".into()))?;
    Ok(ExitCode::SUCCESS)
}

fn read_input(file: Option<&Path>) -> Result<String> {
    match file {
        Some(path) => {
            fs::read_to_string(path).change_context_lazy(|| CliError::ReadFile(path.to_path_buf()))
        }
        None => {
            let mut text = String::new();
            io::stdin()
                .read_to_string(&mut text)
                .change_context_lazy(|| {
                    CliError::Internal("could not read standard input".into())
                })?;
            Ok(text)
        }
    }
}

fn evaluate(filter: &str, text: &str, output: Output, out: &mut dyn Write) -> Result<()> {
    let compiled = compile_with(filter, defs(), data::base_funs(), &[])
        .map_err(|reports| Report::new(CliError::Custom(describe_filter_errors(&reports))))?;
    let input = parse_input(text)?;
    let pp = printer(output);

    data::run(
        &data::Runner::default(),
        &compiled,
        Vars::default(),
        std::iter::once(Ok::<_, String>(input)),
        |message| Report::new(CliError::Internal(message)),
        |result| {
            let value = unwrap_valr(result)
                .map_err(|err| Report::new(CliError::Custom(format!("jq: {err}"))))?;
            write_value(out, &pp, output, &value).change_context_lazy(|| {
                CliError::Internal("could not write to standard output".into())
            })
        },
    )
}

fn parse_input(text: &str) -> Result<Val> {
    if let Ok(value) = read::parse_single(text.as_bytes()) {
        return Ok(value);
    }
    serde_yaml_ng::from_str(text).change_context_lazy(|| {
        CliError::Custom("could not parse the input as JSON or YAML".into())
    })
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
