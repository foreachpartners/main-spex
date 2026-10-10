use std::collections::HashMap;

use super::spex_config::PreexistingError;

#[cfg(test)]
#[path = "spex_yaml_edit_test.rs"]
mod spex_yaml_edit_test;

// Rewrites each in-scope repository's `ignored_errors` block in raw
// `spex.yaml` text, leaving the rest of the file byte-stable. `by_repo` is keyed
// on `repository_name` and holds every repository whose block should be
// regenerated (an empty entry list removes a now-stale block); repositories
// absent from the map are left untouched. The regenerated block is appended to
// the end of the repository's section.
pub fn set_preexisting_errors(
    raw: &str,
    by_repo: &HashMap<String, Vec<PreexistingError>>,
) -> String {
    let lines: Vec<&str> = raw.lines().collect();
    let mut out = String::with_capacity(raw.len());
    let mut pending: Option<String> = None;
    let mut in_scope = false;

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let indent = indent_of(line);
        let trimmed = line.trim_start();

        let repo_start = indent == 2 && trimmed.starts_with("- repository_name:");
        let boundary = repo_start || (indent == 0 && !trimmed.is_empty());

        // A repository start or any top-level key closes the previous section,
        // so flush its regenerated block before emitting the boundary line.
        if boundary {
            if let Some(block) = pending.take() {
                out.push_str(&block);
            }
            in_scope = false;
        }

        if repo_start {
            let name = trimmed["- repository_name:".len()..].trim();
            if let Some(entries) = by_repo.get(name) {
                in_scope = true;
                if !entries.is_empty() {
                    pending = Some(render_preexisting_block(entries));
                }
            }
            out.push_str(line);
            out.push('\n');
            i += 1;
            continue;
        }

        // Drop the existing block (its key line plus every more-indented child)
        // for in-scope repositories; the regenerated block replaces it.
        if in_scope && indent == 4 && trimmed.starts_with("ignored_errors:") {
            i += 1;
            while i < lines.len() && indent_of(lines[i]) > 4 {
                i += 1;
            }
            continue;
        }

        out.push_str(line);
        out.push('\n');
        i += 1;
    }

    if let Some(block) = pending.take() {
        out.push_str(&block);
    }
    out
}

fn render_preexisting_block(entries: &[PreexistingError]) -> String {
    let mut block = String::from("    ignored_errors:\n");
    for entry in entries {
        block.push_str(&format!("      - rule_id: {}\n", entry.rule_id));
        block.push_str(&format!("        check: {}\n", entry.check));
        if !entry.files.is_empty() {
            block.push_str("        files:\n");
            for file in &entry.files {
                block.push_str(&format!("          - {file}\n"));
            }
        }
    }
    block
}

// Rewrites the `cli_version` line in raw `spex.yaml` text, preserving comments
// and field order. Inserts it after `project_name` when absent.
pub fn set_cli_version(raw: &str, version: &str) -> String {
    let new_line = format!("cli_version: {version}");

    if raw.lines().any(|line| line.starts_with("cli_version:")) {
        let mut out = String::with_capacity(raw.len());
        for line in raw.lines() {
            if line.starts_with("cli_version:") {
                out.push_str(&new_line);
            } else {
                out.push_str(line);
            }
            out.push('\n');
        }
        return out;
    }

    let mut out = String::with_capacity(raw.len() + new_line.len() + 1);
    let mut inserted = false;
    for line in raw.lines() {
        out.push_str(line);
        out.push('\n');
        if !inserted && line.starts_with("project_name:") {
            out.push_str(&new_line);
            out.push('\n');
            inserted = true;
        }
    }
    if !inserted {
        out.insert_str(0, &format!("{new_line}\n"));
    }
    out
}

fn indent_of(line: &str) -> usize {
    line.chars().take_while(|ch| *ch == ' ').count()
}
