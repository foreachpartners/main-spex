use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use eros::Context;
use serde::Serialize;
use strum::IntoStaticStr;

use crate::config::Package;
use crate::config::spex::Spex;
use crate::error::Result;
use crate::spec::{AcId, CoverageLayer, SpecId};
use crate::support::fs::{canonical, relative_to};
use crate::support::package_files::PackageFiles;
use crate::tracing::annotations::{self, Annotation};

#[cfg(test)]
#[path = "annotation_index_test.rs"]
mod annotation_index_test;

#[derive(Debug, Default)]
pub(super) struct AnnotationIndex {
    strict: bool,
    by_spec: HashMap<SpecId, Vec<Trace>>,
    pub(super) packages: Vec<PackageScan>,
}

#[derive(Debug, Serialize)]
pub(super) struct Trace {
    pub(super) ac_id: Option<AcId>,
    pub(super) layer: CoverageLayer,
    pub(super) package: String,
    pub(super) path: PathBuf,
    pub(super) line: usize,
    pub(super) declared_contract: bool,
    pub(super) rejection: Option<Rejection>,
}

#[derive(Debug, Clone, Copy, Serialize, IntoStaticStr)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(super) enum Rejection {
    UndeclaredContract,
    ProseFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, IntoStaticStr)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(super) enum ScanState {
    Scanned,
    Missing,
    Unreadable,
}

impl ScanState {
    fn for_error(error: &std::io::Error) -> Self {
        if error.kind() == ErrorKind::NotFound {
            Self::Missing
        } else {
            Self::Unreadable
        }
    }
}

#[derive(Debug, Serialize)]
pub(super) struct ScannedPath {
    pub(super) path: PathBuf,
    pub(super) state: ScanState,
}

#[derive(Debug, Serialize)]
pub(super) struct PackageScan {
    pub(super) package: String,
    pub(super) short_name: String,
    pub(super) root: ScannedPath,
    pub(super) contracts: Vec<ScannedPath>,
    pub(super) source_files_scanned: usize,
    pub(super) non_utf8_files_skipped: usize,
    pub(super) unavailable_source_files: Vec<ScannedPath>,
}

