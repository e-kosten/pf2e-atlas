use crate::test_support::encounter_fixture_worker;
use atlas_app_model::*;

fn update(p: &EncounterParticipantView) -> UpdateEncounterParticipantRequest {
    UpdateEncounterParticipantRequest {
        hp_edit: false,
        max_hp_edit: false,
        variant_edit: false,
        use_derived_max: false,
        participant_key: p.participant_key.clone(),
        display_name: p.display_name.clone(),
        side: p.side,
        participant_variant: p.participant_variant,
        initiative: p.initiative,
        max_hp: p.max_hp,
        current_hp: p.current_hp,
        temporary_hp: p.temporary_hp,
        defeated: p.defeated,
        hidden: p.hidden,
        note: p.note.clone(),
    }
}
#[test]
fn encounter_surface_keeps_non_hp_sections_outside_the_replaceable_vitals_slot() {
    let f = encounter_fixture_worker();
    let e = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Grouped surface".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let d = f
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: e.encounter_key,
            record_ref: "Test Creature 1".into(),
            quantity: 1,
            initiative: None,
        })
        .unwrap();
    let p = &d.participants[0];
    let surface = p.surface.as_ref().unwrap();
    for (kind, key) in [
        (RecordSurfaceSectionKindView::Defenses, "ac"),
        (RecordSurfaceSectionKindView::Saves, "save.fortitude"),
        (RecordSurfaceSectionKindView::Abilities, "ability.str"),
        (RecordSurfaceSectionKindView::Skills, "skill.athletics"),
        (RecordSurfaceSectionKindView::Movement, "speed.land"),
        (RecordSurfaceSectionKindView::Runtime, "actions"),
    ] {
        assert!(
            surface
                .sections
                .iter()
                .any(|s| s.kind == kind && s.values.iter().any(|v| v.key == key)),
            "missing {kind:?}/{key}"
        );
    }
    assert!(
        surface
            .sections
            .iter()
            .filter(|s| s.kind == RecordSurfaceSectionKindView::Vitals)
            .flat_map(|s| &s.values)
            .all(|v| v.key == "hp.max")
    );
    assert!(
        surface
            .sections
            .iter()
            .filter(|s| s.kind == RecordSurfaceSectionKindView::Notes)
            .flat_map(|s| &s.notes)
            .any(|n| n.label == "Identifying spells")
    );
    assert!(
        surface
            .sections
            .iter()
            .flat_map(|s| &s.activities)
            .any(|a| a.label == "Breath Weapon")
    );
}

#[test]
fn derived_capacity_changes_clamp_valid_damage_at_zero_and_preserve_manual_outliers() {
    let f = encounter_fixture_worker();
    let e = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "HP boundaries".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let d = f
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: e.encounter_key.clone(),
            record_ref: "Test Creature 1".into(),
            quantity: 1,
            initiative: None,
        })
        .unwrap();
    let mut change = update(&d.participants[0]);
    change.participant_variant = EncounterParticipantVariantView::Elite;
    change.variant_edit = true;
    let p = f
        .worker
        .update_encounter_participant(&e.encounter_key, change)
        .unwrap();
    let mut change = update(&p);
    change.current_hp = Some(3);
    change.hp_edit = true;
    let p = f
        .worker
        .update_encounter_participant(&e.encounter_key, change)
        .unwrap();
    let mut change = update(&p);
    change.participant_variant = EncounterParticipantVariantView::Normal;
    change.variant_edit = true;
    let p = f
        .worker
        .update_encounter_participant(&e.encounter_key, change)
        .unwrap();
    assert_eq!((p.max_hp, p.current_hp), (Some(60), Some(0)));
    for current in [400, -5] {
        let mut change = update(&p);
        change.current_hp = Some(current);
        change.hp_edit = true;
        let p = f
            .worker
            .update_encounter_participant(&e.encounter_key, change)
            .unwrap();
        let mut change = update(&p);
        change.participant_variant = EncounterParticipantVariantView::Elite;
        change.variant_edit = true;
        let p = f
            .worker
            .update_encounter_participant(&e.encounter_key, change)
            .unwrap();
        assert_eq!(p.current_hp, Some(current));
    }
}

