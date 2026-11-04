use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anstream::println;
use clap::{Args, ValueEnum};
use eros::{Context, IntoAnyUnion, bail};

use crate::error::Result;
use crate::support::bundled_rule::{bundled_rule_paths, write_bundled_rule};
use crate::support::entry_view::{
    Keywords, ensure_known_fields, entries_to_yaml, keep_fields, print_entries, schema_fields,
};
use crate::support::fs::create_dir_all;
use crate::templates::{
    BUNDLED_TEMPLATES_DIR, RULE_SCHEMA_FILENAME, RULES_LIBRARY_DIRNAME, SPECS_LIBRARY_DIRNAME,
};

#[cfg(test)]
#[path = "library_test.rs"]
mod library_test;

const SPEC_ENTRY_FIELDS: [&str; 4] = ["id", "title", "description", "acceptance_criteria"];

#[derive(Clone, Copy, ValueEnum)]
enum LibKind {
    Specs,
    Rules,
}

impl LibKind {
    fn dirname(self) -> &'static str {
        match self {
            Self::Specs => SPECS_LIBRARY_DIRNAME,
            Self::Rules => RULES_LIBRARY_DIRNAME,
        }
    }

    fn fields(self) -> Vec<String> {
        match self {
            Self::Specs => SPEC_ENTRY_FIELDS.map(String::from).to_vec(),
            Self::Rules => schema_fields(RULE_SCHEMA_FILENAME),
        }
    }
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
    #[arg(
        long,
        value_name = "DIR",
        requires = "id",
        conflicts_with_all = ["search", "fields", "output"],
        help = "Write the rules named by --id, with their scripts, into this directory instead of printing them"
    )]
    copy: Option<PathBuf>,
}

struct Entry {
    id: String,
    text: &'static str,
}

pub fn run(args: LibraryArgs) -> Result<ExitCode> {
    ensure_known_fields(&args.fields, "library entry", &args.kind.fields())?;
    let entries = entries(args.kind.dirname());

    match &args.copy {
        Some(dir) => copy_rules(args.kind, &entries, &args.id, dir)?,
        None => {
            let yaml = render(&entries, &args.id, &args.search, &args.fields)?;
            print_entries(&yaml, args.output.as_deref())?;
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn entries(dirname: &str) -> Vec<Entry> {
    let dir = BUNDLED_TEMPLATES_DIR
        .get_dir(dirname)
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
                .expect("bundled entry has no file name")
                .to_string(),
            text: file.contents_utf8().expect("bundled entry is not UTF-8"),
        })
        .collect()
}

fn ensure_known_ids(entries: &[Entry], ids: &[String]) -> Result<()> {
    for id in ids {
        if !entries.iter().any(|entry| entry.id == *id) {
            bail!("unknown id `{id}` — not found in the library");
        }
    }
    Ok(())
}

fn render(
    entries: &[Entry],
    ids: &[String],
    keywords: &[String],
    fields: &[String],
) -> Result<String> {
    ensure_known_ids(entries, ids)?;

    let parsed = entries
        .iter()
        .map(|entry| Ok((entry, parse(entry)?)))
        .collect::<Result<Vec<_>>>()?;

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

fn copy_rules(kind: LibKind, entries: &[Entry], ids: &[String], dir: &Path) -> Result<()> {
    let LibKind::Rules = kind else {
        bail!("`--copy` copies rules only — draft a spec from `spex lib specs --id <id>` instead");
    };
    ensure_known_ids(entries, ids)?;

    let rules: Vec<_> = entries
        .iter()
        .filter(|entry| ids.contains(&entry.id))
        .map(|entry| {
            let file = BUNDLED_TEMPLATES_DIR
                .get_file(Path::new(RULES_LIBRARY_DIRNAME).join(format!("{}.yaml", entry.id)))
                .expect("bundled rule is missing");
            (file, dir.join(format!("{}.yaml", entry.id)))
        })
        .collect();

    for (file, dest) in &rules {
        let taken = bundled_rule_paths(file, dest)
            .into_iter()
            .find(|path| path.symlink_metadata().is_ok());
        if let Some(path) = taken {
            bail!(
                "`{}` already exists — `--copy` never overwrites a rule",
                path.display()
            );
        }
    }

    create_dir_all(dir)?;
    for (file, dest) in &rules {
        write_bundled_rule(file, dest)?;
        println!("{}", dest.display());
    }
    Ok(())
}

fn parse(entry: &Entry) -> Result<serde_json::Value> {
    serde_yaml_ng::from_str(entry.text)
        .with_context(|| format!("could not parse library entry `{}`", entry.id))
        .any_union()
}
