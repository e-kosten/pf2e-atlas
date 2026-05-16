use serde_json::Value;

use atlas_record::render_plain_text;

use crate::generated::afflictions::AfflictionFamily;
use crate::records::references::extract_reference_candidates_from_text;
use crate::records::variants;
use crate::source::normalize::{
    extract_traits, normalize_text, normalized_pointer_string, parse_foundry_content,
    pointer_string,
};

pub(super) fn detect_affliction_family(raw: &Value) -> Option<AfflictionFamily> {
    let traits = extract_traits(raw);
    let system_category =
        normalized_pointer_string(raw, "/system/category").map(|value| normalize_text(&value));
    if traits.iter().any(|trait_value| trait_value == "disease")
        || system_category.as_deref() == Some("disease")
    {
        return Some(AfflictionFamily::Disease);
    }
    if traits.iter().any(|trait_value| trait_value == "poison")
        || system_category.as_deref() == Some("poison")
    {
        return Some(AfflictionFamily::Poison);
    }
    if traits.iter().any(|trait_value| trait_value == "curse")
        || system_category.as_deref() == Some("curse")
    {
        return Some(AfflictionFamily::Curse);
    }
    None
}

pub(super) fn affliction_family_label(family: AfflictionFamily) -> &'static str {
    match family {
        AfflictionFamily::Curse => "curse",
        AfflictionFamily::Disease => "disease",
        AfflictionFamily::Poison => "poison",
    }
}

pub(super) fn has_affliction_shape(raw: &Value) -> bool {
    let Some(description) = record_description_plain_text(raw) else {
        return false;
    };
    let normalized = normalize_text(&description);
    normalized.contains("saving throw") && normalized.contains("stage 1")
}

pub(super) fn record_description_markup(raw: &Value) -> Option<String> {
    [
        "/system/description/value",
        "/system/details/description",
        "/system/details/publicNotes",
        "/system/details/blurb",
    ]
    .into_iter()
    .find_map(|pointer| pointer_string(raw, pointer))
    .filter(|value| !value.trim().is_empty())
}

pub(super) fn record_description_plain_text(raw: &Value) -> Option<String> {
    record_description_markup(raw)
        .map(|value| render_plain_text(&parse_foundry_content(&value).document))
        .filter(|value| !value.trim().is_empty())
}

pub(super) fn record_slug(raw: &Value) -> Option<String> {
    normalized_pointer_string(raw, "/system/slug")
}

pub(super) fn compendium_source(raw: &Value) -> Option<String> {
    normalized_pointer_string(raw, "/_stats/compendiumSource")
}

pub(super) fn parse_compendium_source(value: &str) -> Option<(String, String)> {
    let parts = value.split('.').collect::<Vec<_>>();
    if parts.len() >= 5 && parts.first() == Some(&"Compendium") && parts.get(1) == Some(&"pf2e") {
        return Some((normalize_text(parts.get(2)?), normalize_text(parts.last()?)));
    }
    None
}

pub(super) fn extract_linked_names_from_markup(markup: Option<&str>) -> Vec<String> {
    let Some(markup) = markup else {
        return Vec::new();
    };
    variants::sorted_unique(
        extract_reference_candidates_from_text(markup)
            .into_iter()
            .filter_map(|candidate| {
                candidate
                    .display_text
                    .or_else(|| fallback_linked_name(&candidate.raw_target))
            })
            .collect(),
    )
}

fn fallback_linked_name(locator: &str) -> Option<String> {
    let tail = locator.split('.').next_back()?.replace(['-', '_'], " ");
    let trimmed = tail.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}
