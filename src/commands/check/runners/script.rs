use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::super::*;

// Cap files per script invocation so large repos don't blow past the OS arg limit.
const SCRIPT_BATCH_SIZE: usize = 1000;

pub(in crate::commands::check) fn run_workspace_checks(
    spex_dir: &Path,
    workspace_checks: &[WorkspaceCheck],
    findings: &mut Vec<Finding>,
) {
    for pending in workspace_checks {
        let Some(script) = locate_script(&pending.spec_id, &pending.check, spex_dir, findings)
        else {
            continue;
        };
        run_workspace_script(
            &pending.spec_id,
            &pending.check,
            &script,
            spex_dir,
            findings,
        );
    }
}

// Runs a `workspace`-scoped script exactly once in the spex directory, passing
// no files; the script discovers whatever it needs from that directory.
fn run_workspace_script(
    spec_id: &str,
    check: &Check,
    script: &Path,
    spex_dir: &Path,
    findings: &mut Vec<Finding>,
) {
    run_script_command(
        spec_id,
        check,
        script,
        spex_dir,
        std::iter::empty::<&OsStr>(),
        findings,
    );
}

// Spawns the script in `cwd` and records any failure. Returns `false` when the
// script could not be spawned so batched callers stop issuing further batches.
fn run_script_command<I, S>(
    spec_id: &str,
    check: &Check,
    script: &Path,
    cwd: &Path,
    args: I,
    findings: &mut Vec<Finding>,
) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = match Command::new(script).args(args).current_dir(cwd).output() {
        Ok(output) => output,
        Err(err) => {
            findings.push(run_error(spec_id, check, script, err));
            return false;
        }
    };

    if !output.status.success() {
        push_script_findings(spec_id, check, &output, findings);
    }
    true
}

// Parses a failed script's stdout into one finding per line, following the
// script location contract: each line is `LOCATION<TAB>MESSAGE`, where a
// missing `LOCATION` (empty, or no tab at all) yields a file-less finding. When
// the script printed nothing on stdout, falls back to a single file-less finding
// built from stderr or the exit status.
fn push_script_findings(
    spec_id: &str,
    check: &Check,
    output: &std::process::Output,
    findings: &mut Vec<Finding>,
) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let before = findings.len();
    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let (location, message) = parse_script_line(line);
        findings.push(violation(check, spec_id, location, message));
    }
    if findings.len() == before {
        findings.push(violation(check, spec_id, None, script_fallback(output)));
    }
}

fn parse_script_line(line: &str) -> (Option<String>, String) {
    match line.split_once('\t') {
        Some((location, message)) => {
            let location = location.trim();
            let location = (!location.is_empty()).then(|| location.to_string());
            (location, message.trim().to_string())
        }
        None => (None, line.trim().to_string()),
    }
}

// Resolves a script check's program path, pushing a config error when the check
// declares no script or the file is missing.
fn locate_script(
    spec_id: &str,
    check: &Check,
    spex_dir: &Path,
    findings: &mut Vec<Finding>,
) -> Option<PathBuf> {
    let CheckKind::Script {
        script: reference, ..
    } = &check.kind
    else {
        unreachable!("locate_script called with non-script check")
    };

    let Some(script) = resolve_script(reference, spex_dir, spec_id) else {
        let dir = spex_dir.join(rule::SCRIPTS_DIR).join(spec_id);
        findings.push(config_error(
            spec_id,
            &check.name,
            format!("script `{reference}` not found in {}", dir.display()),
        ));
        return None;
    };

    Some(script)
}

// Runs a `per-file` script once per batch of in-scope files, passing each
// file's repository-relative path as an argument.
pub(in crate::commands::check) fn run_per_file_script(
    spec_id: &str,
    check: &Check,
    repo_path: &Path,
    spex_dir: &Path,
    files: &[&ScopedFile],
    findings: &mut Vec<Finding>,
) {
    let Some(script) = locate_script(spec_id, check, spex_dir, findings) else {
        return;
    };
    for batch in files.chunks(SCRIPT_BATCH_SIZE) {
        let args = batch.iter().map(|file| file.rel.as_os_str());
        if !run_script_command(spec_id, check, &script, repo_path, args, findings) {
            return;
        }
    }
}

// Runs a `per-repo` script exactly once in the repository directory, passing no
// files; the script discovers whatever it needs from that directory.
pub(in crate::commands::check) fn run_per_repo_script(
    spec_id: &str,
    check: &Check,
    repo_path: &Path,
    spex_dir: &Path,
    findings: &mut Vec<Finding>,
) {
    let Some(script) = locate_script(spec_id, check, spex_dir, findings) else {
        return;
    };
    run_script_command(
        spec_id,
        check,
        &script,
        repo_path,
        std::iter::empty::<&OsStr>(),
        findings,
    );
}

// Scripts are stored at `<base>/scripts/<spec-id>/<reference>`, so the
// `script` field only carries the bare filename.
pub(in crate::commands::check) fn resolve_script(
    reference: &str,
    base: &Path,
    spec_id: &str,
) -> Option<PathBuf> {
    let script = base.join(rule::SCRIPTS_DIR).join(spec_id).join(reference);
    script.is_file().then_some(script)
}

fn run_error(spec_id: &str, check: &Check, script: &Path, err: std::io::Error) -> Finding {
    config_error(
        spec_id,
        &check.name,
        format!("could not run script `{}`: {err}", script.display()),
    )
}

fn script_fallback(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.trim().is_empty() {
        return stderr.trim().to_string();
    }
    let code = output
        .status
        .code()
        .map_or_else(|| "signal".to_string(), |code| code.to_string());
    format!("script exited with status {code}")
}
