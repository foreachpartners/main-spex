use std::sync::LazyLock;

use regex::Regex;

use crate::spec::{AcId, CoverageLayer, SpecId};

#[cfg(test)]
#[path = "annotations_test.rs"]
mod annotations_test;

pub struct Annotation {
    pub layer: CoverageLayer,
    pub spec_id: SpecId,
    pub ac_id: Option<AcId>,
}

impl Annotation {
    fn from_token(layer: CoverageLayer, token: &str) -> Option<Self> {
        let (spec_id, ac_id) = match token.split_once('#') {
            Some((spec_id, ac_id)) => (spec_id, Some(ac_id)),
            None => (token, None),
        };
        Some(Self {
            layer,
            spec_id: spec_id.parse().ok()?,
            ac_id: ac_id.map(str::parse::<AcId>).transpose().ok()?,
        })
    }
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

fn candidates(content: &str) -> impl Iterator<Item = Option<Annotation>> {
    ANNOTATION
        .captures_iter(content)
        .filter_map(|caps| {
            let layer = caps[1].parse::<CoverageLayer>().ok()?;
            Some((layer, caps.get(2)?.as_str()))
        })
        .flat_map(|(layer, tokens)| {
            TOKEN
                .find_iter(tokens)
                .map(move |token| Annotation::from_token(layer, token.as_str()))
        })
}

pub fn parse(content: &str) -> Vec<Annotation> {
    candidates(content).flatten().collect()
}

fn is_well_formed(line: &str) -> bool {
    ANNOTATION.is_match(line) && candidates(line).all(|candidate| candidate.is_some())
}

pub fn malformed_lines(content: &str) -> Vec<usize> {
    content
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains("@spec ") && !is_well_formed(line))
        .map(|(index, _)| index + 1)
        .collect()
}
