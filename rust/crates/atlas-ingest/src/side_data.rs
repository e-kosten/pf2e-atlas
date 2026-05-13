use serde_json::Value;

use crate::metrics::{first_number_like_at_paths, number_like_at_pointer};
use crate::normalize::{
    extract_damage_types, extract_disable_skills, extract_sense_types, extract_speed_types,
    normalized_pointer_string, parse_bulk_value, parse_hands_requirement, pointer_bool,
    pointer_string, string_array_at_pointer, strip_markup, typed_collection,
};
use crate::{ActorSideData, ItemSideData, SpellSideData};

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
