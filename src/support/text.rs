#[cfg(test)]
#[path = "text_test.rs"]
mod text_test;

pub fn plural(count: usize, noun: &str) -> String {
    if count == 1 {
        format!("1 {noun}")
    } else {
        format!("{count} {noun}s")
    }
}

pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

pub fn level_two_sections(markdown: &str) -> Vec<String> {
    let mut sections: Vec<String> = Vec::new();
    for line in markdown.lines() {
        if line.starts_with("## ") {
            sections.push(String::new());
        }
        if let Some(section) = sections.last_mut() {
            section.push_str(line);
            section.push('\n');
        }
    }
    sections
}
