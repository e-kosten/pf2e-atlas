use super::*;
use crate::source_record::SourceNodeView;
use atlas_foundry_model::{SourceContext, admit_document_source};
use serde_json::{Value, json};

fn source(family: &str, system: Value) -> atlas_foundry_model::FoundryDocumentSource {
    admit_document_source(
        "Item",
        SourceContext::new("fixture", "test", "$"),
        &serde_json::to_vec(
            &json!({"_id":"aaaaaaaaaaaaaaaa", "name":"Fixture", "type":family, "system":system}),
        )
        .unwrap(),
    )
    .unwrap()
    .model
    .unwrap()
}
fn item(source: &atlas_foundry_model::FoundryDocumentSource) -> ItemSourceView<'_> {
    match SourceNodeView::from(source) {
        SourceNodeView::Item(i) => i,
        _ => panic!("expected Item"),
    }
}

#[test]
fn spell_authored_and_derived_fields_keep_states_and_units() {
    let s = source(
        "spell",
        json!({"level":{"value":3},"traits":{"value":["cantrip"],"traditions":[]},"time":{"value":" Reaction "},"ritual":null,
        "defense":{"save":{"statistic":"reflex","basic":false},"passive":null},"area":{"type":"burst","value":30},
        "duration":{"sustained":false,"value":"1 minute"},"damage":{"fire":{"type":"fire","kinds":["damage"]},"life":{"type":"vitality","kinds":["healing"]}}}),
    );
    let i = item(&s);
    assert_eq!(i.spell_rank().value().unwrap().as_i64(), Some(3));
    assert_eq!(i.item_level(), SourceFieldView::NotApplicable);
    assert_eq!(i.spell_cantrip(), SourceFieldView::Value(true));
    assert_eq!(i.spell_focus(), SourceFieldView::Value(true));
    assert_eq!(i.spell_ritual(), SourceFieldView::Value(false));
    assert_eq!(i.spell_casting_time().value(), Some(" Reaction "));
    assert_eq!(i.spell_casting_form().value(), Some("reaction"));
    assert_eq!(i.spell_defense_basic(), SourceFieldView::Value(false));
    assert_eq!(
        i.spell_defense_save().value(),
        Some(&SpellDefenseSourceSaveStatistic::Reflex)
    );
    assert_eq!(i.spell_defense_passive(), SourceFieldView::Null);
    assert_eq!(i.spell_area_type().value(), Some(&SpellAreaType::Burst));
    assert_eq!(i.spell_area_size().value().unwrap().as_i64(), Some(30));
    assert_eq!(i.spell_sustained(), SourceFieldView::Value(false));
    assert_eq!(i.spell_duration_text().value(), Some("1 minute"));
    let damage = &i.spell_damage().value().unwrap().entries;
    assert_eq!(damage.len(), 2);
    assert_eq!(damage[0].0, "fire");
    assert_eq!(damage[1].0, "life");
    assert_eq!(
        damage[0].1.r#type,
        SourcePresence::Value(ConsumableDamageHealingType::Fire)
    );
    assert_eq!(
        damage[0].1.kinds,
        SourcePresence::Value(vec![DamageKind::Damage])
    );
    assert_eq!(
        damage[1].1.kinds,
        SourcePresence::Value(vec![DamageKind::Healing])
    );
    let s = source(
        "spell",
        json!({"area":{"value":"30"},"traits":{"value":["focus"]}}),
    );
    assert!(matches!(
        item(&s).spell_area_size(),
        SourceFieldView::ProjectionInvalid { .. }
    ));
    assert_eq!(item(&s).spell_focus(), SourceFieldView::Value(true));
    assert_eq!(item(&s).spell_ritual(), SourceFieldView::Missing);
    let s = source(
        "spell",
        json!({"traits":{"value":["cantrip"]},"ritual":"bad","damage":{"x":{"type":"fire"}}}),
    );
    assert_eq!(item(&s).spell_focus(), SourceFieldView::Missing);
    assert!(matches!(
        item(&s).spell_ritual(),
        SourceFieldView::Invalid(_)
    ));
    assert_eq!(
        item(&s).spell_damage().value().unwrap().entries[0].1.kinds,
        SourcePresence::Missing
    );
}

