use std::collections::BTreeSet;

use atlas_domain::MetricDomain;
use atlas_record::metrics as metric_definitions;
use serde_json::Value;

use crate::records::{MetricRow, MetricValue};
use crate::source::normalize::pointer_string;

pub(crate) fn extract_metrics(
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

fn extract_actor_metrics(raw: &Value) -> Vec<MetricRow> {
    let mut metrics = Vec::new();

    for ability_key in ["str", "dex", "con", "int", "wis", "cha"] {
        add_metric_number(
            &mut metrics,
            MetricDomain::Actor,
            &metric_definitions::actor::ability::mod_key(ability_key),
            first_number_at_paths(
                raw,
                &[
                    &format!("/system/abilities/{ability_key}/mod"),
                    &format!("/system/abilities/{ability_key}/modifier"),
                ],
            ),
        );
    }

    add_defined_metric_number(
        &mut metrics,
        metric_definitions::actor::PERCEPTION_MOD,
        first_number_at_paths(
            raw,
            &[
                "/system/perception/mod",
                "/system/perception/modifier",
                "/system/perception/value",
            ],
        ),
    );
    add_defined_metric_number(
        &mut metrics,
        metric_definitions::actor::ARMOR_CLASS,
        number_at_pointer(raw, "/system/attributes/ac/value"),
    );
    add_defined_metric_number(
        &mut metrics,
        metric_definitions::actor::HARDNESS,
        number_at_pointer(raw, "/system/attributes/hardness"),
    );

    for (definition, pointer) in [
        (
            metric_definitions::actor::HP_VALUE,
            "/system/attributes/hp/value",
        ),
        (
            metric_definitions::actor::HP_MAX,
            "/system/attributes/hp/max",
        ),
        (
            metric_definitions::actor::HP_BROKEN_THRESHOLD,
            "/system/attributes/hp/brokenThreshold",
        ),
        (
            metric_definitions::actor::HP_BROKEN_THRESHOLD,
            "/system/attributes/hp/broken",
        ),
        (
            metric_definitions::actor::HP_BROKEN_THRESHOLD,
            "/system/attributes/hp/bt",
        ),
    ] {
        let metric_key = exact_metric_key(definition);
        if metrics.iter().any(|metric| metric.key == metric_key) {
            continue;
        }
        add_defined_metric_number(&mut metrics, definition, number_at_pointer(raw, pointer));
    }

    let save_values = extract_save_metrics(raw, &mut metrics);
    add_best_worst_save_metrics(&mut metrics, &save_values);
    extract_skill_metrics(raw, &mut metrics);
    extract_speed_metrics(raw, &mut metrics);
    extract_sense_metrics(raw, &mut metrics);
    extract_stealth_metrics(raw, &mut metrics);
    extract_disable_metrics(raw, &mut metrics);
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
                &metric_definitions::actor::save::mod_key(save_key),
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
        key: exact_metric_key(metric_definitions::actor::save::BEST).to_string(),
        value: MetricValue::Text((*best_save).to_string()),
    });
    metrics.push(MetricRow {
        domain: MetricDomain::Actor,
        key: exact_metric_key(metric_definitions::actor::save::WORST).to_string(),
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
            &metric_definitions::actor::skill::mod_key(&skill_key),
            first_number_at_paths(value, &["/mod", "/modifier", "/value"]),
        );
        if let Some(rank) = number_at_pointer(value, "/rank") {
            add_metric_number(
                metrics,
                MetricDomain::Actor,
                &metric_definitions::actor::skill::rank_key(&skill_key),
                Some(rank),
            );
            metrics.push(MetricRow {
                domain: MetricDomain::Actor,
                key: metric_definitions::actor::skill::proficient_key(&skill_key),
                value: MetricValue::Boolean(rank >= 1.0),
            });
        }
    }
}

