use serde_json::Value;

pub(super) fn number_at_pointer(raw: &Value, pointer: &str) -> Option<f64> {
    raw.pointer(pointer).and_then(value_as_f64)
}

pub(crate) fn number_like_at_pointer(raw: &Value, pointer: &str) -> Option<f64> {
    raw.pointer(pointer).and_then(number_like_value)
}

fn value_as_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.trim().parse::<f64>().ok(),
        _ => None,
    }
}

pub(super) fn number_like_value(value: &Value) -> Option<f64> {
    if let Some(number) = value_as_f64(value) {
        return Some(number);
    }
    let Value::String(text) = value else {
        return None;
    };
    let mut buffer = String::new();
    for character in text.chars() {
        if character.is_ascii_digit() || character == '.' || character == '-' {
            buffer.push(character);
        } else if !buffer.is_empty() {
            break;
        }
    }
    buffer.parse::<f64>().ok()
}

pub(super) fn damage_die_faces(value: Option<&Value>) -> Option<f64> {
    match value? {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text
            .trim()
            .strip_prefix('d')
            .or_else(|| text.trim().strip_prefix('D'))
            .and_then(|faces| faces.parse::<f64>().ok()),
        _ => None,
    }
}

pub(crate) fn slugify_metric_segment(value: &str) -> String {
    atlas_record::metrics::normalize_metric_key_segment(value)
}
