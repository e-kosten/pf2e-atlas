use atlas_domain::{MetricDomain, TimeKind};

use crate::{
    MetricDefinition, MetricRow, MetricValue, NormalizedTime, SpellMechanics, definition_for,
};

pub(crate) fn metric_number(metrics: &[MetricRow], domain: MetricDomain, key: &str) -> Option<f64> {
    metrics.iter().find_map(|metric| {
        if metric.domain == domain
            && metric.key == key
            && let MetricValue::Number(value) = metric.value
        {
            return Some(value);
        }
        None
    })
}

pub(crate) fn metric_number_for_definition(
    rows: &[MetricRow],
    definition: MetricDefinition,
) -> Option<f64> {
    metric_number(rows, definition.domain(), definition.exact_key()?)
}

pub(crate) fn format_ability_mods(metrics: &[MetricRow]) -> Option<String> {
    let parts = [
        ("str", "Str"),
        ("dex", "Dex"),
        ("con", "Con"),
        ("int", "Int"),
        ("wis", "Wis"),
        ("cha", "Cha"),
    ]
    .into_iter()
    .filter_map(|(key, label)| {
        metric_number(
            metrics,
            MetricDomain::Actor,
            &crate::metrics::actor::ability::mod_key(key),
        )
        .map(|value| format!("{label} {}", format_modifier(value)))
    })
    .collect::<Vec<_>>();
    non_empty_join(parts)
}

pub(crate) fn format_skill_mods(metrics: &[MetricRow]) -> Option<String> {
    let mut parts = metrics
        .iter()
        .filter_map(|metric| {
            let skill = and_definition_match(metric)
                .filter(|matched| *matched.definition == crate::metrics::actor::skill::MOD)
                .and_then(|matched| {
                    matched
                        .captures
                        .first()
                        .map(|capture| capture.label.clone())
                })?;
            match metric.value {
                MetricValue::Number(value) => Some(format!("{skill} {}", format_modifier(value))),
                MetricValue::Text(_) | MetricValue::Boolean(_) => None,
            }
        })
        .collect::<Vec<_>>();
    parts.sort();
    non_empty_join(parts)
}

pub(crate) fn format_saves(metrics: &[MetricRow]) -> Option<String> {
    let parts = [
        (crate::metrics::actor::save::mod_key("fort"), "Fort"),
        (crate::metrics::actor::save::mod_key("ref"), "Ref"),
        (crate::metrics::actor::save::mod_key("will"), "Will"),
    ]
    .into_iter()
    .filter_map(|(key, label)| {
        metric_number(metrics, MetricDomain::Actor, &key)
            .map(|value| format!("{label} {}", format_modifier(value)))
    })
    .collect::<Vec<_>>();
    non_empty_join(parts)
}

pub(crate) fn format_speeds(metrics: &[MetricRow]) -> Option<String> {
    let mut parts = metrics
        .iter()
        .filter_map(|metric| {
            let speed = and_definition_match(metric)
                .filter(|matched| *matched.definition == crate::metrics::actor::speed::VALUE)
                .and_then(|matched| {
                    matched
                        .captures
                        .first()
                        .map(|capture| capture.label.clone())
                })?;
            match metric.value {
                MetricValue::Number(value) => Some(format!("{speed} {}", format_feet(value))),
                MetricValue::Text(_) | MetricValue::Boolean(_) => None,
            }
        })
        .collect::<Vec<_>>();
    parts.sort();
    non_empty_join(parts)
}

pub(crate) fn format_stealth(metrics: &[MetricRow]) -> Option<String> {
    let stealth_mod = metric_number_for_definition(metrics, crate::metrics::actor::STEALTH_MOD)
        .map(format_modifier);
    let stealth_dc =
        metric_number_for_definition(metrics, crate::metrics::actor::STEALTH_DC).map(format_number);
    match (stealth_mod, stealth_dc) {
        (Some(modifier), Some(dc)) => Some(format!("{modifier} (DC {dc})")),
        (Some(modifier), None) => Some(modifier),
        (None, Some(dc)) => Some(format!("DC {dc}")),
        (None, None) => None,
    }
}

