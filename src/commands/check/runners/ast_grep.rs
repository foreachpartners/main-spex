use std::collections::HashMap;

use ast_grep_config::{DeserializeEnv, RuleCore, RuleCoreError, SerializableRuleCore};
use ast_grep_language::{Language, LanguageExt, SupportLang};
use serde_yaml_ng::{Mapping, Value};

use crate::rule::Check;

use super::super::finding::Location;
use super::super::scoped_file::ScopedFile;
use super::{Outcome, PATTERN_WORDING, presence_hits};

pub(in crate::commands::check) fn run_ast_grep(
    check: &Check,
    pattern: Option<&str>,
    rule: Option<&Value>,
    language: Option<&str>,
    negate: bool,
    files: &[&ScopedFile],
) -> Outcome {
    let rule_core = ast_grep_rule_core_value(pattern, rule)?;

    let explicit_lang = language
        .map(|name| {
            name.parse::<SupportLang>()
                .map_err(|err| format!("unknown ast-grep language `{name}`: {err}"))
        })
        .transpose()?;

    let mut matchers: HashMap<SupportLang, RuleCore> = HashMap::new();
    let mut hits = Vec::new();
    for file in files {
        let Some(lang) = explicit_lang.or_else(|| SupportLang::from_path(&file.rel)) else {
            continue;
        };
        if !matchers.contains_key(&lang) {
            let matcher = build_ast_grep_matcher(&rule_core, lang)
                .map_err(|err| format!("invalid ast-grep rule: {err}"))?;
            matchers.insert(lang, matcher);
        }
        let matcher = &matchers[&lang];

        let Ok(content) = std::fs::read_to_string(&file.abs) else {
            continue;
        };
        let tree = lang.ast_grep(&content);
        let found = tree
            .root()
            .find_all(matcher)
            .map(|found| Location::at_line(&file.rel, found.start_pos().line() + 1))
            .collect();
        hits.extend(presence_hits(
            check,
            negate,
            found,
            Some(Location::in_file(&file.rel)),
            &PATTERN_WORDING,
        ));
    }
    Ok(hits)
}

fn ast_grep_rule_core_value(
    pattern: Option<&str>,
    rule: Option<&Value>,
) -> core::result::Result<Value, String> {
    if let Some(rule) = rule {
        return Ok(rule.clone());
    }
    let Some(pattern) = pattern else {
        return Err("ast-grep check requires `pattern` or `rule`".to_string());
    };
    let mut inner = Mapping::new();
    inner.insert(Value::from("pattern"), Value::from(pattern));
    let mut outer = Mapping::new();
    outer.insert(Value::from("rule"), Value::Mapping(inner));
    Ok(Value::Mapping(outer))
}

fn build_ast_grep_matcher(
    rule_core: &Value,
    lang: SupportLang,
) -> core::result::Result<ast_grep_config::RuleCore, String> {
    let json = serde_json::to_value(rule_core).map_err(|err| err.to_string())?;
    let core: SerializableRuleCore = serde_json::from_value(json).map_err(|err| err.to_string())?;
    core.get_matcher(DeserializeEnv::new(lang))
        .map_err(|err: RuleCoreError| err.to_string())
}
