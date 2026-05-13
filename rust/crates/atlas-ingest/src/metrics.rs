use std::collections::BTreeSet;

use atlas_domain::MetricDomain;
use serde_json::Value;

use crate::normalize::{
    extract_damage_types, extract_disable_skills, extract_sense_types, extract_speed_types,
    normalized_pointer_string, parse_bulk_value, parse_hands_requirement, pointer_bool,
    pointer_string, string_array_at_pointer, strip_markup, typed_collection,
};
use crate::{ActorSideData, ItemSideData, MetricRow, MetricValue, SpellSideData};

pub(super) fn extract_metrics(
    raw: &Value,
    document_type: &str,
    record_type: &str,
) -> Vec<MetricRow> {
    let metrics = match document_type {
        "Actor" => extract_actor_metrics(raw),
        "Item" => extract_item_metrics(raw, record_type),
        _ => Vec::new(),
    };
    dedupe_metrics(metrics)
}

fn dedupe_metrics(metrics: Vec<MetricRow>) -> Vec<MetricRow> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for metric in metrics.into_iter().rev() {
        if seen.insert((metric.domain, metric.key.clone())) {
            deduped.push(metric);
        }
    }
    deduped.reverse();
    deduped
}

pub(super) fn extract_actor_side_data(raw: &Value) -> ActorSideData {
    let disable_text =
        pointer_string(raw, "/system/details/disable").map(|value| strip_markup(&value));
    ActorSideData {
        size: normalized_pointer_string(raw, "/system/traits/size/value"),
        languages: string_array_at_pointer(raw, "/system/details/languages/value"),
        speed_types: extract_speed_types(raw),
        senses: extract_sense_types(raw),
        immunities: typed_collection(raw, "/system/attributes/immunities"),
        resistances: typed_collection(raw, "/system/attributes/resistances"),
        weaknesses: typed_collection(raw, "/system/attributes/weaknesses"),
        disable_text,
        disable_skills: extract_disable_skills(raw),
        is_complex: pointer_bool(raw, "/system/details/isComplex").unwrap_or(false),
    }
}

pub(super) fn extract_item_side_data(
    raw: &Value,
    system_category: Option<String>,
    system_base_item: Option<String>,
    system_group: Option<String>,
    system_usage: Option<String>,
    price_cp: Option<i64>,
) -> ItemSideData {
    ItemSideData {
        system_category,
        system_base_item,
        system_group,
        system_usage: system_usage.clone(),
        price_cp,
        bulk_value: raw.pointer("/system/bulk/value").and_then(parse_bulk_value),
        hands_requirement: system_usage.as_deref().and_then(parse_hands_requirement),
        damage_types: extract_damage_types(raw),
    }
}

pub(super) fn extract_spell_side_data(raw: &Value, traits: &[String]) -> SpellSideData {
    SpellSideData {
        traditions: string_array_at_pointer(raw, "/system/traits/traditions"),
        spell_kinds: ["focus", "ritual", "cantrip"]
            .into_iter()
            .filter(|kind| traits.iter().any(|value| value == kind))
            .map(str::to_string)
            .collect(),
        range_text: normalized_pointer_string(raw, "/system/range/value"),
        range_value: first_number_like_at_paths(
            raw,
            &[
                "/system/range/value",
                "/system/range/increment",
                "/system/area/value",
            ],
        ),
        target_text: pointer_string(raw, "/system/target/value").map(|value| strip_markup(&value)),
        area_type: normalized_pointer_string(raw, "/system/area/type"),
        area_value: number_like_at_pointer(raw, "/system/area/value"),
        save_type: normalized_pointer_string(raw, "/system/defense/save/statistic"),
        sustained: pointer_bool(raw, "/system/duration/sustained").unwrap_or(false),
        basic_save: pointer_bool(raw, "/system/defense/save/basic").unwrap_or(false),
        damage_types: extract_damage_types(raw),
    }
}

