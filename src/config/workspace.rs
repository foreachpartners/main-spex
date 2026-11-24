use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use eros::{Context, bail};
use jsonschema::{Retrieve, Uri};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::Layout;
use crate::error::{Result, could_not_parse, could_not_read};
use crate::support::fs::{canonical, normalize};
use crate::templates::{BUNDLED_TEMPLATES_DIR, SCHEMAS_DIRNAME};

const REVISION_SCHEMA_VERSION: u8 = 1;

#[derive(Default)]
pub struct WorkspaceConfig {
    pub project_name: Option<String>,
    pub cli_version: Option<String>,
    layout: Option<Layout>,
    agent_root: Option<String>,
    packages: Vec<WorkspacePackage>,
}

#[derive(Deserialize)]
struct WorkspacePackage {
    package_name: String,
    path: String,
}

impl WorkspaceConfig {
    pub fn read(path: &Path) -> Result<Self> {
        let raw = fs::read_to_string(path).with_context(|| could_not_read(path))?;
        Self::parse(&raw, path)
    }

    pub fn parse(raw: &str, path: &Path) -> Result<Self> {
        let config: Value = serde_yaml_ng::from_str(raw).with_context(|| could_not_parse(path))?;
        let string_field = |name: &str| config.get(name)?.as_str().map(str::to_string);
        let packages = config
            .get("packages")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|package| serde_json::from_value(package.clone()).ok())
            .collect();

        Ok(Self {
            project_name: string_field("project_name"),
            cli_version: string_field("cli_version"),
            layout: config
                .get("layout")
                .and_then(|value| serde_json::from_value(value.clone()).ok()),
            agent_root: string_field("agent_root"),
            packages,
        })
    }

    pub fn package_paths<'a>(&'a self, dir: &'a Path) -> impl Iterator<Item = (&'a str, PathBuf)> {
        self.packages.iter().map(|package| {
            let path = normalize(&dir.join(&package.path));
            (package.package_name.as_str(), path)
        })
    }

    pub fn expected_git_root(&self, dir: &Path, package: &Path) -> Option<PathBuf> {
        match self.layout? {
            Layout::MultiRepo => Some(package.to_path_buf()),
            Layout::SingleRepo | Layout::Monorepo => self
                .agent_root
                .as_ref()
                .map(|root| normalize(&dir.join(root))),
        }
    }

    pub fn migration_start(&self) -> &str {
        let Some(version) = self.cli_version.as_deref() else {
            return "0";
        };
        let parts: Vec<_> = version.split('.').collect();
        if (1..=3).contains(&parts.len())
            && parts
                .iter()
                .all(|part| !part.is_empty() && part.bytes().all(|digit| digit.is_ascii_digit()))
        {
            version
        } else {
            "0"
        }
    }
}

pub fn configuration_issues(raw: &str, path: &Path) -> Result<Vec<String>> {
    let instance: Value = serde_yaml_ng::from_str(raw).with_context(|| could_not_parse(path))?;
    let schema = bundled_schema("spex.schema.json")
        .map_err(|error| eros::error!("could not load the bundled spex schema: {error}"))?;
    let validator = jsonschema::options()
        .with_retriever(BundledSchemas)
        .build(&schema)
        .map_err(|error| eros::error!("invalid bundled spex schema: {error}"))?;
    let mut issues: Vec<_> = validator
        .iter_errors(&instance)
        .map(|error| {
            let location = error.instance_path().to_string();
            if location.is_empty() {
                error.to_string()
            } else {
                format!("{location}: {error}")
            }
        })
        .collect();
    issues.sort();
    issues.dedup();
    Ok(issues)
}

type SchemaResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

struct BundledSchemas;

impl Retrieve for BundledSchemas {
    fn retrieve(&self, uri: &Uri<String>) -> SchemaResult<Value> {
        let name = uri.as_str().rsplit('/').next().unwrap_or_default();
        bundled_schema(name)
    }
}

fn bundled_schema(name: &str) -> SchemaResult<Value> {
    let path = Path::new(SCHEMAS_DIRNAME).join(name);
    let file = BUNDLED_TEMPLATES_DIR
        .get_file(&path)
        .ok_or_else(|| format!("bundled schema {} is missing", path.display()))?;
    Ok(serde_json::from_slice(file.contents())?)
}

#[derive(Serialize)]
pub struct RevisionStatus {
    pub git_root: Option<PathBuf>,
    pub head: Option<String>,
    pub dirty: Option<bool>,
    pub problem: Option<String>,
}

impl RevisionStatus {
    pub fn inspect(path: &Path) -> Self {
        Self::inspect_owned(path, None)
    }

