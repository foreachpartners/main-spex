use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde_json::Value;

use crate::error::Result;
use crate::rule::RULES_DIR;
use crate::spec::SPECS_DIR;
use crate::templates::SCHEMAS_DIR;
use crate::util::yaml_files;

use super::{Finding, RULE_ID, rel};

#[cfg(test)]
#[path = "property_order_test.rs"]
mod property_order_test;

pub(super) const PROPERTY_ORDER_CHECK: &str = "property-order";

fn load_schema_file(cache: &mut HashMap<String, Value>, schemas_dir: &Path, name: &str) -> Value {
    if let Some(value) = cache.get(name) {
        return value.clone();
    }
    let value = std::fs::read_to_string(schemas_dir.join(name))
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .unwrap_or(Value::Null);
    cache.insert(name.to_string(), value.clone());
    value
}

// Follows a schema node's `$ref` chain (a bare filename for a sibling schema,
// or `#/...` for a pointer within its own document) until it lands on a
// concrete node, returning that node alongside the document it now belongs to
// (needed so a further internal `$ref` inside it resolves correctly).
fn resolve_ref(
    cache: &mut HashMap<String, Value>,
    schemas_dir: &Path,
    root: &Value,
    node: &Value,
) -> (Value, Value) {
    let mut current = node.clone();
    let mut current_root = root.clone();
    loop {
        let Some(reference) = current.get("$ref").and_then(Value::as_str) else {
            return (current, current_root);
        };
        current = if let Some(pointer) = reference.strip_prefix("#/") {
            current_root
                .pointer(&format!("/{pointer}"))
                .cloned()
                .unwrap_or(Value::Null)
        } else {
            let external = load_schema_file(cache, schemas_dir, reference);
            current_root = external.clone();
            external
        };
    }
}

// The canonical property order for an object schema node: its own declared
// `properties` order, with each `allOf` conditional branch's properties
// spliced in right after the discriminator property its `if` matches on (so
// e.g. a check's `pattern`/`negate` land right after `type`, ahead of the
// base schema's trailing `include`/`exclude`/`severity`). A key already
// placed keeps its first position.
fn ordered_properties(resolved: &Value) -> Vec<String> {
    let mut order: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    if let Some(properties) = resolved.get("properties").and_then(Value::as_object) {
        for key in properties.keys() {
            if seen.insert(key.clone()) {
                order.push(key.clone());
            }
        }
    }

    if let Some(all_of) = resolved.get("allOf").and_then(Value::as_array) {
        for entry in all_of {
            let discriminator = entry
                .get("if")
                .and_then(|node| node.get("properties"))
                .and_then(Value::as_object)
                .and_then(|properties| properties.keys().next())
                .cloned();
            let then_keys: Vec<String> = entry
                .get("then")
                .and_then(|then| then.get("properties"))
                .and_then(Value::as_object)
                .map(|properties| properties.keys().cloned().collect())
                .unwrap_or_default();

            let mut insert_at = discriminator
                .and_then(|key| order.iter().position(|existing| *existing == key))
                .map(|position| position + 1)
                .unwrap_or(order.len());

            for key in then_keys {
                if seen.insert(key.clone()) {
                    order.insert(insert_at, key);
                    insert_at += 1;
                }
            }
        }
    }

    order
}

// Looks up the (still unresolved) schema node for a property, checking the
// schema's own `properties` first, then each `allOf` branch's `then.properties`.
fn property_schema(resolved: &Value, key: &str) -> Option<Value> {
    if let Some(node) = resolved
        .get("properties")
        .and_then(|properties| properties.get(key))
    {
        return Some(node.clone());
    }
    resolved.get("allOf").and_then(Value::as_array).and_then(
        |all_of: &Vec<Value>| -> Option<Value> {
            all_of.iter().find_map(|entry| {
                entry
                    .get("then")
                    .and_then(|then| then.get("properties"))
                    .and_then(|properties| properties.get(key))
                    .cloned()
            })
        },
    )
}

// Finds the first key whose position in `actual` disagrees with `canonical`,
// ignoring keys absent from either side (an unknown key or one the schema
// doesn't declare is somebody else's problem, e.g. `spec-files-conform`).
fn find_order_violation(canonical: &[String], actual: &[String]) -> Option<(String, String)> {
    let filtered_actual: Vec<&String> = actual.iter().filter(|k| canonical.contains(k)).collect();
    let filtered_canonical: Vec<&String> =
        canonical.iter().filter(|k| actual.contains(k)).collect();
    filtered_actual
        .iter()
        .zip(filtered_canonical.iter())
        .find(|(found, expected)| found != expected)
        .map(|(found, expected)| ((*found).clone(), (*expected).clone()))
}