fn extract_actor_metrics(raw: &Value) -> Vec<MetricRow> {
    let mut metrics = Vec::new();

    for ability_key in ["str", "dex", "con", "int", "wis", "cha"] {
        add_metric_number(
            &mut metrics,
            MetricDomain::Actor,
            &format!("ability.{ability_key}.mod"),
            first_number_at_paths(
                raw,
                &[
                    &format!("/system/abilities/{ability_key}/mod"),
                    &format!("/system/abilities/{ability_key}/modifier"),
                ],
            ),
        );
    }

    add_metric_number(
        &mut metrics,
        MetricDomain::Actor,
        "perception.mod",
        first_number_at_paths(
            raw,
            &[
                "/system/perception/mod",
                "/system/perception/modifier",
                "/system/perception/value",
            ],
        ),
    );
    add_metric_number(
        &mut metrics,
        MetricDomain::Actor,
        "ac.value",
        number_at_pointer(raw, "/system/attributes/ac/value"),
    );
    add_metric_number(
        &mut metrics,
        MetricDomain::Actor,
        "hardness.value",
        number_at_pointer(raw, "/system/attributes/hardness"),
    );

    for (metric_key, pointer) in [
        ("hp.value", "/system/attributes/hp/value"),
        ("hp.max", "/system/attributes/hp/max"),
        ("hp.bt", "/system/attributes/hp/brokenThreshold"),
        ("hp.bt", "/system/attributes/hp/broken"),
        ("hp.bt", "/system/attributes/hp/bt"),
    ] {
        if metrics.iter().any(|metric| metric.key == metric_key) {
            continue;
        }
        add_metric_number(
            &mut metrics,
            MetricDomain::Actor,
            metric_key,
            number_at_pointer(raw, pointer),
        );
    }

    let save_values = extract_save_metrics(raw, &mut metrics);
    add_best_worst_save_metrics(&mut metrics, &save_values);
    extract_skill_metrics(raw, &mut metrics);
    extract_speed_metrics(raw, &mut metrics);
    extract_sense_metrics(raw, &mut metrics);
    extract_stealth_metrics(raw, &mut metrics);
    metrics
}

fn extract_save_metrics(raw: &Value, metrics: &mut Vec<MetricRow>) -> Vec<(&'static str, f64)> {
    let mut save_values = Vec::new();
    let Some(saves) = raw.pointer("/system/saves").and_then(Value::as_object) else {
        return save_values;
    };

    for (raw_key, value) in saves {
        let Some(save_key) = normalize_save_key(raw_key) else {
            continue;
        };
        let save_value =
            first_number_at_paths(value, &["/mod", "/modifier", "/value", "/totalModifier"]);
        if let Some(number) = save_value {
            add_metric_number(
                metrics,
                MetricDomain::Actor,
                &format!("save.{save_key}.mod"),
                Some(number),
            );
            save_values.push((save_key, number));
        }
    }

    save_values
}