    pub fn inspect_owned(path: &Path, expected_root: Option<&Path>) -> Self {
        if !path.is_dir() {
            return Self::unavailable(None, "package or project directory is missing".into());
        }
        let git_root = match read_git(path, &["rev-parse", "--show-toplevel"]) {
            Ok(root) => PathBuf::from(root),
            Err(problem) => return Self::unavailable(None, problem),
        };
        if let Some(expected) = expected_root
            && canonical(expected) != canonical(&git_root)
        {
            let reason = format!(
                "expected Git root {}, found {}",
                expected.display(),
                git_root.display()
            );
            return Self::unavailable(Some(git_root), reason);
        }

        let head = match read_git(path, &["rev-parse", "--verify", "HEAD^{commit}"]) {
            Ok(head) if is_full_commit(&head) => head,
            Ok(_) => {
                return Self::unavailable(
                    Some(git_root),
                    "Git did not return a full commit ID".into(),
                );
            }
            Err(problem) => return Self::unavailable(Some(git_root), problem),
        };

        match read_git(
            path,
            &[
                "status",
                "--porcelain=v1",
                "--untracked-files=normal",
                "--ignore-submodules=none",
            ],
        ) {
            Ok(status) => Self {
                git_root: Some(git_root),
                head: Some(head),
                dirty: Some(!status.is_empty()),
                problem: None,
            },
            Err(problem) => Self {
                git_root: Some(git_root),
                head: Some(head),
                dirty: None,
                problem: Some(problem),
            },
        }
    }

    fn unavailable(git_root: Option<PathBuf>, problem: String) -> Self {
        Self {
            git_root,
            head: None,
            dirty: None,
            problem: Some(problem),
        }
    }

    fn clean_head(&self, name: &str) -> std::result::Result<String, String> {
        match (&self.head, self.dirty) {
            (Some(head), Some(false)) => Ok(head.clone()),
            (_, Some(true)) => Err(format!("{name} has a dirty worktree")),
            _ => Err(format!(
                "{name} has no verified clean Git revision ({})",
                self.problem.as_deref().unwrap_or("Git status unavailable")
            )),
        }
    }
}

fn read_git(path: &Path, args: &[&str]) -> std::result::Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["-c", "core.fsmonitor=false"])
        .args(args)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_COMMON_DIR")
        .output()
        .map_err(|error| format!("could not run Git: {error}"))?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr);
        let detail = detail.trim();
        return Err(if detail.is_empty() {
            format!("Git exited with {}", output.status)
        } else {
            detail.to_string()
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn is_full_commit(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RevisionManifest {
    schema_version: u8,
    project: String,
    project_revision: String,
    packages: BTreeMap<String, String>,
}

impl RevisionManifest {
    pub fn read(path: &Path) -> Result<Self> {
        let raw = fs::read(path).with_context(|| could_not_read(path))?;
        let manifest: Self = serde_json::from_slice(&raw).with_context(|| could_not_parse(path))?;
        if manifest.schema_version != REVISION_SCHEMA_VERSION {
            bail!(
                "unsupported revision manifest schema version {}",
                manifest.schema_version
            );
        }
        if !is_full_commit(&manifest.project_revision)
            || !manifest
                .packages
                .values()
                .all(|revision| is_full_commit(revision))
        {
            bail!("revision manifest must contain full Git commit IDs");
        }
        Ok(manifest)
    }

    pub fn capture<'a>(
        project_name: &str,
        project: &RevisionStatus,
        packages: impl Iterator<Item = (&'a str, &'a RevisionStatus)>,
    ) -> Result<Self> {
        let mut issues = Vec::new();
        let project_revision = match project.clean_head("spex config repository") {
            Ok(head) => head,
            Err(issue) => {
                issues.push(issue);
                String::new()
            }
        };
        let mut revisions = BTreeMap::new();
        for (name, package) in packages {
            match package.clean_head(name) {
                Ok(head) => {
                    if revisions.insert(name.to_string(), head).is_some() {
                        issues.push(format!("package {name} is declared more than once"));
                    }
                }
                Err(issue) => issues.push(issue),
            }
        }
        if !issues.is_empty() {
            bail!("could not capture clean revisions:\n{}", issues.join("\n"));
        }
        Ok(Self {
            schema_version: REVISION_SCHEMA_VERSION,
            project: project_name.into(),
            project_revision,
            packages: revisions,
        })
    }

    pub fn mismatches<'a>(
        &self,
        project_name: &str,
        project: &RevisionStatus,
        packages: impl Iterator<Item = (&'a str, &'a RevisionStatus)>,
    ) -> Vec<String> {
        let mut issues = Vec::new();
        if self.project != project_name {
            issues.push(format!(
                "manifest project is {}, workspace project is {project_name}",
                self.project
            ));
        }
        compare_revision(
            &mut issues,
            "spex config repository",
            &self.project_revision,
            project,
        );
        let mut seen = BTreeSet::new();
        for (name, package) in packages {
            if !seen.insert(name) {
                issues.push(format!("package {name} is declared more than once"));
                continue;
            }
            if let Some(expected) = self.packages.get(name) {
                compare_revision(&mut issues, name, expected, package);
            } else {
                issues.push(format!("package {name} is missing from the manifest"));
            }
        }
        issues.extend(
            self.packages
                .keys()
                .filter(|name| !seen.contains(name.as_str()))
                .map(|name| format!("manifest package {name} is missing from the workspace")),
        );
        issues
    }
}

fn compare_revision(issues: &mut Vec<String>, name: &str, expected: &str, actual: &RevisionStatus) {
    if let Some(head) = &actual.head
        && head != expected
    {
        issues.push(format!("{name} expected {expected}, found {head}"));
    }
    if let Err(issue) = actual.clean_head(name) {
        issues.push(issue);
    }
}
