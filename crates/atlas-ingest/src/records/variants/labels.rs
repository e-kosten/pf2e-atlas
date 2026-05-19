use crate::source::normalize::normalize_text;

use super::sorted_unique;

pub(super) fn infer_variant_axes(labels: &[String]) -> Vec<String> {
    let axes = labels
        .iter()
        .flat_map(|label| {
            let normalized = normalize_text(label);
            if is_rank_label(&normalized) {
                vec!["rank".to_string()]
            } else if is_grade_label(&normalized) {
                vec!["grade".to_string()]
            } else if is_damage_type_label(&normalized) {
                vec!["damageType".to_string()]
            } else if is_dragon_age_label(&normalized) {
                vec!["dragonAge".to_string()]
            } else if is_specialization_label(&normalized) {
                vec!["specialization".to_string()]
            } else if is_gender_label(&normalized) {
                Vec::new()
            } else {
                vec!["other".to_string()]
            }
        })
        .collect::<Vec<_>>();
    let axes = sorted_unique(axes);
    if axes.is_empty() {
        vec!["other".to_string()]
    } else {
        axes
    }
}

pub(super) fn is_grade_label(value: &str) -> bool {
    matches!(
        value,
        "minor" | "lesser" | "moderate" | "greater" | "major" | "true"
    )
}

fn is_damage_type_label(value: &str) -> bool {
    matches!(
        value,
        "acid" | "cold" | "electricity" | "fire" | "poison" | "sonic" | "void" | "vitality"
    )
}

pub(super) fn is_dragon_age_label(value: &str) -> bool {
    matches!(
        value,
        "wyrmling" | "hatchling" | "young" | "juvenile" | "adult" | "ancient" | "greatwyrm"
    )
}

pub(super) fn is_specialization_label(value: &str) -> bool {
    matches!(value, "spellcaster" | "elite" | "weak" | "variant")
}

pub(super) fn is_gender_label(value: &str) -> bool {
    matches!(value, "male" | "female")
}

pub(super) fn is_gender_only(labels: &[String]) -> bool {
    !labels.is_empty() && labels.iter().all(|label| is_gender_label(label))
}

pub(super) fn is_rank_label(value: &str) -> bool {
    let normalized = value.replace('-', " ");
    let mut parts = normalized.split_whitespace();
    let Some(ordinal) = parts.next() else {
        return false;
    };
    let Some(kind) = parts.next() else {
        return false;
    };
    is_ordinal(ordinal) && matches!(kind, "rank" | "level")
}

fn is_ordinal(value: &str) -> bool {
    ["st", "nd", "rd", "th"].iter().any(|suffix| {
        value
            .strip_suffix(suffix)
            .is_some_and(|prefix| prefix.parse::<u8>().is_ok())
    })
}

pub(super) fn normalize_rank_label(value: &str) -> String {
    value
        .replace('-', " ")
        .split_whitespace()
        .enumerate()
        .map(|(index, part)| {
            if index == 0 {
                part.to_string()
            } else {
                title_case_words(part)
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn title_case_words(value: &str) -> String {
    value
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            format!("{}{}", first.to_uppercase(), chars.as_str().to_lowercase())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn slugify_hyphen(value: &str) -> String {
    let mut output = String::new();
    let mut last_was_separator = false;
    for character in normalize_text(value).chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character);
            last_was_separator = false;
        } else if !last_was_separator && !output.is_empty() {
            output.push('-');
            last_was_separator = true;
        }
    }
    while output.ends_with('-') {
        output.pop();
    }
    output
}