#[test]
fn known_normal_hp_does_not_require_level_and_runtime_level_uses_foundry_bounds() {
    use atlas_local_state::ParticipantVariant;
    let input = atlas_search::test_support::record(
        "actors",
        "Actor",
        serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","system":{"attributes":{"adjustment":null,"hp":{"max":60}}}}),
    );
    assert_eq!(
        super::mechanics::effective_max(&input.record, Some(ParticipantVariant::Normal)),
        Some(60)
    );
    assert_eq!(
        super::mechanics::effective_max(&input.record, Some(ParticipantVariant::Elite)),
        None
    );
    for (level, variant, expected) in [
        (i64::MIN, ParticipantVariant::Normal, -1),
        (-5, ParticipantVariant::Elite, 1),
        (0, ParticipantVariant::Elite, 2),
        (1, ParticipantVariant::Weak, -1),
        (i64::MAX, ParticipantVariant::Normal, 100),
        (200, ParticipantVariant::Elite, 101),
        (200, ParticipantVariant::Weak, 99),
    ] {
        assert_eq!(
            super::mechanics::adjusted_npc_level(level, variant),
            expected
        );
    }
}
#[test]
fn derived_hp_follows_pristine_variant_then_preserves_damage_and_explicit_capacity() {
    let f = encounter_fixture_worker();
    let encounter = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "HP policy".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let detail = f
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: encounter.encounter_key.clone(),
            record_ref: "Test Creature 1".into(),
            quantity: 1,
            initiative: None,
        })
        .unwrap();
    let p = &detail.participants[0];
    assert_eq!((p.max_hp, p.current_hp), (Some(60), Some(60)));
    assert_eq!(p.hp_origin, "derived_pristine");
    assert_eq!(p.variant_origin, "inherited_known");
    let mut change = update(p);
    change.participant_variant = EncounterParticipantVariantView::Elite;
    change.variant_edit = true;
    let p = f
        .worker
        .update_encounter_participant(&encounter.encounter_key, change)
        .unwrap();
    assert_eq!((p.max_hp, p.current_hp), (Some(80), Some(80)));
    let block = p.stat_block.as_ref().unwrap();
    assert_eq!(
        block
            .values
            .iter()
            .find(|v| v.target == "ac")
            .unwrap()
            .adjusted_value,
        serde_json::Number::from(24)
    );
    assert_eq!(
        block
            .activities
            .iter()
            .find(|a| a.label == "Claw")
            .unwrap()
            .damage[0]
            .formula,
        "1d6+4"
    );
    assert!(
        block
            .unapplied_effects
            .iter()
            .any(|n| n.label == "Identifying spells")
    );
    let mut change = update(&p);
    change.current_hp = Some(55);
    change.hp_edit = true;
    change.temporary_hp = 9;
    let p = f
        .worker
        .update_encounter_participant(&encounter.encounter_key, change)
        .unwrap();
    assert_eq!(p.hp_origin, "derived_edited");
    let mut change = update(&p);
    change.participant_variant = EncounterParticipantVariantView::Weak;
    change.variant_edit = true;
    let p = f
        .worker
        .update_encounter_participant(&encounter.encounter_key, change)
        .unwrap();
    assert_eq!(
        (p.max_hp, p.current_hp, p.temporary_hp),
        (Some(45), Some(20), 9)
    );
    let mut change = update(&p);
    change.max_hp = Some(200);
    change.max_hp_edit = true;
    change.current_hp = Some(-5);
    change.hp_edit = true;
    let p = f
        .worker
        .update_encounter_participant(&encounter.encounter_key, change)
        .unwrap();
    assert_eq!(p.hp_origin, "explicit");
    let mut change = update(&p);
    change.participant_variant = EncounterParticipantVariantView::Normal;
    change.variant_edit = true;
    let p = f
        .worker
        .update_encounter_participant(&encounter.encounter_key, change)
        .unwrap();
    assert_eq!((p.max_hp, p.current_hp), (Some(200), Some(-5)));
    let read = f.worker.encounter(&encounter.encounter_key).unwrap();
    assert_eq!(
        (read.participants[0].max_hp, read.participants[0].current_hp),
        (Some(200), Some(-5))
    );
}

