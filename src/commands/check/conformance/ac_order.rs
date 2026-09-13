use std::path::Path;

use crate::rule::RULES_DIR;
use crate::spec::SPECS_DIR;
use crate::util::yaml_files;

use super::{Finding, SPEC_ID, Severity, rel};

const AC_ORDER_CHECK: &str = "ac-ordered";

// Verifies each spec file lists its `acceptance_criteria` in RFC 2119
// commitment order: MUST, MUST NOT, SHOULD, SHOULD NOT, MAY.
pub(super) fn validate_ac_order(spex_dir: &Path, findings: &mut Vec<Finding>) {
    for dir_name in [SPECS_DIR, RULES_DIR] {
        for file in yaml_files(&spex_dir.join(dir_name)) {
            check_ac_order(spex_dir, &file, findings);
        }
    }
}

fn rfc2119_rank(text: &str) -> Option<(u8, &'static str)> {
    if text.starts_with("MUST NOT") {
        Some((2, "MUST NOT"))
    } else if text.starts_with("MUST") {
        Some((1, "MUST"))
    } else if text.starts_with("SHOULD NOT") {
        Some((4, "SHOULD NOT"))
    } else if text.starts_with("SHOULD") {
        Some((3, "SHOULD"))
    } else if text.starts_with("MAY") {
        Some((5, "MAY"))
    } else {
        None
    }
}

fn check_ac_order(spex_dir: &Path, file: &Path, findings: &mut Vec<Finding>) {
    let Ok(text) = std::fs::read_to_string(file) else {
        return;
    };

    let mut in_ac = false;
    let mut prev_rank: u8 = 0;
    let mut prev_label = "";

    for (index, line) in text.lines().enumerate() {
        if line.starts_with("acceptance_criteria:") {
            in_ac = true;
            prev_rank = 0;
            prev_label = "";
            continue;
        }
        if !in_ac {
            continue;
        }
        if !line.starts_with(' ') && !line.starts_with('\t') {
            in_ac = false;
            continue;
        }

        let Some(mut item) = line.trim_start().strip_prefix("- ") else {
            continue;
        };
        item = item.trim();
        if item.len() >= 2 && item.starts_with('"') && item.ends_with('"') {
            item = &item[1..item.len() - 1];
        }
        let item = item
            .strip_prefix("[ ]")
            .or_else(|| item.strip_prefix("[x]"))
            .unwrap_or(item)
            .trim_start();

        let Some((rank, label)) = rfc2119_rank(item) else {
            continue;
        };
        if prev_rank != 0 && rank < prev_rank {
            findings.push(Finding {
                severity: Severity::Error,
                spec_id: SPEC_ID.to_string(),
                check: AC_ORDER_CHECK.to_string(),
                location: Some(format!("{}:{}", rel(spex_dir, file), index + 1)),
                message: format!(
                    "acceptance criteria out of order: {label} must not follow {prev_label} (order: MUST, MUST NOT, SHOULD, SHOULD NOT, MAY)"
                ),
            });
        }
        prev_rank = rank;
        prev_label = label;
    }
}
