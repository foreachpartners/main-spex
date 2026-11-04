use std::path::Path;
use std::process::ExitCode;

use clap::{Args, ValueEnum};
use eros::{Context, IntoAnyUnion, bail};

use crate::error::Result;
use crate::support::fs::write_file;
use crate::templates::{BUNDLED_TEMPLATES_DIR, SPECS_LIBRARY_DIRNAME};

#[cfg(test)]
#[path = "library_test.rs"]
mod library_test;

const ENTRY_SEPARATOR: &str = "\n";

#[derive(Clone, ValueEnum)]
enum LibKind {
    Specs,
}

#[derive(Args)]
pub struct LibraryArgs {
    #[arg(help = "Library to read")]
    kind: LibKind,
    #[arg(long, num_args = 1.., help = "Print only these entries")]
    id: Vec<String>,
    #[arg(long, num_args = 1.., help = "Print only entries containing any of these keywords")]
    search: Vec<String>,
    #[arg(long, num_args = 1.., help = "Print only these fields of each entry")]
    fields: Vec<String>,
    #[arg(long, help = "Write the output to this file")]
    output: Option<String>,
}

struct Entry {
    id: String,
    text: &'static str,
}

pub fn run(args: LibraryArgs) -> Result<ExitCode> {
    let entries = match args.kind {
        LibKind::Specs => spec_entries(),
    };
    let output = render(&entries, &args.id, &args.search, &args.fields)?;
    if output.is_empty() {
        eprintln!("nothing found");
    }
    match &args.output {
        Some(path) => {
            write_file(Path::new(path), &output)?;
        }
        None => print!("{output}"),
    }
    Ok(ExitCode::SUCCESS)
}

fn spec_entries() -> Vec<Entry> {
    let dir = BUNDLED_TEMPLATES_DIR
        .get_dir(SPECS_LIBRARY_DIRNAME)
        .expect("bundled template directory is missing");
    let mut files: Vec<_> = dir
        .files()
        .filter(|file| {
            file.path()
                .extension()
                .is_some_and(|extension| extension == "yaml")
        })
        .collect();
    files.sort_by_key(|file| file.path());
    files
        .into_iter()
        .map(|file| Entry {
            id: file
                .path()
                .file_stem()
                .and_then(|stem| stem.to_str())
                .expect("bundled spec has no file name")
                .to_string(),
            text: file.contents_utf8().expect("bundled spec is not UTF-8"),
        })
        .collect()
}

fn render(
    entries: &[Entry],
    ids: &[String],
    keywords: &[String],
    fields: &[String],
) -> Result<String> {
    for id in ids {
        if !entries.iter().any(|entry| entry.id == *id) {
            bail!("unknown id `{id}` — not found in the library");
        }
    }

    let parsed = entries
        .iter()
        .map(|entry| Ok((entry, parse(entry)?)))
        .collect::<Result<Vec<_>>>()?;
    for field in fields {
        if !parsed.iter().any(|(_, object)| object.contains_key(field)) {
            bail!("unknown field `{field}` — not found in any entry of the library");
        }
    }

    let keywords: Vec<String> = keywords
        .iter()
        .map(|keyword| keyword.to_lowercase())
        .collect();
    let selected = parsed.into_iter().filter(|(entry, object)| {
        (ids.is_empty() || ids.contains(&entry.id))
            && (keywords.is_empty()
                || keywords
                    .iter()
                    .any(|keyword| object.values().any(|value| contains_text(value, keyword))))
    });

    selected
        .map(|(entry, object)| {
            if fields.is_empty() {
                return Ok(entry.text.to_string());
            }
            let kept: serde_json::Map<String, serde_json::Value> = object
                .into_iter()
                .filter(|(key, _)| fields.contains(key))
                .collect();
            serde_yaml_ng::to_string(&kept)
                .context("could not serialize library entry")
                .any_union()
        })
        .collect::<Result<Vec<_>>>()
        .map(|documents| documents.join(ENTRY_SEPARATOR))
}

fn contains_text(value: &serde_json::Value, lowercase_keyword: &str) -> bool {
    match value {
        serde_json::Value::String(text) => text.to_lowercase().contains(lowercase_keyword),
        serde_json::Value::Array(items) => items
            .iter()
            .any(|item| contains_text(item, lowercase_keyword)),
        serde_json::Value::Object(object) => object
            .values()
            .any(|item| contains_text(item, lowercase_keyword)),
        _ => false,
    }
}

fn parse(entry: &Entry) -> Result<serde_json::Map<String, serde_json::Value>> {
    serde_yaml_ng::from_str(entry.text)
        .with_context(|| format!("could not parse library entry `{}`", entry.id))
        .any_union()
}
