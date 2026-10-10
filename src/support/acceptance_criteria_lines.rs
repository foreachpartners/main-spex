#[cfg(test)]
#[path = "acceptance_criteria_lines_test.rs"]
mod acceptance_criteria_lines_test;

pub fn acceptance_criteria_lines(text: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    let mut in_block = false;
    for (index, line) in text.lines().enumerate() {
        if line.starts_with("acceptance_criteria:") {
            in_block = true;
            continue;
        }
        if !in_block {
            continue;
        }
        if !line.starts_with(' ') && !line.starts_with('\t') {
            in_block = false;
            continue;
        }
        if line.trim_start().starts_with("- ") {
            lines.push(index + 1);
        }
    }
    lines
}

pub fn with_source_lines<C, A>(
    criteria: Vec<C>,
    raw: &str,
    build: impl Fn(C, usize) -> A,
) -> Vec<A> {
    let lines = acceptance_criteria_lines(raw);
    criteria
        .into_iter()
        .enumerate()
        .map(|(index, criterion)| build(criterion, lines.get(index).copied().unwrap_or_default()))
        .collect()
}