fn extract_speed_metrics(raw: &Value, metrics: &mut Vec<MetricRow>) {
    add_metric_number(
        metrics,
        MetricDomain::Actor,
        &metric_definitions::actor::speed::value_key("land"),
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
            &metric_definitions::actor::speed::value_key(&speed_type),
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
            &metric_definitions::actor::sense::range_key(&sense_type),
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
    add_defined_metric_number(metrics, metric_definitions::actor::STEALTH_MOD, stealth_mod);
    add_defined_metric_number(
        metrics,
        metric_definitions::actor::STEALTH_DC,
        number_at_pointer(raw, "/system/attributes/stealth/dc")
            .or_else(|| stealth_mod.map(|value| value + 10.0)),
    );
}

fn extract_disable_metrics(raw: &Value, metrics: &mut Vec<MetricRow>) {
    let Some(disable_markup) = pointer_string(raw, "/system/details/disable") else {
        return;
    };
    let checks = parse_disable_checks(&disable_markup);
    if checks.is_empty() {
        return;
    }

    let all_dcs = checks
        .iter()
        .filter_map(|check| check.dc)
        .collect::<Vec<_>>();
    if let Some((min, max)) = min_max(&all_dcs) {
        add_defined_metric_number(
            metrics,
            metric_definitions::actor::disable::DC_MIN,
            Some(min),
        );
        add_defined_metric_number(
            metrics,
            metric_definitions::actor::disable::DC_MAX,
            Some(max),
        );
    }

    for skill in HAZARD_DISABLE_SKILLS {
        let skill_key = slugify_metric_segment(skill);
        let skill_dcs = checks
            .iter()
            .filter(|check| check.skills.iter().any(|candidate| candidate == &skill_key))
            .filter_map(|check| check.dc)
            .collect::<Vec<_>>();
        if let Some((min, max)) = min_max(&skill_dcs) {
            add_metric_number(
                metrics,
                MetricDomain::Actor,
                &metric_definitions::actor::disable::skill_dc_min_key(&skill_key),
                Some(min),
            );
            add_metric_number(
                metrics,
                MetricDomain::Actor,
                &metric_definitions::actor::disable::skill_dc_max_key(&skill_key),
                Some(max),
            );
        }
        if let Some(rank) = checks
            .iter()
            .filter(|check| check.skills.iter().any(|candidate| candidate == &skill_key))
            .filter_map(|check| check.rank_min)
            .max()
        {
            add_metric_number(
                metrics,
                MetricDomain::Actor,
                &metric_definitions::actor::disable::skill_rank_min_key(&skill_key),
                Some(f64::from(rank)),
            );
        }
    }
}

pub(crate) fn disable_metric_candidate_keys(markup: &str) -> Vec<String> {
    let checks = parse_disable_checks(markup);
    if checks.is_empty() {
        return Vec::new();
    }

    let mut keys = Vec::new();
    if checks.iter().any(|check| check.dc.is_some()) {
        keys.push(exact_metric_key(metric_definitions::actor::disable::DC_MIN).to_string());
        keys.push(exact_metric_key(metric_definitions::actor::disable::DC_MAX).to_string());
    }

    for skill in HAZARD_DISABLE_SKILLS {
        let skill_key = slugify_metric_segment(skill);
        if checks.iter().any(|check| {
            check.dc.is_some() && check.skills.iter().any(|candidate| candidate == &skill_key)
        }) {
            keys.push(metric_definitions::actor::disable::skill_dc_min_key(
                &skill_key,
            ));
            keys.push(metric_definitions::actor::disable::skill_dc_max_key(
                &skill_key,
            ));
        }
        if checks.iter().any(|check| {
            check.rank_min.is_some() && check.skills.iter().any(|candidate| candidate == &skill_key)
        }) {
            keys.push(metric_definitions::actor::disable::skill_rank_min_key(
                &skill_key,
            ));
        }
    }
    keys
}

#[derive(Debug, PartialEq)]
struct DisableCheck {
    dc: Option<f64>,
    skills: Vec<String>,
    rank_min: Option<u8>,
}

const HAZARD_DISABLE_SKILLS: &[&str] = &[
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
];

fn parse_disable_checks(markup: &str) -> Vec<DisableCheck> {
    let mut checks = Vec::new();
    let mut cursor = 0;
    while let Some(relative_start) = markup[cursor..].find("@Check[") {
        let body_start = cursor + relative_start + "@Check[".len();
        let Some(relative_end) = markup[body_start..].find(']') else {
            break;
        };
        let body_end = body_start + relative_end;
        let trailing_start = body_end + 1;
        let trailing_end = markup[trailing_start..]
            .find("@Check[")
            .map(|next| trailing_start + next)
            .unwrap_or(markup.len());
        checks.push(parse_disable_check(
            &markup[body_start..body_end],
            &markup[trailing_start..trailing_end],
        ));
        cursor = trailing_end;
    }
    checks
}

fn parse_disable_check(body: &str, trailing_markup: &str) -> DisableCheck {
    let segments = body
        .split('|')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    let dc = segments
        .iter()
        .find_map(|segment| parse_prefixed_dc_segment(segment));
    let primary_skill = segments
        .iter()
        .find(|segment| !segment.contains(':'))
        .map(|segment| slugify_metric_segment(segment))
        .filter(|segment| !segment.is_empty());
    let trailing_text = strip_markup(trailing_markup);
    let mut skills = Vec::new();
    if let Some(skill) = primary_skill {
        skills.push(skill);
    }
    for skill in HAZARD_DISABLE_SKILLS {
        if contains_word(&trailing_text, skill) {
            skills.push(slugify_metric_segment(skill));
        }
    }
    skills.sort();
    skills.dedup();

    DisableCheck {
        dc,
        rank_min: proficiency_rank_in_text(&trailing_text),
        skills,
    }
}

fn parse_prefixed_dc_segment(segment: &str) -> Option<f64> {
    let (prefix, value) = segment.split_once(':')?;
    if prefix.trim().eq_ignore_ascii_case("dc") {
        parse_numeric_prefix(value)
    } else {
        None
    }
}

fn parse_numeric_prefix(text: &str) -> Option<f64> {
    let mut buffer = String::new();
    for character in text.trim().chars() {
        if character.is_ascii_digit() || character == '.' || character == '-' {
            buffer.push(character);
        } else if !buffer.is_empty() {
            break;
        }
    }
    buffer.parse::<f64>().ok()
}

fn strip_markup(markup: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    for character in markup.chars() {
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(character),
            _ => {}
        }
    }
    output
}