#[test]
fn same_value_hp_edit_is_preserved_and_manual_hp_is_never_clamped() {
    let f = encounter_fixture_worker();
    let encounter = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Manual HP".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let detail = f
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: encounter.encounter_key.clone(),
            record_ref: "Test Creature 1".into(),
            quantity: 1,
            initiative: None,
        })
        .unwrap();
    let mut change = update(&detail.participants[0]);
    change.hp_edit = true;
    let p = f
        .worker
        .update_encounter_participant(&encounter.encounter_key, change)
        .unwrap();
    assert_eq!(p.hp_origin, "derived_edited");
    let mut change = update(&p);
    change.current_hp = Some(400);
    change.hp_edit = true;
    let p = f
        .worker
        .update_encounter_participant(&encounter.encounter_key, change)
        .unwrap();
    assert_eq!(p.current_hp, Some(400));
    let detail = f
        .worker
        .add_encounter_manual_participant(AddEncounterManualParticipantRequest {
            encounter_ref: encounter.encounter_key,
            display_name: "PC".into(),
            max_hp: Some(10),
            current_hp: Some(-2),
            initiative: None,
        })
        .unwrap();
    let p = detail
        .participants
        .iter()
        .find(|p| p.display_name == "PC")
        .unwrap();
    assert_eq!((p.max_hp, p.current_hp), (Some(10), Some(-2)));
    assert_eq!(p.hp_origin, "explicit");
}

#[test]
fn hp_numeric_policy_does_not_coerce_fractional_or_unsigned_out_of_range_values() {
    for n in [
        serde_json::Number::from(u64::MAX),
        serde_json::Number::from_f64(1.5).unwrap(),
    ] {
        assert_eq!(super::mechanics::integer(&n), None);
    }
    assert_eq!(
        super::mechanics::integer(&serde_json::Number::from(i64::MAX)),
        Some(i64::MAX)
    );
    assert_eq!(
        super::mechanics::integer(&serde_json::Number::from_f64(60.0).unwrap()),
        Some(60)
    );
}

#[test]
fn inherited_elite_applies_once_unknown_adjustment_stays_unknown_and_active_hp_preserves_damage() {
    use crate::{executor::RetrievalExecutor, test_support::fixture_worker_with_executor};
    use atlas_search::test_support::{open_source_fixture, record};
    use serde_json::json;
    let executor = RetrievalExecutor::from_test_fixture_factory(1, 16, || {
        Ok(open_source_fixture(
            vec![
                record(
                    "actors",
                    "Actor",
                    json!({"_id":"elite00000000000","name":"Elite source","type":"npc","system":{"details":{"level":{"value":5}},"attributes":{"adjustment":"elite","hp":{"max":60},"ac":{"value":22}}}}),
                ),
                record(
                    "actors",
                    "Actor",
                    json!({"_id":"unknown000000000","name":"Unknown adjustment","type":"npc","system":{"details":{"level":{"value":5}},"attributes":{"adjustment":"bad","hp":{"max":60}}}}),
                ),
                record(
                    "actors",
                    "Actor",
                    json!({"_id":"fraction00000000","name":"Fractional HP","type":"npc","system":{"details":{"level":{"value":5}},"attributes":{"adjustment":null,"hp":{"max":60.5}}}}),
                ),
            ],
            false,
        )?)
    });
    let f = fixture_worker_with_executor(executor);
    let e = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Source adjustment".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let add = |name: &str| {
        f.worker
            .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
                encounter_ref: e.encounter_key.clone(),
                record_ref: name.into(),
                quantity: 1,
                initiative: None,
            })
            .unwrap()
    };
    let detail = add("Elite source");
    let p = &detail.participants[0];
    assert_eq!((p.max_hp, p.current_hp), (Some(80), Some(80)));
    assert_eq!(
        p.participant_variant,
        EncounterParticipantVariantView::Elite
    );
    assert_eq!(
        p.stat_block
            .as_ref()
            .unwrap()
            .values
            .iter()
            .find(|v| v.target == "ac")
            .unwrap()
            .adjusted_value,
        serde_json::Number::from(24)
    );
    f.worker
        .update_encounter(UpdateEncounterRequest {
            encounter_key: e.encounter_key.clone(),
            slug: e.slug.clone(),
            name: e.name.clone(),
            description: None,
            note: None,
            status: EncounterStatusView::Running,
        })
        .unwrap();
    let mut change = update(p);
    change.participant_variant = EncounterParticipantVariantView::Normal;
    change.variant_edit = true;
    let p = f
        .worker
        .update_encounter_participant(&e.encounter_key, change)
        .unwrap();
    assert_eq!((p.max_hp, p.current_hp), (Some(60), Some(60)));
    assert_eq!(p.hp_origin, "derived_edited");
    let detail = add("Unknown adjustment");
    let p = detail
        .participants
        .iter()
        .find(|p| p.display_name == "Unknown adjustment")
        .unwrap();
    assert_eq!((p.max_hp, p.current_hp), (None, None));
    assert_eq!(p.variant_origin, "inherited_unknown");
    assert!(
        p.stat_block
            .as_ref()
            .unwrap()
            .unapplied_effects
            .iter()
            .any(|n| n.source == "npc-adjustment")
    );
    let mut unrelated = update(p);
    unrelated.display_name = "Renamed".into();
    let p = f
        .worker
        .update_encounter_participant(&e.encounter_key, unrelated)
        .unwrap();
    assert_eq!((p.max_hp, p.current_hp), (None, None));
    assert_eq!(p.variant_origin, "inherited_unknown");
    let mut explicit = update(&p);
    explicit.variant_edit = true;
    let p = f
        .worker
        .update_encounter_participant(&e.encounter_key, explicit)
        .unwrap();
    assert_eq!(p.max_hp, Some(60));
    assert_eq!(p.current_hp, None);
    assert_eq!(p.variant_origin, "explicit");
    let detail = add("Fractional HP");
    let p = detail
        .participants
        .iter()
        .find(|p| p.display_name == "Fractional HP")
        .unwrap();
    assert_eq!((p.max_hp, p.current_hp), (None, None));
}