impl AnnotationIndex {
    pub(super) fn traces(&self, spec_id: &SpecId) -> &[Trace] {
        self.by_spec
            .get(spec_id)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub(super) fn has_bare(&self, spec_id: &SpecId, layer: CoverageLayer) -> bool {
        self.traces(spec_id).iter().any(|trace| {
            trace.rejection.is_none()
                && trace.layer == layer
                && (trace.ac_id.is_none() || trace.layer == CoverageLayer::Contract)
        })
    }

    pub(super) fn has_ac(&self, spec_id: &SpecId, ac_id: &AcId, layer: CoverageLayer) -> bool {
        self.traces(spec_id).iter().any(|trace| {
            trace.rejection.is_none() && trace.layer == layer && trace.ac_id.as_ref() == Some(ac_id)
        })
    }

    #[cfg(test)]
    pub(super) fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub(super) fn scan(content: &str) -> Self {
        let mut index = Self::new();
        index.record_all(content, Path::new("src/lib.rs"), "package", false);
        index
    }

    pub(super) fn scan_project(spex: &Spex, strict: bool) -> Result<Self> {
        let mut index = Self {
            strict,
            ..Self::default()
        };
        let spex_dir = canonical(&spex.dir);
        let declared_contracts: HashSet<PathBuf> = spex
            .packages
            .iter()
            .flat_map(|package| package.contracts.iter().map(|path| canonical(path)))
            .collect();

        for package in &spex.packages {
            let scan = index.scan_package(package, &spex_dir, &declared_contracts)?;
            index.packages.push(scan);
        }
        Ok(index)
    }

    fn scan_package(
        &mut self,
        package: &Package,
        spex_dir: &Path,
        declared_contracts: &HashSet<PathBuf>,
    ) -> Result<PackageScan> {
        let state = match fs::metadata(&package.path) {
            Ok(metadata) if metadata.is_dir() => ScanState::Scanned,
            Ok(_) => ScanState::Missing,
            Err(error) => ScanState::for_error(&error),
        };
        let mut recorded_contracts = HashSet::new();
        let contracts = package
            .contracts
            .iter()
            .filter(|path| recorded_contracts.insert(canonical(path)))
            .map(|path| ScannedPath {
                path: path.clone(),
                state: self.scan_contract(path, &package.package_name),
            })
            .collect();
        let mut scan = PackageScan {
            package: package.package_name.clone(),
            short_name: package.package_short_name.clone(),
            root: ScannedPath {
                path: package.path.clone(),
                state,
            },
            contracts,
            source_files_scanned: 0,
            non_utf8_files_skipped: 0,
            unavailable_source_files: Vec::new(),
        };

        if state != ScanState::Scanned {
            return Ok(scan);
        }

        let root = canonical(&package.path);
        for result in PackageFiles::new(&package.path, &package.compile_exclude()?) {
            let path = match result {
                Err(error) if error.is_io() => {
                    scan.root.state = ScanState::Unreadable;
                    continue;
                }
                result => result.context("could not walk package files")?,
            };
            let resolved = root.join(relative_to(&path, &package.path));
            if resolved.starts_with(spex_dir) || declared_contracts.contains(&resolved) {
                continue;
            }
            match fs::read_to_string(&path) {
                Ok(content) => {
                    scan.source_files_scanned += 1;
                    self.record_all(&content, &path, &package.package_name, false);
                }
                Err(error) if error.kind() == ErrorKind::InvalidData => {
                    scan.non_utf8_files_skipped += 1;
                }
                Err(error) => scan.unavailable_source_files.push(ScannedPath {
                    path,
                    state: ScanState::for_error(&error),
                }),
            }
        }
        Ok(scan)
    }

    fn scan_contract(&mut self, path: &Path, package: &str) -> ScanState {
        match fs::metadata(path) {
            Ok(metadata) if metadata.is_file() => {}
            Ok(_) => return ScanState::Unreadable,
            Err(error) => return ScanState::for_error(&error),
        }
        match fs::read_to_string(path) {
            Ok(content) => {
                self.record_all(&content, path, package, true);
                ScanState::Scanned
            }
            Err(error) => ScanState::for_error(&error),
        }
    }

    fn record_all(&mut self, content: &str, path: &Path, package: &str, declared_contract: bool) {
        let annotations = annotations::parse(content);
        let is_prose = self.strict && is_prose_file(path);
        let mut last_offset = 0;
        let mut line = 1;
        for Annotation {
            layer,
            spec_id,
            ac_id,
            offset,
        } in annotations
        {
            line += content[last_offset..offset]
                .bytes()
                .filter(|&byte| byte == b'\n')
                .count();
            last_offset = offset;
            let rejection = match layer {
                CoverageLayer::Contract if self.strict && !declared_contract => {
                    Some(Rejection::UndeclaredContract)
                }
                CoverageLayer::Impl | CoverageLayer::Test if is_prose => Some(Rejection::ProseFile),
                _ => None,
            };
            self.by_spec.entry(spec_id).or_default().push(Trace {
                ac_id,
                layer,
                package: package.to_string(),
                path: path.to_path_buf(),
                line,
                declared_contract,
                rejection,
            });
        }
    }
}

fn is_prose_file(path: &Path) -> bool {
    if let Some(extension) = path.extension().and_then(|value| value.to_str()) {
        return matches!(
            extension.to_ascii_lowercase().as_str(),
            "adoc"
                | "asciidoc"
                | "markdown"
                | "md"
                | "mdx"
                | "mkd"
                | "mkdn"
                | "org"
                | "rst"
                | "text"
                | "txt"
        );
    }
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    [
        "AGENTS",
        "CHANGELOG",
        "CONTRIBUTING",
        "LICENSE",
        "NOTICE",
        "README",
    ]
    .iter()
    .any(|prose_name| name.eq_ignore_ascii_case(prose_name))
}
