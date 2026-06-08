use serde_json::Value;

use atlas_record::{
    ActorMechanics, ItemMechanics, SpellArea, SpellDefense, SpellMechanics, SpellRange,
    SpellTarget, render_plain_text,
};

use crate::records::metrics::{first_number_like_at_paths, number_like_at_pointer};
use crate::source::normalize::{
    LocalizationResolver, extract_damage_types, extract_disable_skills, extract_sense_types,
    extract_speed_types, normalized_pointer_string, parse_bulk_value,
    parse_foundry_content_with_localization, parse_hands_requirement, pointer_bool, pointer_string,
    string_array_at_pointer, typed_collection,
};

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
        foundry_type: None,
        category,
        base_item,
        group,
        usage: usage.clone(),
        price_json,
        price_cp,
        bulk_value: raw.pointer("/system/bulk/value").and_then(parse_bulk_value),
        hands_requirement: usage.as_deref().and_then(parse_hands_requirement),
        damage_types: extract_damage_types(raw),
    }
}

pub(super) fn extract_spell_mechanics(
    raw: &Value,
    traits: &[String],
    localization: Option<&dyn LocalizationResolver>,
) -> SpellMechanics {
    let range_text = normalized_pointer_string(raw, "/system/range/value");
    let range_distance =
        first_number_like_at_paths(raw, &["/system/range/value", "/system/range/increment"]);
    let target_text = pointer_string(raw, "/system/target/value")
        .and_then(|value| content_text(value, localization));
    let area_kind = normalized_pointer_string(raw, "/system/area/type");
    let area_value = number_like_at_pointer(raw, "/system/area/value");
    let save = normalized_pointer_string(raw, "/system/defense/save/statistic");
    let basic = pointer_bool(raw, "/system/defense/save/basic").unwrap_or(false);

    SpellMechanics {
        traditions: string_array_at_pointer(raw, "/system/traits/traditions"),
        kinds: ["focus", "ritual", "cantrip"]
            .into_iter()
            .filter(|kind| traits.iter().any(|value| value == kind))
            .map(str::to_string)
            .collect(),
        range: range_text.map(|text| SpellRange {
            text,
            distance: range_distance,
        }),
        target: target_text.map(|text| SpellTarget { text }),
        area: (area_kind.is_some() || area_value.is_some()).then_some(SpellArea {
            kind: area_kind,
            value: area_value,
        }),
        defense: (save.is_some() || basic).then_some(SpellDefense { save, basic }),
        sustained: pointer_bool(raw, "/system/duration/sustained").unwrap_or(false),
        damage_types: extract_damage_types(raw),
    }
}

fn content_text(value: String, localization: Option<&dyn LocalizationResolver>) -> Option<String> {
    let text =
        render_plain_text(&parse_foundry_content_with_localization(&value, localization).document);
    (!text.trim().is_empty()).then_some(text)
}