#[test]
fn conditions_stack_known_penalties_and_preserve_unapplied_context() {
    let f = encounter_fixture_worker();
    let e = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Conditions".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let detail = f
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: e.encounter_key.clone(),
            record_ref: "Test Creature 1".into(),
            quantity: 1,
            initiative: None,
        })
        .unwrap();
    let key = detail.participants[0].participant_key.clone();
    for (reference, value) in [
        ("conditionitems:TBSHQspnbcqxsmjL", 2),
        ("conditionitems:fesd1n5eVhpCSS18", 1),
    ] {
        f.worker
            .add_encounter_participant_condition(
                &e.encounter_key,
                AddEncounterParticipantConditionRequest {
                    participant_key: key.clone(),
                    condition_ref: Some(reference.into()),
                    name: None,
                    value: Some(value),
                    source_participant_key: None,
                    duration_rounds: None,
                    note: None,
                },
            )
            .unwrap();
    }
    let detail = f.worker.encounter(&e.encounter_key).unwrap();
    let block = detail.participants[0].stat_block.as_ref().unwrap();
    let ac = block.values.iter().find(|v| v.target == "ac").unwrap();
    assert_eq!(ac.adjusted_value, serde_json::Number::from(20));
    assert_eq!(ac.suppressed_modifiers.len(), 1);
    assert_eq!(
        block
            .activities
            .iter()
            .find(|a| a.label == "Claw")
            .unwrap()
            .rolls[0]
            .adjusted_value,
        serde_json::Number::from(10)
    );
    assert!(
        block
            .activities
            .iter()
            .any(|a| a.label == "Breath Weapon" && a.rolls.is_empty() && a.damage.is_empty())
    );
}

