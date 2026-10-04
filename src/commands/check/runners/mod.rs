use globset::{Glob, GlobSet, GlobSetBuilder};

use super::*;

mod ast_grep;
mod glob;
mod regexp;
mod script;

pub(super) use ast_grep::run_ast_grep;
pub(super) use glob::run_glob;
pub(super) use regexp::run_regexp;
pub(super) use script::{
    resolve_script, run_per_file_script, run_per_repo_script, run_workspace_checks,
};

impl RepoCtx<'_> {
    // Runs a non-workspace check against one repository, dispatching on the check's
    // kind and (for scripts) its run scope. Workspace scripts are deferred by
    // `dispatch_check` and run later in `run_workspace_checks`, so they no-op here.
    pub(super) fn run_check(&self, spec_id: &str, check: &Check, findings: &mut Vec<Finding>) {
        let scoped = match scope_files(check, self.candidates) {
            Ok(scoped) => scoped,
            Err(message) => {
                findings.push(Finding::config_error(spec_id, &check.name, message));
                return;
            }
        };

        match &check.kind {
            CheckKind::Regexp { .. } => run_regexp(spec_id, check, &scoped, findings),
            CheckKind::Glob { .. } => run_glob(spec_id, check, &scoped, findings),
            CheckKind::AstGrep { .. } => run_ast_grep(spec_id, check, &scoped, findings),
            CheckKind::Script {
                run: RunScope::PerFile,
                ..
            } => {
                run_per_file_script(
                    spec_id,
                    check,
                    self.repo_path,
                    self.spex_dir,
                    &scoped,
                    findings,
                );
            }
            CheckKind::Script {
                run: RunScope::PerRepo,
                ..
            } => {
                run_per_repo_script(spec_id, check, self.repo_path, self.spex_dir, findings);
            }
            CheckKind::Script {
                run: RunScope::Workspace,
                ..
            } => {}
        }
    }
}

pub(super) fn scope_files<'a>(
    check: &Check,
    files: &'a [ScopedFile],
) -> core::result::Result<Vec<&'a ScopedFile>, String> {
    let include =
        build_globset(&check.include).map_err(|err| format!("invalid include glob: {err}"))?;
    let exclude =
        build_globset(&check.exclude).map_err(|err| format!("invalid exclude glob: {err}"))?;

    Ok(files
        .iter()
        .filter(|file| {
            if let Some(include) = &include
                && !include.is_match(&file.rel)
            {
                return false;
            }
            if let Some(exclude) = &exclude
                && exclude.is_match(&file.rel)
            {
                return false;
            }
            true
        })
        .collect())
}

pub(super) fn build_globset(
    patterns: &[String],
) -> core::result::Result<Option<GlobSet>, globset::Error> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    Ok(Some(builder.build()?))
}
