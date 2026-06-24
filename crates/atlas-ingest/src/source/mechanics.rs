use serde_json::Value;

use atlas_record::{
    ActivityRoll, ActivityRollAbility, ActivityRollSurface, ActorMechanics, DamageExpression,
    ItemMechanics, MechanicActivity, MechanicActivityKind, MechanicActivityUsage, SpellArea,
    SpellDefense, SpellMechanics, SpellRange, SpellTarget, SpellcastingEntryMechanics,
    SpellcastingPreparation, render_plain_text,
};

use crate::records::EmbeddedItemFact;
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

pub(super) fn extract_embedded_record_mechanics(
    embedded_items: &[EmbeddedItemFact],
) -> (Vec<SpellcastingEntryMechanics>, Vec<MechanicActivity>) {
    let spellcasting_entries = embedded_items
        .iter()
        .filter_map(spellcasting_entry)
        .collect::<Vec<_>>();
    let activities = embedded_items
        .iter()
        .filter_map(|item| activity(item, &spellcasting_entries))
        .collect::<Vec<_>>();
    (spellcasting_entries, activities)
}

fn spellcasting_entry(item: &EmbeddedItemFact) -> Option<SpellcastingEntryMechanics> {
    if item.foundry_item_type != "spellcastingEntry" {
        return None;
    }
    let raw = item.raw_provenance.as_ref()?;
    let preparation = match normalized_pointer_string(raw, "/system/prepared/value").as_deref() {
        Some("prepared") => SpellcastingPreparation::Prepared,
        Some("spontaneous") => SpellcastingPreparation::Spontaneous,
        Some("focus") => SpellcastingPreparation::Focus,
        Some("innate") => SpellcastingPreparation::Innate,
        Some(value) => SpellcastingPreparation::Other(value.to_string()),
        None => SpellcastingPreparation::Other("unknown".to_string()),
    };
    Some(SpellcastingEntryMechanics {
        entry_id: item.item_id.clone(),
        label: item.name.clone(),
        preparation,
        spell_attack: raw.pointer("/system/spelldc/value").and_then(Value::as_i64),
        spell_dc: raw.pointer("/system/spelldc/dc").and_then(Value::as_i64),
    })
}

fn activity(
    item: &EmbeddedItemFact,
    spellcasting_entries: &[SpellcastingEntryMechanics],
) -> Option<MechanicActivity> {
    let raw = item.raw_provenance.as_ref()?;
    match item.foundry_item_type.as_str() {
        "melee" => strike_activity(item, raw),
        "spell" => spell_activity(item, raw, spellcasting_entries),
        "action" => action_activity(item, raw),
        _ => None,
    }
}

fn strike_activity(item: &EmbeddedItemFact, raw: &Value) -> Option<MechanicActivity> {
    let ability = strike_ability(item);
    let damage = damage_rolls(raw, "/system/damageRolls", "damage", "damageType", ability);
    if damage.is_empty() {
        return None;
    }
    Some(MechanicActivity {
        activity_id: item.item_id.clone(),
        label: item.name.clone(),
        kind: MechanicActivityKind::Strike,
        traits: item.traits.clone(),
        compendium_source: item.compendium_source.clone(),
        usage: MechanicActivityUsage::Unlimited,
        rolls: strike_rolls(item, raw),
        damage,
    })
}

fn spell_activity(
    item: &EmbeddedItemFact,
    raw: &Value,
    spellcasting_entries: &[SpellcastingEntryMechanics],
) -> Option<MechanicActivity> {
    let damage = damage_rolls(raw, "/system/damage", "formula", "type", None);
    if damage.is_empty() {
        return None;
    }
    let usage = spell_usage(item, raw, spellcasting_entries);
    Some(MechanicActivity {
        activity_id: item.item_id.clone(),
        label: item.name.clone(),
        kind: MechanicActivityKind::Spell,
        traits: item.traits.clone(),
        compendium_source: item.compendium_source.clone(),
        usage,
        rolls: spell_rolls(raw, spellcasting_entries),
        damage,
    })
}

fn action_activity(item: &EmbeddedItemFact, raw: &Value) -> Option<MechanicActivity> {
    let damage = damage_rolls(raw, "/system/damageRolls", "damage", "damageType", None)
        .into_iter()
        .chain(damage_rolls(raw, "/system/damage", "formula", "type", None))
        .collect::<Vec<_>>();
    if damage.is_empty() {
        return None;
    }
    let usage = if raw
        .pointer("/system/frequency")
        .is_some_and(|value| !value.is_null())
    {
        MechanicActivityUsage::Limited
    } else {
        MechanicActivityUsage::Ambiguous
    };
    Some(MechanicActivity {
        activity_id: item.item_id.clone(),
        label: item.name.clone(),
        kind: MechanicActivityKind::Other,
        traits: item.traits.clone(),
        compendium_source: item.compendium_source.clone(),
        usage,
        rolls: Vec::new(),
        damage,
    })
}

