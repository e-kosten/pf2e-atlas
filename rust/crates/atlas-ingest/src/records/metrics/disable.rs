use atlas_domain::MetricDomain;
use atlas_record::metrics as metric_definitions;
use serde_json::Value;

use crate::records::MetricRow;
use crate::source::normalize::pointer_string;

use super::value::slugify_metric_segment;
use super::{add_defined_metric_number, add_metric_number};

pub(super) fn extract_disable_metrics(raw: &Value, metrics: &mut Vec<MetricRow>) {
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
