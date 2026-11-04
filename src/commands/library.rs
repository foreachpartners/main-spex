use std::process::ExitCode;

use clap::{Args, ValueEnum};
use eros::{Context, IntoAnyUnion, bail};

use crate::error::Result;
use crate::support::entry_view::{
    Keywords, ensure_known_fields, entries_to_yaml, keep_fields, print_entries,
};
use crate::templates::{BUNDLED_TEMPLATES_DIR, SPECS_LIBRARY_DIRNAME};

#[cfg(test)]
#[path = "library_test.rs"]
mod library_test;

const ENTRY_FIELDS: [&str; 4] = ["id", "title", "description", "acceptance_criteria"];

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
    let yaml = render(&entries, &args.id, &args.search, &args.fields)?;
    print_entries(&yaml, args.output.as_deref())?;
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
    ensure_known_fields(fields, "library entry", &ENTRY_FIELDS)?;

    let keywords = Keywords::new(keywords);
    let selected: Vec<serde_json::Value> = parsed
        .into_iter()
        .filter(|(entry, object)| {
            (ids.is_empty() || ids.contains(&entry.id)) && keywords.matches(object)
        })
        .map(|(_, object)| keep_fields(object, fields))
        .collect();

    entries_to_yaml(&selected, "library entries")
}

fn parse(entry: &Entry) -> Result<serde_json::Value> {
    serde_yaml_ng::from_str(entry.text)
        .with_context(|| format!("could not parse library entry `{}`", entry.id))
        .any_union()
}