fn add_best_worst_save_metrics(metrics: &mut Vec<MetricRow>, save_values: &[(&'static str, f64)]) {
    let Some((best_save, _)) = save_values
        .iter()
        .max_by(|left, right| left.1.total_cmp(&right.1))
    else {
        return;
    };
    let Some((worst_save, _)) = save_values
        .iter()
        .min_by(|left, right| left.1.total_cmp(&right.1))
    else {
        return;
    };

    metrics.push(MetricRow {
        domain: MetricDomain::Actor,
        key: "save.best".to_string(),
        value: MetricValue::Text((*best_save).to_string()),
    });
    metrics.push(MetricRow {
        domain: MetricDomain::Actor,
        key: "save.worst".to_string(),
        value: MetricValue::Text((*worst_save).to_string()),
    });
}

fn extract_skill_metrics(raw: &Value, metrics: &mut Vec<MetricRow>) {
    let Some(skills) = raw.pointer("/system/skills").and_then(Value::as_object) else {
        return;
    };

    for (raw_key, value) in skills {
        let skill_key = slugify_metric_segment(raw_key);
        if skill_key.is_empty() {
            continue;
        }
        add_metric_number(
            metrics,
            MetricDomain::Actor,
            &format!("skill.{skill_key}.mod"),
            first_number_at_paths(value, &["/mod", "/modifier", "/value"]),
        );
        if let Some(rank) = number_at_pointer(value, "/rank") {
            add_metric_number(
                metrics,
                MetricDomain::Actor,
                &format!("skill.{skill_key}.rank"),
                Some(rank),
            );
            metrics.push(MetricRow {
                domain: MetricDomain::Actor,
                key: format!("skill.{skill_key}.proficient"),
                value: MetricValue::Boolean(rank >= 1.0),
            });
        }
    }
}

fn extract_speed_metrics(raw: &Value, metrics: &mut Vec<MetricRow>) {
    add_metric_number(
        metrics,
        MetricDomain::Actor,
        "speed.land.value",
        number_like_at_pointer(raw, "/system/attributes/speed/value"),
    );

    let Some(other_speeds) = raw
        .pointer("/system/attributes/speed/otherSpeeds")
        .and_then(Value::as_array)
    else {
        return;
    };

    for speed in other_speeds {
        let speed_type = pointer_string(speed, "/type")
            .map(|value| slugify_metric_segment(&value))
            .unwrap_or_default();
        if speed_type.is_empty() {
            continue;
        }
        add_metric_number(
            metrics,
            MetricDomain::Actor,
            &format!("speed.{speed_type}.value"),
            number_like_at_pointer(speed, "/value"),
        );
    }
}

fn extract_sense_metrics(raw: &Value, metrics: &mut Vec<MetricRow>) {
    let Some(senses) = raw
        .pointer("/system/perception/senses")
        .and_then(Value::as_array)
    else {
        return;
    };

    for sense in senses {
        let sense_type = pointer_string(sense, "/type")
            .map(|value| slugify_metric_segment(&value))
            .unwrap_or_default();
        if sense_type.is_empty() {
            continue;
        }
        add_metric_number(
            metrics,
            MetricDomain::Actor,
            &format!("sense.{sense_type}.range"),
            number_like_at_pointer(sense, "/range"),
        );
    }
}

fn extract_stealth_metrics(raw: &Value, metrics: &mut Vec<MetricRow>) {
    let stealth_mod = first_number_at_paths(
        raw,
        &[
            "/system/attributes/stealth/value",
            "/system/attributes/stealth/mod",
            "/system/attributes/stealth/modifier",
        ],
    );
    add_metric_number(metrics, MetricDomain::Actor, "stealth.mod", stealth_mod);
    add_metric_number(
        metrics,
        MetricDomain::Actor,
        "stealth.dc",
        number_at_pointer(raw, "/system/attributes/stealth/dc")
            .or_else(|| stealth_mod.map(|value| value + 10.0)),
    );
}

fn extract_item_metrics(raw: &Value, record_type: &str) -> Vec<MetricRow> {
    let mut metrics = Vec::new();
    match slugify_metric_segment(record_type).as_str() {
        "weapon" => {
            add_metric_number(
                &mut metrics,
                MetricDomain::Item,
                "weapon.range_increment",
                first_number_like_at_paths(
                    raw,
                    &[
                        "/system/range/increment",
                        "/system/range/value",
                        "/system/range",
                    ],
                ),
            );
            add_metric_number(
                &mut metrics,
                MetricDomain::Item,
                "weapon.reload",
                first_number_like_at_paths(raw, &["/system/reload/value", "/system/reload"]),
            );
            add_metric_number(
                &mut metrics,
                MetricDomain::Item,
                "weapon.damage_dice",
                number_at_pointer(raw, "/system/damage/dice"),
            );
            add_metric_number(
                &mut metrics,
                MetricDomain::Item,
                "weapon.damage_die_faces",
                damage_die_faces(raw.pointer("/system/damage/die")),
            );
        }
        "armor" => {
            for (metric_key, pointer) in [
                ("armor.ac_bonus", "/system/acBonus"),
                ("armor.dex_cap", "/system/dexCap"),
                ("armor.strength", "/system/strength"),
                ("armor.check_penalty", "/system/checkPenalty"),
                ("armor.speed_penalty", "/system/speedPenalty"),
            ] {
                add_metric_number(
                    &mut metrics,
                    MetricDomain::Item,
                    metric_key,
                    number_at_pointer(raw, pointer),
                );
            }
        }
        "shield" => {
            for (metric_key, pointer) in [
                ("shield.ac_bonus", "/system/acBonus"),
                ("shield.hardness", "/system/hardness"),
                ("shield.hp", "/system/hp/value"),
                ("shield.hp", "/system/hp/max"),
                ("shield.bt", "/system/hp/brokenThreshold"),
                ("shield.bt", "/system/hp/broken"),
                ("shield.bt", "/system/hp/bt"),
            ] {
                if metrics.iter().any(|metric| metric.key == metric_key) {
                    continue;
                }
                add_metric_number(
                    &mut metrics,
                    MetricDomain::Item,
                    metric_key,
                    number_at_pointer(raw, pointer),
                );
            }
        }
        _ => {}
    }
    metrics
}

fn add_metric_number(
    metrics: &mut Vec<MetricRow>,
    domain: MetricDomain,
    key: &str,
    value: Option<f64>,
) {
    let Some(value) = value.filter(|value| value.is_finite()) else {
        return;
    };
    metrics.push(MetricRow {
        domain,
        key: key.to_string(),
        value: MetricValue::Number(value),
    });
}

fn first_number_at_paths(raw: &Value, pointers: &[&str]) -> Option<f64> {
    pointers
        .iter()
        .find_map(|pointer| number_at_pointer(raw, pointer))
}

fn first_number_like_at_paths(raw: &Value, pointers: &[&str]) -> Option<f64> {
    pointers
        .iter()
        .find_map(|pointer| number_like_at_pointer(raw, pointer))
}

fn number_at_pointer(raw: &Value, pointer: &str) -> Option<f64> {
    raw.pointer(pointer).and_then(value_as_f64)
}

fn number_like_at_pointer(raw: &Value, pointer: &str) -> Option<f64> {
    raw.pointer(pointer).and_then(parse_numeric_like_value)
}

fn value_as_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.trim().parse::<f64>().ok(),
        _ => None,
    }
}