#[test]
fn movement_restrictions_do_not_compound_across_sources_or_depend_on_order() {
    use crate::{executor::RetrievalExecutor, test_support::fixture_worker_with_executor};
    use atlas_search::test_support::{open_source_fixture, record};
    use serde_json::{Number, json};
    let f = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        || {
            Ok(open_source_fixture([30i64, i64::MAX,4,0].into_iter().enumerate().map(|(i,speed)| record("actors","Actor",json!({"_id":format!("speed{i:011}"),"type":"npc","name":format!("Speed {speed}"),"system":{"attributes":{"adjustment":null,"speed":{"value":speed,"otherSpeeds":[]}}}}))).collect(),false)?)
        },
    ));
    for baseline in [30i64, i64::MAX, 4, 0] {
        for immobilized_order in [None, Some(0), Some(2)] {
            let encounter = f
                .worker
                .create_encounter(CreateEncounterRequest {
                    name: format!("Movement {baseline} {immobilized_order:?}"),
                    description: None,
                    note: None,
                })
                .unwrap()
                .encounter;
            let detail = f
                .worker
                .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
                    encounter_ref: encounter.encounter_key.clone(),
                    record_ref: format!("Speed {baseline}"),
                    quantity: 3,
                    initiative: None,
                })
                .unwrap();
            let participants = detail.participants;
            let mut conditions = vec![
                "conditionitems:D5mg6Tc7Jzrj6ro7",
                "conditionitems:D5mg6Tc7Jzrj6ro7",
            ];
            if let Some(position) = immobilized_order {
                conditions.insert(position, "conditionitems:eIcWbB5o3pP6OIMe");
            }
            for (i, condition) in conditions.iter().enumerate() {
                f.worker
                    .add_encounter_participant_condition(
                        &encounter.encounter_key,
                        AddEncounterParticipantConditionRequest {
                            participant_key: participants[0].participant_key.clone(),
                            condition_ref: Some((*condition).into()),
                            name: None,
                            value: None,
                            source_participant_key: Some(
                                participants[1 + i % 2].participant_key.clone(),
                            ),
                            duration_rounds: None,
                            note: None,
                        },
                    )
                    .unwrap();
            }
            let detail = f.worker.encounter(&encounter.encounter_key).unwrap();
            let participant = &detail.participants[0];
            let block = participant.stat_block.as_ref().unwrap();
            let speed = &block.speeds[0];
            let expected = if immobilized_order.is_some() {
                0
            } else if baseline > 0 {
                (baseline - 10).max(5)
            } else {
                baseline
            };
            assert_eq!(speed.base_value_feet, Number::from(baseline));
            assert_eq!(
                speed.adjusted_value_feet,
                Number::from(expected),
                "{baseline} {immobilized_order:?}"
            );
            assert_eq!(speed.notes.len(), 1);
            assert_eq!(
                participant
                    .conditions
                    .iter()
                    .filter(
                        |c| c.condition_key.as_deref() == Some("conditionitems:D5mg6Tc7Jzrj6ro7")
                    )
                    .count(),
                2
            );
        }
    }
}

#[test]
fn artifact_rebuild_does_not_reinitialize_capacity_or_current_hp() {
    use crate::{executor::RetrievalExecutor, test_support::fixture_worker_with_executor};
    use atlas_search::test_support::{open_source_fixture, record};
    use serde_json::json;
    let before = encounter_fixture_worker();
    let e = before
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Rebuild".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let d = before
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: e.encounter_key.clone(),
            record_ref: "Test Creature 1".into(),
            quantity: 1,
            initiative: None,
        })
        .unwrap();
    let mut change = update(&d.participants[0]);
    change.current_hp = Some(17);
    change.hp_edit = true;
    change.temporary_hp = 4;
    before
        .worker
        .update_encounter_participant(&e.encounter_key, change)
        .unwrap();
    let executor = RetrievalExecutor::from_test_fixture_factory(1, 16, || {
        Ok(open_source_fixture(
            vec![record(
                "actors",
                "Actor",
                json!({"_id":"testCreature1000","type":"npc","name":"Updated creature","system":{"details":{"level":{"value":6}},"attributes":{"adjustment":null,"hp":{"max":120},"ac":{"value":24}}}}),
            )],
            false,
        )?)
    });
    let mut after = fixture_worker_with_executor(executor);
    after.worker.local_state_path = before.worker.local_state_path.clone();
    let d = after.worker.encounter(&e.encounter_key).unwrap();
    let p = &d.participants[0];
    assert_eq!(
        (p.max_hp, p.current_hp, p.temporary_hp),
        (Some(60), Some(17), 4)
    );
    assert_eq!(p.hp_origin, "derived_edited");
    assert_eq!(p.record.as_ref().unwrap().title, "Updated creature");
    assert_eq!(
        p.stat_block
            .as_ref()
            .unwrap()
            .values
            .iter()
            .find(|v| v.target == "hp.max")
            .unwrap()
            .base_value,
        serde_json::Number::from(120)
    );
}

