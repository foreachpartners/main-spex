use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use super::super::finding::Location;
use super::super::scoped_file::ScopedFile;
use super::{Hit, Outcome};

const SCRIPT_BATCH_SIZE: usize = 1000;
const SPEX_DIR_ENV: &str = "SPEX_DIR";

pub(in crate::commands::check) fn run_per_file_script(
    program: &Path,
    repo_path: &Path,
    spex_dir: &Path,
    files: &[&ScopedFile],
) -> Outcome {
    let mut hits = Vec::new();
    for batch in files.chunks(SCRIPT_BATCH_SIZE) {
        hits.extend(run_script(
            program,
            repo_path,
            spex_dir,
            batch.iter().map(|file| file.rel.as_os_str()),
        )?);
    }
    Ok(hits)
}

pub(in crate::commands::check) fn run_script_once(
    program: &Path,
    cwd: &Path,
    spex_dir: &Path,
) -> Outcome {
    run_script(program, cwd, spex_dir, std::iter::empty::<&OsStr>())
}

fn run_script<I, S>(program: &Path, cwd: &Path, spex_dir: &Path, args: I) -> Outcome
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .env(SPEX_DIR_ENV, spex_dir)
        .output()
        .map_err(|err| format!("could not run script `{}`: {err}", program.display()))?;

    Ok(if output.status.success() {
        Vec::new()
    } else {
        script_hits(&output)
    })
}

fn script_hits(output: &Output) -> Vec<Hit> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let hits: Vec<Hit> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(parse_script_line)
        .collect();
    if hits.is_empty() {
        vec![Hit::new(None, script_fallback(output))]
    } else {
        hits
    }
}

fn parse_script_line(line: &str) -> Hit {
    match line.split_once('\t') {
        Some((location, message)) => {
            let location = location.trim();
            Hit::new(
                (!location.is_empty()).then(|| Location::parse(location)),
                message.trim(),
            )
        }
        None => Hit::new(None, line.trim()),
    }
}

pub(in crate::commands::check) fn locate_script(
    script_dir: &Path,
    reference: &str,
) -> core::result::Result<PathBuf, String> {
    resolve_script(script_dir, reference)
        .ok_or_else(|| format!("script `{reference}` not found in {}", script_dir.display()))
}

pub(in crate::commands::check) fn resolve_script(
    script_dir: &Path,
    reference: &str,
) -> Option<PathBuf> {
    let script = script_dir.join(reference);
    script.is_file().then_some(script)
}

fn script_fallback(output: &Output) -> String {
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