fn parse_numeric_like_value(value: &Value) -> Option<f64> {
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

fn damage_die_faces(value: Option<&Value>) -> Option<f64> {
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

fn normalize_save_key(value: &str) -> Option<&'static str> {
    match slugify_metric_segment(value).as_str() {
        "fort" | "fortitude" => Some("fort"),
        "ref" | "reflex" => Some("ref"),
        "will" => Some("will"),
        _ => None,
    }
}

pub(super) fn slugify_metric_segment(value: &str) -> String {
    let mut output = String::new();
    let mut last_was_separator = false;
    for character in value.trim().to_lowercase().chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character);
            last_was_separator = false;
        } else if !last_was_separator && !output.is_empty() {
            output.push('_');
            last_was_separator = true;
        }
    }
    while output.ends_with('_') {
        output.pop();
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugifies_metric_segments_to_stable_keys() {
        assert_eq!(slugify_metric_segment("Fortitude Save"), "fortitude_save");
        assert_eq!(slugify_metric_segment("  Land-Speed! "), "land_speed");
    }

    #[test]
    fn parses_first_numeric_prefix_from_text_values() {
        assert_eq!(
            parse_numeric_like_value(&Value::String("30 feet".to_string())),
            Some(30.0)
        );
        assert_eq!(
            parse_numeric_like_value(&Value::String("-5 penalty".to_string())),
            Some(-5.0)
        );
    }

    #[test]
    fn dedupes_metrics_by_domain_and_key_with_later_values_winning() {
        let metrics = dedupe_metrics(vec![
            MetricRow {
                domain: MetricDomain::Actor,
                key: "hp.value".to_string(),
                value: MetricValue::Number(5.0),
            },
            MetricRow {
                domain: MetricDomain::Actor,
                key: "hp.value".to_string(),
                value: MetricValue::Number(9.0),
            },
        ]);

        assert_eq!(
            metrics,
            vec![MetricRow {
                domain: MetricDomain::Actor,
                key: "hp.value".to_string(),
                value: MetricValue::Number(9.0),
            }]
        );
    }
}
