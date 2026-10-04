use std::collections::HashMap;

use ast_grep_config::{DeserializeEnv, RuleCoreError, SerializableRuleCore};
use ast_grep_language::{Language, LanguageExt, SupportLang};
use serde_yaml_ng::{Mapping, Value};

use super::super::*;

pub(in crate::commands::check) fn run_ast_grep(
    spec_id: &str,
    check: &Check,
    files: &[&ScopedFile],
    findings: &mut Vec<Finding>,
) {
    let CheckKind::AstGrep {
        pattern,
        rule,
        language,
        negate,
    } = &check.kind
    else {
        unreachable!("run_ast_grep called with non-ast-grep check")
    };

    let rule_core = match ast_grep_rule_core_value(pattern, rule) {
        Ok(value) => value,
        Err(message) => {
            findings.push(Finding::config_error(spec_id, &check.name, message));
            return;
        }
    };

    let explicit_lang = match language {
        Some(name) => match name.parse::<SupportLang>() {
            Ok(lang) => Some(lang),
            Err(err) => {
                findings.push(Finding::config_error(
                    spec_id,
                    &check.name,
                    format!("unknown ast-grep language `{name}`: {err}"),
                ));
                return;
            }
        },
        None => None,
    };

    let mut by_lang: HashMap<SupportLang, Vec<&&ScopedFile>> = HashMap::new();
    for file in files {
        let Some(lang) = explicit_lang.or_else(|| SupportLang::from_path(&file.rel)) else {
            continue;
        };
        by_lang.entry(lang).or_default().push(file);
    }

    for (lang, files) in by_lang {
        let matcher = match build_ast_grep_matcher(&rule_core, lang) {
            Ok(matcher) => matcher,
            Err(err) => {
                findings.push(Finding::config_error(
                    spec_id,
                    &check.name,
                    format!("invalid ast-grep rule: {err}"),
                ));
                return;
            }
        };

        for file in files {
            let Ok(content) = std::fs::read_to_string(&file.abs) else {
                continue;
            };
            let tree = lang.ast_grep(&content);
            let root = tree.root();
            let mut matches = root.find_all(&matcher).peekable();

            if *negate {
                if matches.peek().is_none() {
                    findings.push(Finding::violation(
                        check,
                        spec_id,
                        Some(file.rel_str()),
                        check.message("required pattern not found"),
                    ));
                }
            } else {
                for found in matches {
                    let line = found.start_pos().line() + 1;
                    findings.push(Finding::violation(
                        check,
                        spec_id,
                        Some(format!("{}:{}", file.rel_str(), line)),
                        check.message("matches forbidden pattern"),
                    ));
                }
            }
        }
    }
}

// Builds the `SerializableRuleCore` YAML value (`{rule: ..., constraints: ...}`)
// backing an ast-grep check: either the check's own `rule` object as-is, or a
// simple `{rule: {pattern: <pattern>}}` wrapper around `pattern`, mirroring how
// regexp/glob checks reuse the same `pattern` field.
fn ast_grep_rule_core_value(
    pattern: &Option<String>,
    rule: &Option<Value>,
) -> core::result::Result<Value, String> {
    if let Some(rule) = rule {
        return Ok(rule.clone());
    }
    let Some(pattern) = pattern else {
        return Err("ast-grep check requires `pattern` or `rule`".to_string());
    };
    let mut inner = Mapping::new();
    inner.insert(Value::from("pattern"), Value::from(pattern.clone()));
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