fn and_definition_match(metric: &MetricRow) -> Option<crate::MetricDefinitionMatch> {
    definition_for(metric.domain, &metric.key)
}

pub(crate) fn format_list(values: &[String]) -> Option<String> {
    non_empty_join(values.iter().map(|value| humanize(value)).collect())
}

pub(crate) fn format_area(spell: &SpellMechanics) -> Option<String> {
    let area = spell.area.as_ref()?;
    match (&area.kind, area.value) {
        (Some(area_type), Some(area_value)) => Some(format!(
            "{} {}",
            format_number(area_value),
            humanize(area_type)
        )),
        (Some(area_type), None) => Some(humanize(area_type)),
        (None, Some(area_value)) => Some(format_number(area_value)),
        (None, None) => None,
    }
}

pub(crate) fn format_save(spell: &SpellMechanics) -> Option<String> {
    let defense = spell.defense.as_ref()?;
    defense.save.as_ref().map(|save_type| {
        if defense.basic {
            format!("basic {}", humanize(save_type))
        } else {
            humanize(save_type)
        }
    })
}

pub(crate) fn activation_text(
    activation_time: Option<&NormalizedTime>,
    system_actions_value: Option<i64>,
) -> Option<String> {
    activation_time
        .map(format_time)
        .or_else(|| action_count_text(system_actions_value))
}

pub(crate) fn duration_text(duration: Option<&NormalizedTime>) -> Option<String> {
    duration.map(format_time)
}

pub(crate) fn action_count_text(value: Option<i64>) -> Option<String> {
    match value {
        Some(1) => Some("1 action".to_string()),
        Some(value) if value > 1 => Some(format!("{value} actions")),
        _ => None,
    }
}

pub(crate) fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}

pub(crate) fn format_price_cp(price_cp: i64) -> String {
    let gp = price_cp / 100;
    let sp = (price_cp % 100) / 10;
    let cp = price_cp % 10;
    let mut parts = Vec::new();
    if gp > 0 {
        parts.push(format!("{gp} gp"));
    }
    if sp > 0 {
        parts.push(format!("{sp} sp"));
    }
    if cp > 0 || price_cp == 0 {
        parts.push(format!("{cp} cp"));
    }
    parts.join(", ")
}

pub(crate) fn format_bulk(value: f64) -> String {
    if value == 0.0 {
        "-".to_string()
    } else if value == 0.1 {
        "L".to_string()
    } else {
        format_number(value)
    }
}

pub fn format_size(value: &str) -> String {
    match value {
        "tiny" => "Tiny".to_string(),
        "sm" | "small" => "Small".to_string(),
        "med" | "medium" => "Medium".to_string(),
        "lg" | "large" => "Large".to_string(),
        "huge" => "Huge".to_string(),
        "grg" | "gargantuan" => "Gargantuan".to_string(),
        _ => humanize(value),
    }
}

pub(crate) fn humanize(value: &str) -> String {
    value
        .split(['_', '-'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first
                    .to_uppercase()
                    .chain(chars.flat_map(char::to_lowercase))
                    .collect(),
                None => String::new(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn non_empty_join(parts: Vec<String>) -> Option<String> {
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(", "))
    }
}

fn format_time(time: &NormalizedTime) -> String {
    if !time.text.trim().is_empty() {
        return time.text.clone();
    }
    match time.kind {
        TimeKind::Actions => {
            action_count_text(time.actions).unwrap_or_else(|| "Actions".to_string())
        }
        TimeKind::Free => "Free".to_string(),
        TimeKind::Reaction => "Reaction".to_string(),
        TimeKind::Duration => match (time.duration_value, time.duration_unit) {
            (Some(value), Some(unit)) => format!("{value} {}", unit.as_str()),
            _ => "Duration".to_string(),
        },
        TimeKind::Variable => "Variable".to_string(),
        TimeKind::Other => "Other".to_string(),
    }
}

pub(crate) fn format_modifier(value: f64) -> String {
    let formatted = format_number(value);
    if value >= 0.0 {
        format!("+{formatted}")
    } else {
        formatted
    }
}

fn format_feet(value: f64) -> String {
    format!("{} feet", format_number(value))
}