#[test]
fn hazard_uses_authored_hp_hardness_and_stealth_without_npc_adjustments() {
    let f = encounter_fixture_worker();
    let e = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Hazard".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let d = f
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: e.encounter_key.clone(),
            record_ref: "Test Hazard 1".into(),
            quantity: 1,
            initiative: None,
        })
        .unwrap();
    let p = &d.participants[0];
    assert_eq!((p.max_hp, p.current_hp), (Some(40), Some(40)));
    let block = p.stat_block.as_ref().unwrap();
    assert_eq!(
        block
            .values
            .iter()
            .find(|v| v.target == "hazard.hardness")
            .unwrap()
            .base_value,
        serde_json::Number::from(8)
    );
    assert_eq!(
        block
            .values
            .iter()
            .find(|v| v.target == "hazard.stealth")
            .unwrap()
            .base_value,
        serde_json::Number::from(12)
    );
    assert!(
        !block
            .unapplied_effects
            .iter()
            .any(|n| n.source == "npc-adjustment")
    );
    let mut change = update(p);
    change.participant_variant = EncounterParticipantVariantView::Elite;
    change.variant_edit = true;
    let p = f
        .worker
        .update_encounter_participant(&e.encounter_key, change)
        .unwrap();
    assert_eq!((p.max_hp, p.current_hp), (Some(40), Some(40)));
    assert_eq!(
        p.stat_block
            .as_ref()
            .unwrap()
            .values
            .iter()
            .find(|v| v.target == "ac")
            .unwrap()
            .adjusted_value,
        serde_json::Number::from(18)
    );
}

#[test]
fn authored_lore_variants_and_nondamaging_casting_entries_remain_context() {
    let input = atlas_search::test_support::record(
        "actors",
        "Actor",
        serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","items":[{"_id":"bbbbbbbbbbbbbbbb","type":"lore","name":"Sailing Lore","system":{"mod":{"value":14},"variants":{"storm":{"label":"Storm navigation","options":"Only while navigating a storm"}}}},{"_id":"cccccccccccccccc","type":"spellcastingEntry","name":"Arcane Spells","system":{"description":{"value":"<p>Cast authored spells</p>"}}}]}),
    );
    let block = super::mechanics::record_stat_block(&input.record, "fixture-fingerprint").unwrap();
    let lore = block
        .values
        .iter()
        .find(|v| v.label == "Sailing Lore")
        .unwrap();
    assert_eq!(lore.base_value, serde_json::Number::from(14));
    assert!(
        block
            .unapplied_effects
            .iter()
            .any(|n| n.label == "Storm navigation" && n.reason == "Only while navigating a storm")
    );
    let casting = block
        .activities
        .iter()
        .find(|a| a.label == "Arcane Spells")
        .unwrap();
    assert!(casting.rolls.is_empty() && casting.damage.is_empty());
    assert_eq!(casting.navigation.owners.len(), 1);
}

#[test]
fn nondamaging_owned_effects_keep_checked_navigation() {
    let input = atlas_search::test_support::record(
        "actors",
        "Actor",
        serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","items":[{"type":"effect","name":"Aura","system":{"description":{"value":"<p>Authored effect</p>"}}}]}),
    );
    let block = super::mechanics::record_stat_block(&input.record, "fixture-fingerprint").unwrap();
    let effect = &block.activities[0];
    assert_eq!(effect.label, "Aura");
    assert!(effect.rolls.is_empty() && effect.damage.is_empty());
    assert_eq!(
        effect.navigation.source_fingerprint.as_deref(),
        Some("fixture-fingerprint")
    );
}