fn property_order_message(
    found: &str,
    expected: &str,
    canonical: &[String],
    label: &str,
) -> String {
    let scope = if label.is_empty() {
        String::new()
    } else {
        format!(" in `{label}`")
    };
    format!(
        "property `{found}` is out of order{scope}: expected `{expected}` at this position (canonical order: {})",
        canonical.join(", ")
    )
}

// Verifies every spec file, rule file, and `spex.yaml` declares its
// properties — at every nesting level their JSON Schema describes — in the
// order the schema declares them.
pub(super) fn validate_property_order(spex_dir: &Path) -> Result<Vec<Finding>> {
    let schemas_dir = spex_dir.join(SCHEMAS_DIR);
    let mut cache: HashMap<String, Value> = HashMap::new();
    let mut findings = Vec::new();

    let spec_schema = load_schema_file(&mut cache, &schemas_dir, "spec.schema.json");
    for file in yaml_files(&spex_dir.join(SPECS_DIR))? {
        check_file_property_order(
            spex_dir,
            &file,
            &spec_schema,
            &mut cache,
            &schemas_dir,
            &mut findings,
        );
    }

    let rule_schema = load_schema_file(&mut cache, &schemas_dir, "rule.schema.json");
    for file in yaml_files(&spex_dir.join(RULES_DIR))? {
        check_file_property_order(
            spex_dir,
            &file,
            &rule_schema,
            &mut cache,
            &schemas_dir,
            &mut findings,
        );
    }

    let spex_yaml = spex_dir.join(crate::config::SPEX_CONFIG_FILE);
    if spex_yaml.is_file() {
        let spex_schema = load_schema_file(&mut cache, &schemas_dir, "spex.schema.json");
        check_file_property_order(
            spex_dir,
            &spex_yaml,
            &spex_schema,
            &mut cache,
            &schemas_dir,
            &mut findings,
        );
    }
    Ok(findings)
}

fn check_file_property_order(
    spex_dir: &Path,
    file: &Path,
    schema: &Value,
    cache: &mut HashMap<String, Value>,
    schemas_dir: &Path,
    findings: &mut Vec<Finding>,
) {
    if schema.is_null() {
        return;
    }
    let Ok(text) = std::fs::read_to_string(file) else {
        return;
    };
    let Ok(value) = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&text) else {
        return;
    };

    check_node_order(
        spex_dir,
        file,
        "",
        &value,
        schema,
        schema,
        cache,
        schemas_dir,
        findings,
    );
}

#[allow(clippy::too_many_arguments)]
fn check_node_order(
    spex_dir: &Path,
    file: &Path,
    label: &str,
    actual: &serde_yaml_ng::Value,
    schema_node: &Value,
    root: &Value,
    cache: &mut HashMap<String, Value>,
    schemas_dir: &Path,
    findings: &mut Vec<Finding>,
) {
    let (resolved, resolved_root) = resolve_ref(cache, schemas_dir, root, schema_node);

    match actual {
        serde_yaml_ng::Value::Mapping(mapping) => {
            let canonical = ordered_properties(&resolved);
            if canonical.is_empty() {
                return;
            }

            let actual_keys: Vec<String> = mapping
                .keys()
                .filter_map(|k| k.as_str().map(str::to_string))
                .collect();
            if let Some((found, expected)) = find_order_violation(&canonical, &actual_keys) {
                findings.push(Finding::error(
                    RULE_ID,
                    PROPERTY_ORDER_CHECK,
                    Some(rel(spex_dir, file)),
                    property_order_message(&found, &expected, &canonical, label),
                ));
            }

            for (key, value) in mapping {
                let Some(key) = key.as_str() else {
                    continue;
                };
                let Some(child_schema) = property_schema(&resolved, key) else {
                    continue;
                };
                let child_label = if label.is_empty() {
                    key.to_string()
                } else {
                    format!("{label}.{key}")
                };
                check_node_order(
                    spex_dir,
                    file,
                    &child_label,
                    value,
                    &child_schema,
                    &resolved_root,
                    cache,
                    schemas_dir,
                    findings,
                );
            }
        }
        serde_yaml_ng::Value::Sequence(items) => {
            let Some(items_schema) = resolved.get("items") else {
                return;
            };
            for (index, item) in items.iter().enumerate() {
                let child_label = format!("{label}[{index}]");
                check_node_order(
                    spex_dir,
                    file,
                    &child_label,
                    item,
                    items_schema,
                    &resolved_root,
                    cache,
                    schemas_dir,
                    findings,
                );
            }
        }
        _ => {}
    }
}
