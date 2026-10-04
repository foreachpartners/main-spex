use regex::Regex;

use super::super::*;

pub(in crate::commands::check) fn run_regexp(
    spec_id: &str,
    check: &Check,
    files: &[&ScopedFile],
    findings: &mut Vec<Finding>,
) {
    let CheckKind::Regexp { pattern, negate } = &check.kind else {
        unreachable!("run_regexp called with non-regexp check")
    };
    let regex = match Regex::new(pattern) {
        Ok(regex) => regex,
        Err(err) => {
            findings.push(Finding::config_error(
                spec_id,
                &check.name,
                format!("invalid regexp: {err}"),
            ));
            return;
        }
    };

    for file in files {
        let Ok(content) = std::fs::read_to_string(&file.abs) else {
            continue;
        };

        if *negate {
            if !content.lines().any(|line| regex.is_match(line)) {
                findings.push(Finding::violation(
                    check,
                    spec_id,
                    Some(file.rel_str()),
                    check.message("required pattern not found"),
                ));
            }
        } else {
            for (index, line) in content.lines().enumerate() {
                if regex.is_match(line) {
                    findings.push(Finding::violation(
                        check,
                        spec_id,
                        Some(format!("{}:{}", file.rel_str(), index + 1)),
                        check.message("matches forbidden pattern"),
                    ));
                }
            }
        }
    }
}