#[test]
fn casting_form_is_exact_glyph_lookup_without_duration_guessing() {
    for (text, form) in [
        ("2", "two_actions"),
        ("free", "free"),
        ("0", "free"),
        ("1 or 2", "one_or_two"),
        ("1 to 3", "one_to_three"),
        ("2 or 3", "two_or_three"),
        ("2 rounds", "two_rounds"),
        ("1 minute", "other"),
        ("2 to 2 rounds", "other"),
        ("", "other"),
        ("Unknown", "other"),
    ] {
        let s = source("spell", json!({"time":{"value":text}}));
        assert_eq!(item(&s).spell_casting_form().value(), Some(form), "{text}");
        assert_eq!(item(&s).spell_casting_time().value(), Some(text));
    }
}

#[test]
fn bounded_price_codec_preserves_fractional_copper_and_unknowns() {
    for (coins, per, expected) in [
        (json!({}), None, 0.0),
        (json!({"sp":1}), Some(10), 1.0),
        (json!({"sp":1}), Some(5), 2.0),
        (json!({"cp":1}), Some(10), 0.1),
        (json!({"cp":2,"sp":3,"gp":4,"pp":5}), Some(2), 2716.0),
    ] {
        let mut price = json!({"value":coins});
        if let Some(per) = per {
            price["per"] = json!(per);
        }
        let s = source("weapon", json!({"price":price}));
        assert_eq!(
            item(&s).physical_price_per_item_cp().value(),
            Some(expected)
        );
    }
    for price in [
        json!({"value":{"cp":-1}}),
        json!({"value":{"cp":0.5}}),
        json!({"value":{"cp":9007199254740993u64}}),
        json!({"value":{"pp":18446744073709551615u64}}),
        json!({"value":{},"per":0}),
        json!({"value":{},"per":0.5}),
    ] {
        let s = source("weapon", json!({"price":price}));
        assert!(
            matches!(
                item(&s).physical_price_per_item_cp(),
                SourceFieldView::ProjectionInvalid { .. }
            ),
            "{price}"
        );
    }
    let s = source(
        "weapon",
        json!({"price":{"value":{"cp":9007199254740992u64}}}),
    );
    assert_eq!(
        item(&s).physical_price_per_item_cp().value(),
        Some(9007199254740992.0)
    );
    let s = source("weapon", json!({"price":{"value":{"cp":null}}}));
    assert_eq!(item(&s).physical_price_per_item_cp(), SourceFieldView::Null);
    let s = source("weapon", json!({"price":{"value":{},"per":null}}));
    assert_eq!(item(&s).physical_price_per_item_cp(), SourceFieldView::Null);
    let s = source("weapon", json!({"price":{"value":{"cp":"1"}}}));
    assert!(matches!(
        item(&s).physical_price_per_item_cp(),
        SourceFieldView::Invalid(_)
    ));
}

#[test]
fn physical_baselines_do_not_apply_runtime_adjustments() {
    let s = source(
        "weapon",
        json!({"level":{"value":0},"size":"med","bulk":{"value":0.1},"price":{"value":{}},"usage":{"value":"held-in-one-hand"},"category":"martial","group":"sword","damage":{"damageType":"slashing"},"range":null,"reload":{"value":"-"}}),
    );
    let i = item(&s);
    assert_eq!(i.item_level().value().unwrap().as_i64(), Some(0));
    assert_eq!(i.physical_bulk().value().unwrap().as_f64(), Some(0.1));
    assert_eq!(
        i.physical_size().value(),
        Some(&PhysicalSystemSourceSize::Med)
    );
    assert_eq!(i.physical_usage().value(), Some("held-in-one-hand"));
    assert_eq!(
        i.weapon_category().value(),
        Some(&WeaponSystemSourceCategory::Martial)
    );
    assert_eq!(
        i.weapon_group().value(),
        Some(&WeaponSystemSourceGroup::Sword)
    );
    assert_eq!(
        i.weapon_damage_type().value(),
        Some(&ConsumableDamageHealingType::Slashing)
    );
    assert_eq!(i.weapon_range_increment(), SourceFieldView::Null);
    assert_eq!(
        i.weapon_reload().value(),
        Some(&WeaponSystemSourceReloadValue::ValueU2D)
    );
    let s = source(
        "armor",
        json!({"category":"heavy","acBonus":6,"dexCap":0,"bulk":{"value":4},"usage":{"value":"worn"},"runes":{"potency":3}}),
    );
    let i = item(&s);
    assert_eq!(
        i.armor_category().value(),
        Some(&ArmorSystemSourceCategory::Heavy)
    );
    assert_eq!(i.armor_ac_bonus().value().unwrap().as_i64(), Some(6));
    assert_eq!(i.armor_dex_cap().value().unwrap().as_i64(), Some(0));
    assert_eq!(i.physical_usage(), SourceFieldView::NotApplicable);
    let s = source("shield", json!({"hardness":5,"hp":{"max":20,"value":2}}));
    assert_eq!(
        item(&s).shield_hardness().value().unwrap().as_i64(),
        Some(5)
    );
    assert_eq!(
        item(&s).shield_hp_maximum().value().unwrap().as_i64(),
        Some(20)
    );
    let s = source("consumable", json!({"category":"ammo"}));
    assert_eq!(
        item(&s).consumable_category().value(),
        Some(&ConsumableSystemSourceCategory::Ammo)
    );
    let s = source(
        "kit",
        json!({"price":{"value":{"gp":1}},"bulk":{"value":1}}),
    );
    assert_eq!(
        item(&s).physical_price_per_item_cp(),
        SourceFieldView::NotApplicable
    );
    assert_eq!(item(&s).physical_bulk(), SourceFieldView::NotApplicable);
}