#[test]
fn omitted_adjustment_under_known_attributes_defaults_local_hp_without_rewriting_source() {
    use atlas_local_state::{ParticipantVariant, ParticipantVariantOrigin};
    use atlas_record::source_record::{SourceFieldView, SourceQueryView};
    for (attributes, origin, max) in [
        (
            serde_json::json!({"hp":{"max":20}}),
            ParticipantVariantOrigin::DefaultUnadjusted,
            Some(20),
        ),
        (
            serde_json::json!({"adjustment":"bad","hp":{"max":20}}),
            ParticipantVariantOrigin::InheritedUnknown,
            None,
        ),
        (
            serde_json::Value::Null,
            ParticipantVariantOrigin::InheritedUnknown,
            None,
        ),
    ] {
        let input = atlas_search::test_support::record(
            "actors",
            "Actor",
            serde_json::json!({"_id":"LHHgGSs0ELCR4CYK","type":"npc","name":"Ghoul","system":{"attributes":attributes}}),
        );
        let (variant, actual_origin) = super::mechanics::initial_variant(&input.record);
        assert_eq!(actual_origin, origin);
        assert_eq!(super::mechanics::effective_max(&input.record, variant), max);
        if origin == ParticipantVariantOrigin::DefaultUnadjusted {
            assert_eq!(variant, Some(ParticipantVariant::Normal));
            assert!(matches!(
                SourceQueryView::new(input.record.source(), "actors", "")
                    .actor()
                    .authored_adjustment(),
                SourceFieldView::Missing
            ));
        }
    }
    let executor = crate::executor::RetrievalExecutor::from_test_fixture_factory(1, 16, || {
        Ok(atlas_search::test_support::open_source_fixture(
            vec![atlas_search::test_support::record(
                "actors",
                "Actor",
                serde_json::json!({"_id":"LHHgGSs0ELCR4CYK","type":"npc","name":"Ghoul","system":{"attributes":{"hp":{"max":20}}}}),
            )],
            false,
        )?)
    });
    let f = crate::test_support::fixture_worker_with_executor(executor);
    let e = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Default adjustment".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let d = f
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: e.encounter_key,
            record_ref: "Ghoul".into(),
            quantity: 1,
            initiative: None,
        })
        .unwrap();
    assert_eq!(
        (d.participants[0].max_hp, d.participants[0].current_hp),
        (Some(20), Some(20))
    );
    assert_eq!(d.participants[0].variant_origin, "default_unadjusted");
}

#[test]
#[ignore = "requires the pinned source corpus; run explicitly for cutover evidence"]
fn corpus_npc_adjustment_defaults_preserve_typed_ghoul_hp() {
    use atlas_foundry_model::{SourceContext, admit_document_source};
    use atlas_local_state::ParticipantVariantOrigin;
    use atlas_record::source_record::SourceBackedRecord;
    let current = std::env::current_dir().unwrap();
    let source = current
        .ancestors()
        .find_map(|p| {
            let path = p.join("scratch/source-contracts-full/pf2e/packs");
            path.is_dir().then_some(path)
        })
        .expect("pinned corpus path available in checkout ancestor");
    let mut pending = vec![source.clone()];
    let (mut total, mut defaults, mut inherited, mut unknown) = (0, 0, 0, 0);
    let mut ghoul = None;
    while let Some(path) = pending.pop() {
        if path.is_dir() {
            pending.extend(std::fs::read_dir(path).unwrap().map(|e| e.unwrap().path()));
            continue;
        }
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        let bytes = std::fs::read(&path).unwrap();
        let raw: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        if raw["type"] != "npc" {
            continue;
        }
        let pack = path
            .strip_prefix(&source)
            .unwrap()
            .components()
            .next()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap();
        let admitted = admit_document_source(
            "Actor",
            SourceContext::new(pack, path.to_str().unwrap(), "$"),
            &bytes,
        )
        .unwrap()
        .model
        .unwrap();
        let record = SourceBackedRecord::new(pack, admitted).unwrap();
        let (variant, origin) = super::mechanics::initial_variant(&record);
        let max = super::mechanics::effective_max(&record, variant);
        total += 1;
        match origin {
            ParticipantVariantOrigin::DefaultUnadjusted => {
                defaults += 1;
                assert!(max.is_some(), "{}", record.key());
            }
            ParticipantVariantOrigin::InheritedKnown => inherited += 1,
            ParticipantVariantOrigin::InheritedUnknown => unknown += 1,
            ParticipantVariantOrigin::Explicit => panic!("source initialization is never explicit"),
        }
        if record.key().to_string() == "pathfinder-bestiary:LHHgGSs0ELCR4CYK" {
            ghoul = Some((origin, max));
        }
    }
    assert_eq!((total, defaults, inherited, unknown), (5492, 5237, 252, 3));
    assert_eq!(
        ghoul,
        Some((ParticipantVariantOrigin::DefaultUnadjusted, Some(20)))
    );
    eprintln!(
        "NPC defaults: roots={total} default_unadjusted={defaults} inherited={inherited} unknown={unknown}; actual Ghoul HP=20"
    );
}
