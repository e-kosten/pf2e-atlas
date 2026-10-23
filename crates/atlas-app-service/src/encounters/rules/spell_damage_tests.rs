use super::super::conditions::{ConditionRule, modeled_condition_by_rule};
use atlas_local_state::*;
use serde_json::json;
fn participant() -> EncounterParticipant {
    EncounterParticipant {
        level_override: None,
        base_death_threshold: 4,
        persistent_precedence: vec![],
        track_dying: false,
        mortality: Default::default(),
        spell_resources: vec![],
        expended_spells: vec![],
        revision: 0,
        hp_origin: ParticipantHpOrigin::Explicit,
        variant_origin: ParticipantVariantOrigin::Explicit,
        participant_key: "p".into(),
        record_key: None,
        participant_kind: ParticipantKind::Creature,
        participant_variant: ParticipantVariant::Normal,
        position: 0,
        display_name: "Creature".into(),
        record_title_snapshot: None,
        record_kind_snapshot: None,
        side: ParticipantSide::Enemy,
        initiative: None,
        initiative_order: 0,
        max_hp: Some(60),
        current_hp: Some(37),
        temporary_hp: 0,
        defeated: false,
        hazard_disabled: false,
        hidden: false,
        note: None,
        created_at: String::new(),
        updated_at: String::new(),
        conditions: vec![],
    }
}
fn source(
    apply_mod: serde_json::Value,
    formula: &str,
    ability: Option<i64>,
) -> atlas_record::source_record::SourceBackedRecord {
    let mut value = json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","name":"Caster","system":{"details":{"level":{"value":4}},"attributes":{"adjustment":null},"abilities":{}},"items":[
        {"_id":"bbbbbbbbbbbbbbbb","type":"spellcastingEntry","name":"Strength magic","system":{"ability":{"value":"str"},"spelldc":{"value":12,"dc":22}}},
        {"_id":"cccccccccccccccc","type":"spell","name":"Bolt","system":{"level":{"value":1},"location":{"value":"bbbbbbbbbbbbbbbb"},"traits":{"value":["attack"]},"ritual":null,"damage":{"first":{"formula":formula,"applyMod":apply_mod,"type":"fire","kinds":["damage"],"category":null}}}}
    ]});
    if let Some(value_mod) = ability {
        value["system"]["abilities"]["str"] = json!({"mod":value_mod});
    }
    atlas_search::test_support::record("actors", "Actor", value).record
}
#[test]
fn spell_ability_and_strength_damage_penalty_apply_once_with_variant() {
    let source = source(json!(true), "1d6", Some(5));
    let baseline = super::source_baseline(&source, "fixture", None).unwrap();
    let mut participant = participant();
    participant.participant_variant = ParticipantVariant::Elite;
    let definition = modeled_condition_by_rule(ConditionRule::Enfeebled).unwrap();
    participant.conditions.push(EncounterParticipantCondition {
        condition_id: 1,
        condition_key: Some(definition.condition_ref.into()),
        name: definition.name.into(),
        value: Some(2),
        source_participant_key: None,
        scope: Default::default(),
        extra_action: Default::default(),
        immunity_override: false,
        parameters: Default::default(),
        origin: None,
        duration: None,
        note: None,
        created_at: String::new(),
        updated_at: String::new(),
    });
    let projected = super::project_participant(&participant, &baseline, true, &[]);
    let damage = &projected
        .activities
        .iter()
        .find(|activity| activity.label == "Bolt")
        .unwrap()
        .damage[0];
    assert_eq!(damage.adjusted_formula.as_deref(), Some("(1d6)+7"));
    assert!(damage.unavailable_reason.is_none());
    assert_eq!(
        super::project_participant(&participant, &baseline, true, &[]),
        projected
    );
}
#[test]
fn missing_ability_or_unsupported_formula_never_produces_partial_total() {
    for source in [
        source(json!(true), "1d6", None),
        source(json!(true), "@actor.level+1d6", Some(5)),
    ] {
        let baseline = super::source_baseline(&source, "fixture", None).unwrap();
        let projected = super::project_participant(&participant(), &baseline, true, &[]);
        let damage = &projected
            .activities
            .iter()
            .find(|activity| activity.label == "Bolt")
            .unwrap()
            .damage[0];
        assert!(damage.unavailable_reason.is_some());
        assert!(damage.adjusted_formula.is_none());
    }
}
#[test]
fn false_apply_mod_does_not_require_an_ability() {
    let source = source(json!(false), "1d6", None);
    let baseline = super::source_baseline(&source, "fixture", None).unwrap();
    let projected = super::project_participant(&participant(), &baseline, true, &[]);
    let damage = &projected
        .activities
        .iter()
        .find(|activity| activity.label == "Bolt")
        .unwrap()
        .damage[0];
    assert!(damage.unavailable_reason.is_none());
    assert!(damage.adjusted_formula.is_none());
}

#[test]
fn invalid_selected_rank_does_not_fall_back_to_an_exact_base_spell() {
    let source = source(json!(false), "1d6", Some(5));
    let baseline = super::source_baseline(&source, "fixture", None).unwrap();
    let owners = &baseline
        .activities
        .iter()
        .find(|activity| activity.label == "Bolt")
        .unwrap()
        .navigation
        .owners;
    let selected = super::source_baseline(&source, "fixture", Some((owners, 0))).unwrap();
    let spell = selected
        .activities
        .iter()
        .find(|activity| activity.label == "Bolt")
        .unwrap();
    assert!(spell.calculation_unavailable.is_some());
    assert!(spell.rolls.is_empty());
    assert!(spell.difficulty_class.is_none());
    assert!(
        spell
            .damage
            .iter()
            .all(|damage| damage.unavailable_reason.is_some())
    );
    assert!(
        selected
            .activities
            .iter()
            .find(|activity| activity.label == "Strength magic")
            .unwrap()
            .difficulty_class
            .is_some()
    );
}
