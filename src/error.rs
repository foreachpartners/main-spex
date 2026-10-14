use std::backtrace::BacktraceStatus;
use std::path::Path;

use eros::ErrorUnion;
pub use eros::Result;

use crate::config::SPEX_CONFIG_FILE;

#[cfg(test)]
#[path = "error_test.rs"]
mod error_test;

pub fn describe(error: &ErrorUnion) -> String {
    error
        .contexts()
        .rev()
        .map(ToString::to_string)
        .chain(std::iter::once(error.inner().to_string()))
        .collect::<Vec<_>>()
        .join(": ")
}

pub fn failure_report(error: &ErrorUnion) -> String {
    let summary = format!("error: {}", describe(error));
    match error.backtrace().status() {
        BacktraceStatus::Captured => format!("{summary}\n\n{error:?}"),
        _ => summary,
    }
}

#[track_caller]
pub fn config_not_found() -> ErrorUnion {
    eros::error!("`{SPEX_CONFIG_FILE}` not found — run `spex` from the project root or any package")
}

pub fn could_not_read(path: &Path) -> String {
    format!("could not read `{}`", path.display())
}

pub fn could_not_parse(path: &Path) -> String {
    format!("could not parse `{}`", path.display())
}

pub fn could_not_read_dir(path: &Path) -> String {
    format!("could not read directory `{}`", path.display())
}