fn strike_rolls(item: &EmbeddedItemFact, raw: &Value) -> Vec<ActivityRoll> {
    let Some(base_value) = raw.pointer("/system/bonus/value").and_then(Value::as_i64) else {
        return Vec::new();
    };
    vec![ActivityRoll {
        roll_id: "attack".to_string(),
        label: "Attack".to_string(),
        base_value,
        surface: ActivityRollSurface::AttackRoll,
        ability: strike_ability(item),
    }]
}

fn strike_ability(item: &EmbeddedItemFact) -> Option<ActivityRollAbility> {
    if item
        .traits
        .iter()
        .any(|trait_slug| trait_slug.starts_with("thrown-"))
    {
        return Some(ActivityRollAbility::Strength);
    }
    if item.traits.iter().any(|trait_slug| {
        trait_slug == "ranged"
            || trait_slug.starts_with("range-")
            || trait_slug.starts_with("reload-")
    }) {
        return Some(ActivityRollAbility::Dexterity);
    }
    Some(ActivityRollAbility::Strength)
}

fn spell_rolls(
    raw: &Value,
    spellcasting_entries: &[SpellcastingEntryMechanics],
) -> Vec<ActivityRoll> {
    let Some(entry_id) = normalized_pointer_string(raw, "/system/location/value") else {
        return Vec::new();
    };
    let Some(entry) = spellcasting_entries
        .iter()
        .find(|entry| entry.entry_id == entry_id)
    else {
        return Vec::new();
    };
    let mut rolls = Vec::new();
    if let Some(base_value) = entry.spell_attack {
        rolls.push(ActivityRoll {
            roll_id: "spell.attack".to_string(),
            label: "Spell Attack".to_string(),
            base_value,
            surface: ActivityRollSurface::AttackRoll,
            ability: None,
        });
    }
    if let Some(base_value) = entry.spell_dc {
        rolls.push(ActivityRoll {
            roll_id: "spell.dc".to_string(),
            label: "Spell DC".to_string(),
            base_value,
            surface: ActivityRollSurface::Dc,
            ability: None,
        });
    }
    rolls
}

fn spell_usage(
    item: &EmbeddedItemFact,
    raw: &Value,
    spellcasting_entries: &[SpellcastingEntryMechanics],
) -> MechanicActivityUsage {
    if item.traits.iter().any(|trait_slug| trait_slug == "cantrip") {
        return MechanicActivityUsage::Unlimited;
    }
    if raw
        .pointer("/system/location/uses")
        .is_some_and(|value| !value.is_null())
    {
        return MechanicActivityUsage::Limited;
    }
    let Some(entry_id) = normalized_pointer_string(raw, "/system/location/value") else {
        return MechanicActivityUsage::Ambiguous;
    };
    let Some(entry) = spellcasting_entries
        .iter()
        .find(|entry| entry.entry_id == entry_id)
    else {
        return MechanicActivityUsage::Ambiguous;
    };
    match entry.preparation {
        SpellcastingPreparation::Prepared
        | SpellcastingPreparation::Spontaneous
        | SpellcastingPreparation::Focus => MechanicActivityUsage::Limited,
        SpellcastingPreparation::Innate | SpellcastingPreparation::Other(_) => {
            MechanicActivityUsage::Ambiguous
        }
    }
}

fn damage_rolls(
    raw: &Value,
    pointer: &str,
    formula_field: &str,
    damage_type_field: &str,
    ability: Option<ActivityRollAbility>,
) -> Vec<DamageExpression> {
    let Some(entries) = raw.pointer(pointer).and_then(Value::as_object) else {
        return Vec::new();
    };
    let mut damage = entries
        .iter()
        .filter_map(|(damage_id, entry)| {
            let formula = normalized_pointer_string(entry, &format!("/{formula_field}"))?;
            if formula.is_empty() {
                return None;
            }
            Some(DamageExpression {
                damage_id: damage_id.clone(),
                label: normalized_pointer_string(entry, "/category"),
                formula,
                damage_type: normalized_pointer_string(entry, &format!("/{damage_type_field}")),
                ability,
            })
        })
        .collect::<Vec<_>>();
    damage.sort_by(|left, right| left.damage_id.cmp(&right.damage_id));
    damage
}

fn content_text(value: String, localization: Option<&dyn LocalizationResolver>) -> Option<String> {
    let text =
        render_plain_text(&parse_foundry_content_with_localization(&value, localization).document);
    (!text.trim().is_empty()).then_some(text)
}