#[test]
fn character_options_effect_condition_and_deity_borrow_authoritative_facts() {
    let s = source("ancestry", json!({"size":"sm"}));
    assert_eq!(
        item(&s).ancestry_size().value(),
        Some(&PhysicalSystemSourceSize::Sm)
    );
    let s = source(
        "action",
        json!({"actionType":{"value":"passive"},"actions":{"value":null},"category":"interaction"}),
    );
    assert_eq!(
        item(&s).action_type().value(),
        Some(&AbilitySystemSourceActionTypeSourceFromSchemaValue::Passive)
    );
    assert_eq!(item(&s).action_count(), SourceFieldView::Null);
    assert_eq!(
        item(&s).action_category().value(),
        Some(&AbilitySystemSourceCategory::Interaction)
    );
    let s = source("feat", json!({"category":"class","actions":{"value":2}}));
    assert_eq!(
        item(&s).feat_category().value(),
        Some(&FeatSystemSourceCategory::Class)
    );
    assert_eq!(item(&s).action_count().value().unwrap().as_i64(), Some(2));
    let s = source("campaignFeature", json!({"category":"kingdom-feat"}));
    assert_eq!(
        item(&s).campaign_feature_category().value(),
        Some(&CampaignFeatureSystemSourceCategory::KingdomFeat)
    );
    let s = source(
        "heritage",
        json!({"ancestry":{"slug":"elf","uuid":"Compendium.pf2e.ancestries.Item.aaaaaaaaaaaaaaaa"}}),
    );
    assert_eq!(item(&s).heritage_ancestry_slug().value(), Some("elf"));
    assert_eq!(
        item(&s).heritage_ancestry_uuid().value(),
        Some("Compendium.pf2e.ancestries.Item.aaaaaaaaaaaaaaaa")
    );
    assert_eq!(item(&s).heritage_versatile(), SourceFieldView::Value(false));
    let s = source("heritage", json!({"ancestry":null}));
    assert_eq!(item(&s).heritage_versatile(), SourceFieldView::Value(true));
    assert_eq!(item(&s).heritage_ancestry_slug(), SourceFieldView::Null);
    let s = source(
        "effect",
        json!({"duration":{"unit":"unlimited","value":-1}}),
    );
    assert_eq!(
        item(&s).effect_duration_unit().value(),
        Some(&DurationDataUnit::Unlimited)
    );
    assert_eq!(
        item(&s).effect_duration(DurationDataUnit::Rounds),
        SourceFieldView::NotApplicable
    );
    let s = source("effect", json!({"duration":{"unit":"hours","value":3}}));
    assert_eq!(
        item(&s)
            .effect_duration(DurationDataUnit::Hours)
            .value()
            .unwrap()
            .as_i64(),
        Some(3)
    );
    assert_eq!(
        item(&s).effect_duration(DurationDataUnit::Rounds),
        SourceFieldView::NotApplicable
    );
    let s = source(
        "condition",
        json!({"value":{"isValued":false,"value":null}}),
    );
    assert_eq!(
        item(&s).condition_is_valued(),
        SourceFieldView::Value(false)
    );
    let s = source(
        "deity",
        json!({"domains":{"primary":["moon"],"alternate":["sun"]},"font":["harm","heal"]}),
    );
    assert_eq!(
        item(&s).deity_domains_primary().value(),
        Some(&[DeitySystemSourceDomainsAlternateEntry::Moon][..])
    );
    assert_eq!(
        item(&s).deity_domains_alternate().value(),
        Some(&[DeitySystemSourceDomainsAlternateEntry::Sun][..])
    );
    assert_eq!(item(&s).deity_fonts().value(), Some(&["harm", "heal"][..]));
}

