use serde_json::Value;

use crate::source::normalize::{
    LocalizationResolver, extract_damage_types, extract_disable_skills, extract_sense_types,
    extract_speed_types, normalized_pointer_string, parse_bulk_value,
    parse_foundry_content_with_localization, pointer_bool, pointer_string, string_array_at_pointer,
    typed_collection,
};
use atlas_record::{ActorMechanics, ItemMechanics, render_plain_text};

pub(super) fn extract_actor_mechanics(
    raw: &Value,
    localization: Option<&dyn LocalizationResolver>,
) -> ActorMechanics {
    let disable_text = pointer_string(raw, "/system/details/disable")
        .and_then(|value| content_text(value, localization));
    ActorMechanics {
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

pub(super) fn extract_item_mechanics(
    raw: &Value,
    category: Option<String>,
    base_item: Option<String>,
    group: Option<String>,
    usage: Option<String>,
    price_json: Option<String>,
    price_cp: Option<i64>,
) -> ItemMechanics {
    ItemMechanics {
        category,
        base_item,
        group,
        usage: usage.clone(),
        price_json,
        price_cp,
        bulk_value: raw.pointer("/system/bulk/value").and_then(parse_bulk_value),
        hands_requirement: usage
            .as_deref()
            .and_then(atlas_record::hands_requirement_from_usage)
            .map(str::to_string),
        damage_types: extract_damage_types(raw),
    }
}

fn content_text(value: String, localization: Option<&dyn LocalizationResolver>) -> Option<String> {
    let text =
        render_plain_text(&parse_foundry_content_with_localization(&value, localization).document);
    (!text.trim().is_empty()).then_some(text)
}
