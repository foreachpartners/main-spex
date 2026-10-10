use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Args;
use error_stack::{Report, ResultExt};
use jsonschema::{Retrieve, Uri, Validator};
use serde_json::Value;

use crate::error::{CliError, Result, describe};

#[cfg(test)]
#[path = "check_schema_test.rs"]
mod check_schema_test;

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
                eprintln!("{file}: {}", describe(&report));
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
        .change_context_lazy(|| CliError::ReadFile(schema_path.to_path_buf()))?;
    let schema: Value = serde_json::from_str(&text)
        .change_context_lazy(|| CliError::ParseFile(schema_path.to_path_buf()))?;

    let dir = schema_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    jsonschema::options()
        .with_retriever(DirRetriever { dir })
        .build(&schema)
        .map_err(|err| {
            Report::new(CliError::Internal(format!(
                "invalid schema `{}`: {err}",
                schema_path.display()
            )))
        })
}

pub(crate) fn validate_file(validator: &Validator, path: &Path) -> Result<Vec<String>> {
    let text =
        fs::read_to_string(path).change_context_lazy(|| CliError::ReadFile(path.to_path_buf()))?;
    let instance: Value = serde_yaml_ng::from_str(&text)
        .change_context_lazy(|| CliError::ParseFile(path.to_path_buf()))?;

    Ok(validate_instance(validator, &instance))
}

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