fn proficiency_rank_in_text(text: &str) -> Option<u8> {
    [
        ("legendary", 4),
        ("master", 3),
        ("expert", 2),
        ("trained", 1),
    ]
    .into_iter()
    .find_map(|(rank, value)| contains_word(text, rank).then_some(value))
}

fn contains_word(text: &str, word: &str) -> bool {
    let lower_text = text.to_lowercase();
    lower_text
        .split(|character: char| !character.is_ascii_alphanumeric())
        .any(|segment| segment == word)
}

fn min_max(values: &[f64]) -> Option<(f64, f64)> {
    let mut iter = values.iter().copied();
    let first = iter.next()?;
    Some(iter.fold((first, first), |(min, max), value| {
        (min.min(value), max.max(value))
    }))
}

fn extract_item_metrics(raw: &Value, record_type: &str) -> Vec<MetricRow> {
    let mut metrics = Vec::new();
    match slugify_metric_segment(record_type).as_str() {
        "weapon" => {
            add_defined_metric_number(
                &mut metrics,
                metric_definitions::item::weapon::RANGE_INCREMENT,
                first_number_like_at_paths(
                    raw,
                    &[
                        "/system/range/increment",
                        "/system/range/value",
                        "/system/range",
                    ],
                ),
            );
            add_defined_metric_number(
                &mut metrics,
                metric_definitions::item::weapon::RELOAD,
                first_number_like_at_paths(raw, &["/system/reload/value", "/system/reload"]),
            );
            add_defined_metric_number(
                &mut metrics,
                metric_definitions::item::weapon::DAMAGE_DICE,
                number_at_pointer(raw, "/system/damage/dice"),
            );
            add_defined_metric_number(
                &mut metrics,
                metric_definitions::item::weapon::DAMAGE_DIE_FACES,
                damage_die_faces(raw.pointer("/system/damage/die")),
            );
        }
        "armor" => {
            for (definition, pointer) in [
                (metric_definitions::item::armor::AC_BONUS, "/system/acBonus"),
                (metric_definitions::item::armor::DEX_CAP, "/system/dexCap"),
                (
                    metric_definitions::item::armor::STRENGTH,
                    "/system/strength",
                ),
                (
                    metric_definitions::item::armor::CHECK_PENALTY,
                    "/system/checkPenalty",
                ),
                (
                    metric_definitions::item::armor::SPEED_PENALTY,
                    "/system/speedPenalty",
                ),
            ] {
                add_defined_metric_number(
                    &mut metrics,
                    definition,
                    number_at_pointer(raw, pointer),
                );
            }
        }
        "shield" => {
            for (definition, pointer) in [
                (
                    metric_definitions::item::shield::AC_BONUS,
                    "/system/acBonus",
                ),
                (
                    metric_definitions::item::shield::HARDNESS,
                    "/system/hardness",
                ),
                (metric_definitions::item::shield::HP, "/system/hp/value"),
                (metric_definitions::item::shield::HP, "/system/hp/max"),
                (
                    metric_definitions::item::shield::BROKEN_THRESHOLD,
                    "/system/hp/brokenThreshold",
                ),
                (
                    metric_definitions::item::shield::BROKEN_THRESHOLD,
                    "/system/hp/broken",
                ),
                (
                    metric_definitions::item::shield::BROKEN_THRESHOLD,
                    "/system/hp/bt",
                ),
            ] {
                let metric_key = exact_metric_key(definition);
                if metrics.iter().any(|metric| metric.key == metric_key) {
                    continue;
                }
                add_defined_metric_number(
                    &mut metrics,
                    definition,
                    number_at_pointer(raw, pointer),
                );
            }
        }
        _ => {}
    }
    metrics
}