#[test]
fn missing_null_and_invalid_ancestors_propagate() {
    for system in [json!({}), Value::Null, json!("wrong")] {
        let s = source("spell", system.clone());
        let i = item(&s);
        let fields = [
            i.spell_casting_time().availability(),
            i.spell_area_size().availability(),
            i.spell_focus().availability(),
            i.spell_damage().availability(),
        ];
        for field in fields {
            match &system {
                Value::Null => assert_eq!(field, super::super::FieldAvailability::Null),
                Value::String(_) => assert!(matches!(
                    field,
                    super::super::FieldAvailability::Invalid { .. }
                )),
                _ => assert_eq!(field, super::super::FieldAvailability::Missing),
            }
        }
    }
}

#[test]
fn shared_item_level_and_physical_capabilities_have_explicit_applicability() {
    for family in [
        "action",
        "affliction",
        "ancestry",
        "armor",
        "background",
        "book",
        "campaignFeature",
        "class",
        "condition",
        "consumable",
        "backpack",
        "deity",
        "effect",
        "equipment",
        "feat",
        "heritage",
        "kit",
        "lore",
        "melee",
        "shield",
        "spell",
        "spellcastingEntry",
        "treasure",
        "weapon",
    ] {
        let s = source(
            family,
            json!({"level":{"value":4},"bulk":{"value":0},"size":"med","price":{"value":{}},"usage":{"value":"held-in-one-hand"}}),
        );
        let i = item(&s);
        let physical = matches!(
            family,
            "armor"
                | "book"
                | "consumable"
                | "backpack"
                | "equipment"
                | "shield"
                | "treasure"
                | "weapon"
        );
        let level =
            physical || matches!(family, "affliction" | "campaignFeature" | "effect" | "feat");
        assert_eq!(i.item_level().value().is_some(), level, "{family} level");
        assert_eq!(
            i.physical_bulk().value().is_some(),
            physical,
            "{family} bulk"
        );
        assert_eq!(
            i.physical_size().value().is_some(),
            physical,
            "{family} size"
        );
        assert_eq!(
            i.physical_price_per_item_cp().value().is_some(),
            physical,
            "{family} price"
        );
        assert_eq!(
            i.physical_usage().value().is_some(),
            physical && !matches!(family, "armor" | "shield" | "treasure"),
            "{family} usage"
        );
        assert_eq!(
            i.ancestry_size().value().is_some(),
            family == "ancestry",
            "{family} ancestry size"
        );
    }
}

#[test]
fn root_and_actor_item_use_the_same_extractor_and_original_component_order() {
    let system = json!({"level":{"value":2},"traits":{"value":["cantrip"],"traditions":["arcane"]},"damage":{"z":{"type":"fire","kinds":["damage"]},"a":{"type":"cold","kinds":["healing"]}}});
    let root = source("spell", system.clone());
    let actor=admit_document_source("Actor",SourceContext::new("fixture","test","$"),
        &serde_json::to_vec(&json!({"_id":"bbbbbbbbbbbbbbbb","type":"npc","items":[{"_id":"aaaaaaaaaaaaaaaa","type":"spell","system":system}]})).unwrap()).unwrap().model.unwrap();
    let embedded = SourceNodeView::from(&actor).actor_items().value().unwrap();
    let child = ItemSourceView::from(&embedded[0]);
    let root = item(&root);
    assert_eq!(child.spell_rank(), root.spell_rank());
    assert_eq!(child.spell_traditions(), root.spell_traditions());
    assert_eq!(child.spell_focus(), root.spell_focus());
    assert_eq!(child.spell_damage(), root.spell_damage());
}
