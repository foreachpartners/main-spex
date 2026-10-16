use super::spex_config::PreexistingError;
use super::spex_yaml_edit::{indent_of, render_check_ignore};

#[cfg(test)]
#[path = "package_yaml_edit_test.rs"]
mod package_yaml_edit_test;

pub fn set_check_ignore(raw: &str, entries: &[PreexistingError]) -> String {
    let lines: Vec<&str> = raw.lines().collect();
    let existing = lines
        .iter()
        .position(|line| line.starts_with("check_ignore:"));
    if existing.is_none() && entries.is_empty() {
        return raw.to_string();
    }

    let (start, end) = match existing {
        Some(start) => (start, block_end(&lines, start)),
        None => {
            let before_coverage_ignore = lines
                .iter()
                .position(|line| line.starts_with("coverage_ignore:"))
                .unwrap_or(lines.len());
            (before_coverage_ignore, before_coverage_ignore)
        }
    };

    let mut out = String::with_capacity(raw.len());
    for line in &lines[..start] {
        out.push_str(line);
        out.push('\n');
    }
    if !entries.is_empty() {
        out.push_str(&render_check_ignore(entries, 0));
    }
    for line in &lines[end..] {
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn block_end(lines: &[&str], start: usize) -> usize {
    let mut end = start + 1;
    for (offset, line) in lines[start + 1..].iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        if indent_of(line) == 0 && !line.starts_with('-') {
            break;
        }
        end = start + 2 + offset;
    }
    end
}