fn add_defined_metric_number(
    metrics: &mut Vec<MetricRow>,
    definition: metric_definitions::MetricDefinition,
    value: Option<f64>,
) {
    add_metric_number(
        metrics,
        definition.domain(),
        exact_metric_key(definition),
        value,
    );
}

fn exact_metric_key(definition: metric_definitions::MetricDefinition) -> &'static str {
    definition
        .exact_key()
        .expect("static metric definition should have an exact key")
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

pub(crate) fn first_number_like_at_paths(raw: &Value, pointers: &[&str]) -> Option<f64> {
    pointers
        .iter()
        .find_map(|pointer| number_like_at_pointer(raw, pointer))
}

fn number_at_pointer(raw: &Value, pointer: &str) -> Option<f64> {
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

pub(crate) fn number_like_value(value: &Value) -> Option<f64> {
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

pub(crate) fn damage_die_faces(value: Option<&Value>) -> Option<f64> {
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

pub(crate) fn slugify_metric_segment(value: &str) -> String {
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
            number_like_value(&Value::String("30 feet".to_string())),
            Some(30.0)
        );
        assert_eq!(
            number_like_value(&Value::String("-5 penalty".to_string())),
            Some(-5.0)
        );
    }

    #[test]
    fn dedupes_metrics_by_domain_and_key_with_later_values_winning() {
        let metrics = dedupe_metrics(vec![
            MetricRow {
                domain: MetricDomain::Actor,
                key: exact_metric_key(metric_definitions::actor::HP_VALUE).to_string(),
                value: MetricValue::Number(5.0),
            },
            MetricRow {
                domain: MetricDomain::Actor,
                key: exact_metric_key(metric_definitions::actor::HP_VALUE).to_string(),
                value: MetricValue::Number(9.0),
            },
        ]);

        assert_eq!(
            metrics,
            vec![MetricRow {
                domain: MetricDomain::Actor,
                key: exact_metric_key(metric_definitions::actor::HP_VALUE).to_string(),
                value: MetricValue::Number(9.0),
            }]
        );
    }

    #[test]
    fn extracts_disable_dc_and_rank_metrics_from_hazard_checks() {
        let raw = serde_json::json!({
            "system": {
                "details": {
                    "disable": "@Check[thievery|dc:27] (expert) to disable the lock @Check[crafting|dc:30] or Thievery (master) to jam the gears"
                }
            }
        });

        let metrics = extract_actor_metrics(&raw);

        assert_number_metric(
            &metrics,
            exact_metric_key(metric_definitions::actor::disable::DC_MIN),
            27.0,
        );
        assert_number_metric(
            &metrics,
            exact_metric_key(metric_definitions::actor::disable::DC_MAX),
            30.0,
        );
        assert_number_metric(
            &metrics,
            &metric_definitions::actor::disable::skill_dc_min_key("thievery"),
            27.0,
        );
        assert_number_metric(
            &metrics,
            &metric_definitions::actor::disable::skill_dc_max_key("thievery"),
            30.0,
        );
        assert_number_metric(
            &metrics,
            &metric_definitions::actor::disable::skill_rank_min_key("thievery"),
            3.0,
        );
        assert_number_metric(
            &metrics,
            &metric_definitions::actor::disable::skill_dc_min_key("crafting"),
            30.0,
        );
    }

    fn assert_number_metric(metrics: &[MetricRow], key: &str, expected: f64) {
        let actual = metrics.iter().find_map(|metric| {
            if metric.domain == MetricDomain::Actor
                && metric.key == key
                && let MetricValue::Number(value) = metric.value
            {
                return Some(value);
            }
            None
        });
        assert_eq!(actual, Some(expected), "metric {key} should match");
    }
}
