use std::path::Path;

use crate::config::{PreexistingError, Repository};
use crate::rule::{Check, Severity};

#[cfg(test)]
#[path = "finding_test.rs"]
mod finding_test;

pub(super) struct Finding {
    pub(super) severity: Severity,
    pub(super) rule_id: String,
    pub(super) check: String,
    pub(super) location: Option<String>,
    pub(super) message: String,
}

impl Finding {
    fn new(
        severity: Severity,
        rule_id: &str,
        check: &str,
        location: Option<String>,
        message: String,
    ) -> Self {
        Self {
            severity,
            rule_id: rule_id.to_string(),
            check: check.to_string(),
            location,
            message,
        }
    }

    pub(super) fn error(
        rule_id: &str,
        check: &str,
        location: Option<String>,
        message: String,
    ) -> Self {
        Self::new(Severity::Error, rule_id, check, location, message)
    }

    pub(super) fn warn(
        rule_id: &str,
        check: &str,
        location: Option<String>,
        message: String,
    ) -> Self {
        Self::new(Severity::Warn, rule_id, check, location, message)
    }

    pub(super) fn violation(
        check: &Check,
        rule_id: &str,
        location: Option<String>,
        message: String,
    ) -> Self {
        Self::new(check.severity, rule_id, &check.name, location, message)
    }

    pub(super) fn config_error(rule_id: &str, check: &str, message: String) -> Self {
        Self::error(rule_id, check, None, message)
    }

    // A repository-level error not tied to a specific named check (e.g. a missing
    // spec or a misconfigured override).
    pub(super) fn repo_error(rule_id: &str, message: String) -> Self {
        Self::config_error(rule_id, "", message)
    }

    pub(super) fn file(&self) -> Option<String> {
        self.location.as_deref().map(Self::normalize_location)
    }

    // Drops a finding when it matches a repository's pre-existing-error baseline: an
    // `error`-severity finding whose `(rule_id, check)` matches an entry and whose
    // location file is listed (or the entry lists no files, suppressing the whole
    // check). Warnings and unmatched errors always survive.
    pub(super) fn is_preexisting(&self, baseline: &[PreexistingError]) -> bool {
        self.severity == Severity::Error && self.matches_baseline(baseline, self.file().as_deref())
    }

    // Workspace findings carry a location relative to the spex directory, so
    // resolve each to its owning repository and match against that repository's
    // baseline. There is no workspace-level baseline.
    pub(super) fn is_preexisting_workspace(&self, repos: &[&Repository], spex_dir: &Path) -> bool {
        self.severity == Severity::Error
            && repos.iter().any(|repo| {
                self.file_in(repo, spex_dir)
                    .is_some_and(|file| self.matches_baseline(&repo.ignored_errors, Some(&file)))
            })
    }

    // Resolves a workspace finding to its owning repository and the repository-
    // relative file, by checking which repository's path contains its location.
    pub(super) fn owning_repo(
        &self,
        repos: &[&Repository],
        spex_dir: &Path,
    ) -> Option<(String, Option<String>)> {
        repos.iter().find_map(|repo| {
            self.file_in(repo, spex_dir)
                .map(|file| (repo.repository_name.clone(), Some(file)))
        })
    }

    fn file_in(&self, repo: &Repository, spex_dir: &Path) -> Option<String> {
        let located = spex_dir.join(self.file()?);
        let relative = located.strip_prefix(&repo.path).ok()?;
        (!relative.starts_with("..")).then(|| relative.to_string_lossy().into_owned())
    }

    fn matches_baseline(&self, baseline: &[PreexistingError], file: Option<&str>) -> bool {
        baseline.iter().any(|entry| {
            entry.rule_id == self.rule_id
                && entry.check == self.check
                && (entry.files.is_empty()
                    || file.is_some_and(|file| entry.files.iter().any(|listed| listed == file)))
        })
    }

    // Reduces a finding location to its file path by dropping a trailing `:<line>`,
    // so a regexp finding's `file:line` matches a baseline entry keyed on `file`.
    pub(super) fn normalize_location(location: &str) -> String {
        if let Some((path, line)) = location.rsplit_once(':')
            && !line.is_empty()
            && line.bytes().all(|byte| byte.is_ascii_digit())
        {
            return path.to_string();
        }
        location.to_string()
    }
}

pub(super) struct RepoReport {
    pub(super) name: String,
    pub(super) findings: Vec<Finding>,
}
