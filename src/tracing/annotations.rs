use std::sync::LazyLock;

use regex::Regex;

use crate::spec::CoverageLayer;

#[cfg(test)]
#[path = "annotations_test.rs"]
mod annotations_test;

pub struct Annotation {
    pub layer: CoverageLayer,
    pub spec_id: String,
    pub ac_id: Option<String>,
}

fn token_pattern() -> String {
    let id = r"[a-z0-9]+(?:-[a-z0-9]+)*";
    format!(r"{id}(?:#{id})?")
}

static ANNOTATION: LazyLock<Regex> = LazyLock::new(|| {
    let token = token_pattern();
    Regex::new(&format!(
        r"@spec\s+(contract|impl|test)\s+({token}(?:\s*,\s*{token})*)"
    ))
    .expect("valid @spec regex")
});

static TOKEN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&token_pattern()).expect("valid token regex"));

pub fn parse(content: &str) -> Vec<Annotation> {
    let mut annotations = Vec::new();
    for caps in ANNOTATION.captures_iter(content) {
        let Ok(layer) = caps[1].parse::<CoverageLayer>() else {
            continue;
        };

        for token in TOKEN.find_iter(&caps[2]) {
            let (spec_id, ac_id) = match token.as_str().split_once('#') {
                Some((spec_id, ac_id)) => (spec_id, Some(ac_id)),
                None => (token.as_str(), None),
            };
            annotations.push(Annotation {
                layer,
                spec_id: spec_id.to_string(),
                ac_id: ac_id.map(str::to_string),
            });
        }
    }
    annotations
}

pub fn malformed_lines(content: &str) -> Vec<usize> {
    content
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains("@spec ") && !ANNOTATION.is_match(line))
        .map(|(index, _)| index + 1)
        .collect()
}
