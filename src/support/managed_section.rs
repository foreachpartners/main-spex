#[cfg(test)]
#[path = "managed_section_test.rs"]
mod managed_section_test;

pub const PROJECT_SECTION_MARKER: &str = "\n---\n\n# Project";

pub fn split_project_section(contents: &str) -> Option<(&str, &str)> {
    let idx = contents.find(PROJECT_SECTION_MARKER)?;
    Some(contents.split_at(idx))
}

pub fn replace_managed_section(template: &[u8], local: &str) -> Vec<u8> {
    let Some((_, preserved)) = split_project_section(local) else {
        return template.to_vec();
    };
    let Some((managed, _)) = std::str::from_utf8(template)
        .ok()
        .and_then(split_project_section)
    else {
        return local.as_bytes().to_vec();
    };
    format!("{managed}{preserved}").into_bytes()
}

pub fn keep_as_project_section(template: &[u8], existing: &str) -> Vec<u8> {
    if split_project_section(existing).is_some() {
        return replace_managed_section(template, existing);
    }
    let Some((managed, project_section)) = std::str::from_utf8(template)
        .ok()
        .and_then(split_project_section)
    else {
        return existing.as_bytes().to_vec();
    };
    format!("{managed}{project_section}\n{existing}").into_bytes()
}
