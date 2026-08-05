use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Args;
use error_stack::{Report, ResultExt};
use jsonschema::{Retrieve, Uri, Validator};
use serde_json::Value;
use thiserror::Error;

#[cfg(test)]
#[path = "check_schema_test.rs"]
mod check_schema_test;

type Result<T> = core::result::Result<T, Report<CheckSchemaError>>;

#[derive(Debug, Error)]
pub enum CheckSchemaError {
    #[error("could not read `{0}`")]
    Read(String),
    #[error("could not parse `{0}`")]
    Parse(String),
    #[error("invalid schema `{0}`")]
    Schema(String),
}

#[derive(Args)]
pub struct CheckSchemaArgs {
    #[arg(long, help = "JSON Schema the files are validated against")]
    schema: String,
    #[arg(
        long,
        num_args = 1..,
        required = true,
        help = "JSON or YAML files to validate against the schema"
    )]
    files: Vec<String>,
}

pub fn run(args: CheckSchemaArgs) -> Result<ExitCode> {
    let validator = build_validator(Path::new(&args.schema))?;

    let mut failed = false;
    for file in &args.files {
        match validate_file(&validator, Path::new(file)) {
            Ok(messages) => {
                for message in messages {
                    failed = true;
                    eprintln!("{file}: {message}");
                }
            }
            Err(report) => {
                failed = true;
                eprintln!("{file}: {report:?}");
            }
        }
    }

    Ok(if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

pub(crate) fn build_validator(schema_path: &Path) -> Result<Validator> {
    let text = fs::read_to_string(schema_path)
        .change_context_lazy(|| CheckSchemaError::Read(display(schema_path)))?;
    let schema: Value = serde_json::from_str(&text)
        .change_context_lazy(|| CheckSchemaError::Parse(display(schema_path)))?;

    let dir = schema_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    jsonschema::options()
        .with_retriever(DirRetriever { dir })
        .build(&schema)
        // The error borrows `schema`, so render it before `schema` is dropped.
        .map_err(|err| {
            Report::new(CheckSchemaError::Schema(display(schema_path))).attach(err.to_string())
        })
}

pub(crate) fn validate_file(validator: &Validator, path: &Path) -> Result<Vec<String>> {
    let text =
        fs::read_to_string(path).change_context_lazy(|| CheckSchemaError::Read(display(path)))?;
    let instance: Value = serde_yaml_ng::from_str(&text)
        .change_context_lazy(|| CheckSchemaError::Parse(display(path)))?;

    Ok(validate_instance(validator, &instance))
}

// Validates a single already-parsed instance, sorted and deduplicated so
// callers get a stable, readable message list.
pub(crate) fn validate_instance(validator: &Validator, instance: &Value) -> Vec<String> {
    let mut messages: Vec<String> = validator
        .iter_errors(instance)
        .map(|err| {
            let location = err.instance_path().to_string();
            if location.is_empty() {
                err.to_string()
            } else {
                format!("{location}: {err}")
            }
        })
        .collect();
    messages.sort();
    messages.dedup();
    messages
}

fn display(path: &Path) -> String {
    path.display().to_string()
}

// Resolves cross-file `$ref`s against sibling schema files in the schema's own
// directory, so validation stays offline and uses the consumer's own schemas.
struct DirRetriever {
    dir: PathBuf,
}

impl Retrieve for DirRetriever {
    fn retrieve(
        &self,
        uri: &Uri<String>,
    ) -> core::result::Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let name = uri.as_str().rsplit('/').next().unwrap_or_default();
        let text = fs::read_to_string(self.dir.join(name))?;
        Ok(serde_json::from_str(&text)?)
    }
}
