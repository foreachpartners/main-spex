use super::yaml_dir::YamlFile;

pub fn ai_view(file: &YamlFile<serde_json::Value>, dropped_fields: &[&str]) -> serde_json::Value {
    let Some(object) = file.item.as_object() else {
        return file.item.clone();
    };
    let path = serde_json::Value::String(file.path.to_string_lossy().into_owned());
    let mut view = serde_json::Map::with_capacity(object.len() + 1);
    for (key, value) in object {
        if dropped_fields.contains(&key.as_str()) {
            continue;
        }
        view.insert(key.clone(), value.clone());
        if key == "id" {
            view.insert("path".to_string(), path.clone());
        }
    }
    serde_json::Value::Object(view)
}
