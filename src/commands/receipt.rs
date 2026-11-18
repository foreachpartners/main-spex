use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, ErrorKind, Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{Args, Subcommand};
use eros::{Context, bail};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use crate::error::{Result, could_not_parse, could_not_read, could_not_read_dir};
use crate::spec::SpecId;
use crate::support::fs::create_dir_all;
use crate::support::output::print_json;

const SCHEMA_VERSION: u32 = 1;
const MAX_RECEIPT_BYTES: u64 = 64 * 1024;
const MAX_STORED_BYTES: u64 = MAX_RECEIPT_BYTES + 256;
const MAX_REFERENCE_BYTES: usize = 512;

#[derive(Args)]
pub struct ReceiptArgs {
    #[arg(long, help = "Path to the receipt journal")]
    journal: PathBuf,
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    #[command(about = "Append a JSON receipt from a file or standard input (-)")]
    Append {
        #[arg(value_name = "JSON_FILE", allow_hyphen_values = true)]
        file: PathBuf,
    },
    #[command(about = "List recorded receipts")]
    List,
    #[command(about = "Show one recorded receipt")]
    Show { id: String },
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema_version: u32,
    id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    parent: Option<String>,
    context_digest: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    revisions: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    decision: Option<Decision>,
    verification: Verification,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    exception_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    unresolved_refs: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Decision {
    approval: Approval,
    reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reviewer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reference: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Approval {
    Approved,
    Rejected,
    Pending,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Verification {
    rules: Stage,
    trace_coverage: Stage,
    contract_validation: Stage,
    executed_tests: Stage,
    manual_acceptance: Stage,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Stage {
    status: Outcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reference: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Outcome {
    Passed,
    Failed,
    NotRun,
    Unavailable,
    Waived,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct StoredReceipt {
    appended_at_unix_seconds: u64,
    receipt: Receipt,
}

#[derive(Serialize)]
struct ReceiptSummary {
    id: String,
    parent: Option<String>,
    context_digest: String,
    appended_at_unix_seconds: u64,
}

pub fn run(args: ReceiptArgs) -> Result<ExitCode> {
    match args.action {
        Action::Append { file } => print_json(&append(&args.journal, &file)?)?,
        Action::List => print_json(&list(&args.journal)?)?,
        Action::Show { id } => print_json(&read_record(&args.journal, &id)?)?,
    }
    Ok(ExitCode::SUCCESS)
}

fn append(journal: &Path, input: &Path) -> Result<StoredReceipt> {
    let receipt: Receipt = if input == Path::new("-") {
        read_json(io::stdin().lock(), MAX_RECEIPT_BYTES).context("read receipt from stdin")?
    } else {
        let file = File::open(input).with_context(|| could_not_read(input))?;
        read_json(file, MAX_RECEIPT_BYTES).with_context(|| could_not_parse(input))?
    };
    validate_receipt(&receipt)?;
    if let Some(parent) = &receipt.parent {
        read_record(journal, parent).with_context(|| format!("parent receipt `{parent}`"))?;
    }

    create_dir_all(journal)?;
    let record = StoredReceipt {
        appended_at_unix_seconds: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("could not record the append time")?
            .as_secs(),
        receipt,
    };
    let path = receipt_path(journal, &record.receipt.id);
    let mut temporary = NamedTempFile::new_in(journal)
        .with_context(|| format!("could not create a receipt in `{}`", journal.display()))?;
    serde_json::to_writer(temporary.as_file_mut(), &record)
        .context("could not serialize receipt")?;
    writeln!(temporary).context("could not finish writing receipt")?;
    temporary
        .as_file()
        .sync_all()
        .context("could not sync receipt")?;
    match temporary.persist_noclobber(&path) {
        Ok(_) => Ok(record),
        Err(error) if error.error.kind() == ErrorKind::AlreadyExists => {
            let existing = read_record(journal, &record.receipt.id)?;
            if existing.receipt != record.receipt {
                bail!(
                    "receipt `{}` already exists with different data",
                    record.receipt.id
                );
            }
            Ok(existing)
        }
        Err(error) => Err(eros::error!(
            "could not publish receipt `{}`: {}",
            path.display(),
            error.error
        )),
    }
}

fn list(journal: &Path) -> Result<Vec<ReceiptSummary>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(journal).with_context(|| could_not_read_dir(journal))? {
        let entry = entry.with_context(|| could_not_read_dir(journal))?;
        let path = entry.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            paths.push(path);
        }
    }
    paths.sort();
    paths
        .iter()
        .map(|path| {
            let Some(id) = path.file_stem().and_then(|stem| stem.to_str()) else {
                bail!("invalid receipt filename `{}`", path.display());
            };
            let record = read_record(journal, id)?;
            Ok(ReceiptSummary {
                id: record.receipt.id,
                parent: record.receipt.parent,
                context_digest: record.receipt.context_digest,
                appended_at_unix_seconds: record.appended_at_unix_seconds,
            })
        })
        .collect()
}

fn read_record(journal: &Path, id: &str) -> Result<StoredReceipt> {
    validate_id(id)?;
    let path = receipt_path(journal, id);
    let metadata = fs::symlink_metadata(&path).with_context(|| could_not_read(&path))?;
    if !metadata.file_type().is_file() {
        bail!("receipt `{}` must be a regular file", path.display());
    }
    let file = File::open(&path).with_context(|| could_not_read(&path))?;
    let record: StoredReceipt =
        read_json(file, MAX_STORED_BYTES).with_context(|| could_not_parse(&path))?;
    validate_receipt(&record.receipt).with_context(|| could_not_parse(&path))?;
    if record.receipt.id != id {
        bail!(
            "receipt `{id}` has mismatched stored ID `{}`",
            record.receipt.id
        );
    }
    Ok(record)
}

fn receipt_path(journal: &Path, id: &str) -> PathBuf {
    journal.join(format!("{id}.json"))
}

fn read_json<T: DeserializeOwned>(input: impl io::Read, limit: u64) -> Result<T> {
    let mut bytes = Vec::new();
    input
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .context("could not read receipt JSON")?;
    if bytes.len() as u64 > limit {
        bail!("receipt JSON exceeds {limit} bytes");
    }
    Ok(serde_json::from_slice(&bytes).context("could not decode receipt JSON")?)
}

fn validate_id(id: &str) -> Result<()> {
    if SpecId::try_from(id.to_owned()).is_err() {
        bail!("invalid receipt ID `{id}` — use the spec ID grammar");
    }
    Ok(())
}

fn is_lower_hex(value: &str, lengths: &[usize]) -> bool {
    lengths.contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn verify_text(value: &str, field: &str) -> Result<()> {
    if value.trim().is_empty()
        || value.len() > MAX_REFERENCE_BYTES
        || value.chars().any(char::is_control)
    {
        bail!("`{field}` must be one nonempty line of at most {MAX_REFERENCE_BYTES} bytes");
    }
    Ok(())
}

fn validate_receipt(receipt: &Receipt) -> Result<()> {
    if receipt.schema_version != SCHEMA_VERSION {
        bail!(
            "unsupported receipt schema version `{}`",
            receipt.schema_version
        );
    }
    validate_id(&receipt.id)?;
    if let Some(parent) = &receipt.parent {
        validate_id(parent)?;
        if parent == &receipt.id {
            bail!("receipt `{parent}` cannot name itself as its parent");
        }
    }
    if !is_lower_hex(&receipt.context_digest, &[64]) {
        bail!("`context_digest` must be a lowercase hexadecimal SHA-256 digest");
    }
    for (package, revision) in &receipt.revisions {
        verify_text(package, "revisions package name")?;
        if !is_lower_hex(revision, &[40, 64]) {
            bail!(
                "revision for `{package}` must be a complete lowercase hexadecimal Git commit ID"
            );
        }
    }
    if let Some(decision) = &receipt.decision {
        validate_decision(decision)?;
    }
    receipt.verification.validate()?;
    for (field, values) in [
        ("exception_refs", &receipt.exception_refs),
        ("unresolved_refs", &receipt.unresolved_refs),
    ] {
        for reference in values {
            verify_text(reference, field)?;
        }
    }
    Ok(())
}

fn validate_decision(decision: &Decision) -> Result<()> {
    verify_text(&decision.reason, "decision.reason")?;
    if decision.reviewer.is_none() && decision.reference.is_none() {
        bail!("`decision` needs a reviewer or an issue reference");
    }
    if let Some(reviewer) = &decision.reviewer {
        verify_text(reviewer, "decision.reviewer")?;
    }
    if let Some(reference) = &decision.reference {
        verify_text(reference, "decision.reference")?;
    }
    Ok(())
}

impl Verification {
    fn validate(&self) -> Result<()> {
        for (name, stage) in [
            ("rules", &self.rules),
            ("trace_coverage", &self.trace_coverage),
            ("contract_validation", &self.contract_validation),
            ("executed_tests", &self.executed_tests),
            ("manual_acceptance", &self.manual_acceptance),
        ] {
            if let Some(reference) = &stage.reference {
                verify_text(reference, name)?;
            } else if matches!(
                stage.status,
                Outcome::Passed | Outcome::Failed | Outcome::Waived
            ) {
                bail!("`{name}` needs an evidence or exception reference for that status");
            }
        }
        Ok(())
    }
}
