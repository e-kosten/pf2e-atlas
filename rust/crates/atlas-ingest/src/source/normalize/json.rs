use serde_json::Value;

use crate::source::normalize::normalize_text;

pub(crate) fn string_field(raw: &Value, key: &str) -> Option<String> {
    raw.get(key)?.as_str().map(str::to_string)
}

pub(crate) fn pointer_string(raw: &Value, pointer: &str) -> Option<String> {
    raw.pointer(pointer)?.as_str().map(str::to_string)
}

pub(crate) fn normalized_pointer_string(raw: &Value, pointer: &str) -> Option<String> {
    pointer_string(raw, pointer).and_then(|value| {
        let normalized = value.trim();
        if normalized.is_empty() {
            None
        } else {
            Some(normalized.to_string())
        }
    })
}

pub(crate) fn pointer_bool(raw: &Value, pointer: &str) -> Option<bool> {
    raw.pointer(pointer)?.as_bool()
}

pub(crate) fn pointer_i64(raw: &Value, pointer: &str) -> Option<i64> {
    raw.pointer(pointer)?.as_i64()
}

pub(crate) fn string_array_at_pointer(raw: &Value, pointer: &str) -> Vec<String> {
    let mut values = raw
        .pointer(pointer)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(normalize_text)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

pub(crate) fn typed_collection(raw: &Value, pointer: &str) -> Vec<String> {
    let mut values = raw
        .pointer(pointer)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| normalized_pointer_string(entry, "/type"))
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}
