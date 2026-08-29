use globset::Glob;

use super::super::*;

pub(in crate::commands::check) fn run_glob(
    spec_id: &str,
    check: &Check,
    files: &[&ScopedFile],
    findings: &mut Vec<Finding>,
) {
    let CheckKind::Glob { pattern, negate } = &check.kind else {
        unreachable!("run_glob called with non-glob check")
    };
    let matcher = match Glob::new(pattern) {
        Ok(glob) => glob.compile_matcher(),
        Err(err) => {
            findings.push(config_error(
                spec_id,
                &check.name,
                format!("invalid glob: {err}"),
            ));
            return;
        }
    };

    let matched: Vec<&&ScopedFile> = files
        .iter()
        .filter(|file| matcher.is_match(&file.rel))
        .collect();

    if *negate {
        if matched.is_empty() {
            findings.push(violation(
                check,
                spec_id,
                None,
                message(check, &format!("required path not found: {pattern}")),
            ));
        }
    } else {
        for file in matched {
            findings.push(violation(
                check,
                spec_id,
                Some(rel_str(file)),
                message(check, "forbidden path"),
            ));
        }
    }
}
