use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use eros::{Context, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;

use crate::config::Package;
use crate::error::Result;
use crate::rule::Severity;

use super::baseline::split_findings;
use super::finding::Finding;
use super::report::{CheckReport, ReportTarget};

const SNAPSHOT_VERSION: u8 = 1;

#[derive(Serialize, Deserialize)]
pub(super) struct Snapshot {
    schema_version: u8,
    project_name: String,
    packages: BTreeSet<String>,
    findings: BTreeMap<String, usize>,
}

impl Snapshot {
    pub(super) fn load(path: &Path, project_name: &str, packages: &[&Package]) -> Result<Self> {
        let raw = fs::read(path)
            .with_context(|| format!("could not read baseline `{}`", path.display()))?;
        let snapshot: Self = serde_json::from_slice(&raw)
            .with_context(|| format!("invalid baseline JSON `{}`", path.display()))?;
        if snapshot.schema_version != SNAPSHOT_VERSION {
            bail!(
                "unsupported baseline schema version {}",
                snapshot.schema_version
            );
        }
        if snapshot.project_name != project_name || snapshot.packages != package_names(packages) {
            bail!("baseline project or selected package scope differs from the current check");
        }
        Ok(snapshot)
    }

    pub(super) fn save(
        path: &Path,
        project_name: &str,
        packages: &[&Package],
        reports: &[CheckReport<'_>],
    ) -> Result<()> {
        let mut snapshot = Self {
            schema_version: SNAPSHOT_VERSION,
            project_name: project_name.to_string(),
            packages: package_names(packages),
            findings: BTreeMap::new(),
        };
        let mut sources = HashMap::new();
        for report in reports {
            for finding in &report.findings {
                if finding.severity != Severity::Error {
                    continue;
                }
                let Some(key) = fingerprint(report.target, finding, &mut sources) else {
                    bail!(
                        "cannot capture baseline while `{}` has an execution or configuration error: {}",
                        report.target.name(),
                        finding.message
                    );
                };
                *snapshot.findings.entry(key).or_default() += 1;
            }
        }
        let parent = path
            .parent()
            .filter(|dir| !dir.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut file = NamedTempFile::new_in(parent)
            .with_context(|| format!("could not stage baseline `{}`", path.display()))?;
        serde_json::to_writer_pretty(&mut file, &snapshot)
            .context("could not serialize baseline")?;
        writeln!(file).context("could not write baseline")?;
        file.as_file()
            .sync_all()
            .context("could not synchronize baseline")?;
        file.persist_noclobber(path)
            .with_context(|| format!("could not create baseline `{}`", path.display()))?;
        Ok(())
    }

    pub(super) fn separate<'a>(mut self, reports: &mut [CheckReport<'a>]) -> Vec<CheckReport<'a>> {
        let mut sources = HashMap::new();
        split_findings(reports, |target, finding| {
            fingerprint(*target, finding, &mut sources)
                .and_then(|key| self.findings.get_mut(&key))
                .filter(|count| **count > 0)
                .is_some_and(|count| {
                    *count -= 1;
                    true
                })
        })
    }
}

fn package_names(packages: &[&Package]) -> BTreeSet<String> {
    packages
        .iter()
        .map(|package| package.package_name.clone())
        .collect()
}

fn fingerprint(
    target: ReportTarget<'_>,
    finding: &Finding,
    sources: &mut HashMap<PathBuf, Vec<String>>,
) -> Option<String> {
    if finding.severity != Severity::Error || finding.source.baseline_key().is_none() {
        return None;
    }

    let mut hasher = Sha256::new();
    let source = finding.source.to_string();
    for field in [target.name(), source.as_str(), finding.message.as_str()] {
        hash_field(&mut hasher, field);
    }
    if let Some(location) = &finding.location {
        hash_field(&mut hasher, &location.file.to_string_lossy());
        if let Some(line) = location.line.and_then(|number| number.checked_sub(1)) {
            let source = sources
                .entry(target.base_dir().join(&location.file))
                .or_insert_with_key(|path| {
                    fs::read_to_string(path)
                        .unwrap_or_default()
                        .lines()
                        .map(|line| line.trim().to_string())
                        .collect()
                });
            if let Some(content) = source.get(line) {
                hash_field(&mut hasher, content);
            }
        }
    }
    Some(format!("{:x}", hasher.finalize()))
}

fn hash_field(hasher: &mut Sha256, field: &str) {
    hasher.update((field.len() as u64).to_le_bytes());
    hasher.update(field.as_bytes());
}
