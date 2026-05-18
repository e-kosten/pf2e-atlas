use serde_json::Value;

use crate::records::metrics;

use super::{normalize_text, normalized_pointer_string, pointer_string};

pub(crate) fn normalize_price_cp(value: Option<&Value>) -> Option<i64> {
    let object = value?.as_object()?;
    let platinum = object.get("pp").and_then(Value::as_i64).unwrap_or(0);
    let gold = object.get("gp").and_then(Value::as_i64).unwrap_or(0);
    let silver = object.get("sp").and_then(Value::as_i64).unwrap_or(0);
    let copper = object.get("cp").and_then(Value::as_i64).unwrap_or(0);
    let total = platinum * 1000 + gold * 100 + silver * 10 + copper;
    (total > 0).then_some(total)
}

pub(crate) fn extract_speed_types(raw: &Value) -> Vec<String> {
    let mut values = vec!["land".to_string()];
    if let Some(other_speeds) = raw
        .pointer("/system/attributes/speed/otherSpeeds")
        .and_then(Value::as_array)
    {
        values.extend(
            other_speeds
                .iter()
                .filter_map(|speed| normalized_pointer_string(speed, "/type")),
        );
    }
    values.sort();
    values.dedup();
    values
}

pub(crate) fn extract_sense_types(raw: &Value) -> Vec<String> {
    let mut values = raw
        .pointer("/system/perception/senses")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|sense| normalized_pointer_string(sense, "/type"))
        .map(|value| metrics::slugify_metric_segment(&value).replace('_', " "))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

pub(crate) fn parse_bulk_value(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) if text == "L" => Some(0.1),
        Value::String(text) => text.trim().parse::<f64>().ok(),
        _ => None,
    }
}

pub(crate) fn parse_hands_requirement(usage: &str) -> Option<String> {
    if usage.contains("held-in-two-hands") {
        Some("two_hands".to_string())
    } else if usage.contains("held-in-one-plus-hands") {
        Some("one_plus_hands".to_string())
    } else if usage.contains("held-in-one-hand") {
        Some("one_hand".to_string())
    } else {
        None
    }
}

pub(crate) fn extract_damage_types(raw: &Value) -> Vec<String> {
    let mut values = Vec::new();
    if let Some(value) = normalized_pointer_string(raw, "/system/damage/damageType") {
        values.push(value);
    }
    if let Some(entries) = raw
        .pointer("/system/damageRolls")
        .and_then(Value::as_object)
    {
        values.extend(
            entries
                .values()
                .filter_map(|entry| normalized_pointer_string(entry, "/damageType")),
        );
    }
    if let Some(entries) = raw.pointer("/system/damage").and_then(Value::as_object) {
        values.extend(
            entries
                .values()
                .filter_map(|entry| normalized_pointer_string(entry, "/type")),
        );
    }
    values.sort();
    values.dedup();
    values
}

pub(crate) fn extract_disable_skills(raw: &Value) -> Vec<String> {
    let Some(markup) = pointer_string(raw, "/system/details/disable") else {
        return Vec::new();
    };
    let mut skills = Vec::new();
    for skill in [
        "acrobatics",
        "arcana",
        "athletics",
        "crafting",
        "deception",
        "diplomacy",
        "intimidation",
        "medicine",
        "nature",
        "occultism",
        "performance",
        "religion",
        "society",
        "stealth",
        "survival",
        "thievery",
    ] {
        if markup.to_lowercase().contains(skill) {
            skills.push(skill.to_string());
        }
    }
    skills.sort();
    skills.dedup();
    skills
}

pub(crate) fn extract_traits(raw: &Value) -> Vec<String> {
    let mut traits = raw
        .pointer("/system/traits/value")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(normalize_text)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    traits.sort();
    traits.dedup();
    traits
}

pub(crate) fn extract_prerequisites(raw: &Value) -> Vec<String> {
    raw.pointer("/system/prerequisites/value")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| pointer_string(entry, "/value"))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect()
}
