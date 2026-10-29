use super::conditions::ConditionRule;
use crate::{
    executor::RetrievalExecutor,
    test_support::{FixtureWorker, fixture_worker_with_executor},
};
use atlas_app_model::*;
use atlas_search::test_support::{open_source_fixture, record};
use serde_json::json;

fn fixture(items: serde_json::Value) -> (FixtureWorker, String, String) {
    let f = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        move || {
            let mut items = items.as_array().unwrap().clone();
            items.push(json!({"_id":"claw000000000000","type":"melee","name":"Claw","system":{"range":null,"traits":{"value":[]},"bonus":{"value":12},"damageRolls":{"main":{"damage":"1d6+4","damageType":"slashing"}},"description":{"value":"<p>Claw</p>"}}}));
            let mut records = vec![record(
                "actors",
                "Actor",
                json!({
                    "_id":"aaaaaaaaaaaaaaaa","type":"npc","name":"Effective sources",
                    "system":{"traits":{"value":[]},"details":{"level":{"value":5}},
                        "attributes":{"adjustment":null,"hp":{"max":60},"ac":{"value":20},"immunities":[],"weaknesses":[],"resistances":[]},
                        "perception":{"mod":10},"abilities":{"con":{"mod":2}}}, "items":items
                }),
            )];
            for definition in
                atlas_foundry_model::generated::condition_definitions::CONDITION_DEFINITIONS
            {
                records.push(record("conditionitems", "Item", json!({
                "_id":definition.record_key.split_once(':').unwrap().1,"type":"condition","name":definition.name,
                "system":{"slug":definition.slug,"value":{"isValued":definition.has_value,"value":definition.default_value}}
            })));
            }
            Ok(open_source_fixture(records, false)?)
        },
    ));
    let key = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Effective sources".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter
        .encounter_key;
    f.worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: key.clone(),
            record_ref: "actors:aaaaaaaaaaaaaaaa".into(),
            quantity: 2,
            initiative: Some(20),
            participant_variant: None,
        })
        .unwrap();
    let p = f.worker.encounter(&key).unwrap().participants[0]
        .participant_key
        .clone();
    (f, key, p)
}

fn grant(
    rule: ConditionRule,
    value: i64,
    predicate: serde_json::Value,
    toggle: bool,
) -> serde_json::Value {
    let mut rules = vec![];
    if toggle {
        rules.push(json!({"key":"RollOption","option":"maintained","toggleable":true}));
    }
    rules.push(json!({"key":"GrantItem","inMemoryOnly":true,"uuid":format!("Compendium.pf2e.conditionitems.Item.{}", rule.definition().record_key.split_once(':').unwrap().1),
        "predicate":predicate,"alterations":[{"mode":"override","property":"badge-value","value":value}]}));
    json!([{"_id":"bbbbbbbbbbbbbbbb","type":"action","name":"Maintained condition","system":{"actionType":{"value":"passive"},"rules":rules}}])
}

fn track(f: &FixtureWorker, key: &str, p: &str) {
    f.worker
        .set_encounter_life_settings(
            key,
            p,
            SetEncounterLifeSettingsRequest {
                request_id: "track".into(),
                expected_encounter_revision: f.worker.encounter(key).unwrap().revision,
                track_dying: true,
                mortality: None,
                knockout_context: None,
                level_override: None,
                base_death_threshold: None,
            },
        )
        .unwrap();
}
fn correct_hp(f: &FixtureWorker, key: &str, p: &str, hp: i64) {
    let before = f.worker.encounter(key).unwrap();
    f.worker
        .apply_encounter_hp_action(
            key,
            ApplyEncounterHpActionRequest {
                request_id: crate::test_support::next_request_id(),
                participant_key: p.into(),
                expected_revision: before
                    .participants
                    .iter()
                    .find(|row| row.participant_key == p)
                    .unwrap()
                    .revision,
                context: None,
                action: EncounterHpAction::SetCurrent {
                    input: Some(hp.to_string()),
                },
            },
        )
        .unwrap();
}
fn add(
    f: &FixtureWorker,
    key: &str,
    p: &str,
    rule: ConditionRule,
    value: Option<i64>,
    intent: ConditionEventIntentView,
) {
    f.worker
        .add_encounter_participant_condition(
            key,
            AddEncounterParticipantConditionRequest {
                request_id: crate::test_support::next_request_id(),
                expected_encounter_revision: f.worker.encounter(key).unwrap().revision,
                event_intent: intent,
                decisions: Default::default(),
                participant_key: p.into(),
                condition_ref: Some(rule.definition().record_key.into()),
                name: None,
                value,
                source_participant_key: None,
                scope: Default::default(),
                extra_action: Default::default(),
                immunity_override: false,
                parameters: Default::default(),
                origin: None,
                duration: None,
                note: None,
            },
        )
        .unwrap();
}
fn toggle_request(
    f: &FixtureWorker,
    key: &str,
    p: &str,
    selection: Option<String>,
) -> UpdateEncounterAbilityStateRequest {
    let current = f.worker.encounter(key).unwrap();
    let participant = current
        .participants
        .iter()
        .find(|row| row.participant_key == p)
        .unwrap();
    let ability = &participant.ability_states.as_ref().unwrap()[0];
    UpdateEncounterAbilityStateRequest {
        request_id: crate::test_support::next_request_id(),
        expected_encounter_revision: current.revision,
        expected_participant_revision: participant.revision,
        participant_key: p.into(),
        navigation: ability.navigation.clone(),
        rule_ordinal: ability.rule_ordinal,
        action: EncounterAbilityStateAction::SetOption {
            active: true,
            selection,
        },
    }
}

#[test]
fn recovery_reads_inherited_doomed_and_retry_recovers_the_committed_result() {
    let (f, key, p) = fixture(grant(ConditionRule::Doomed, 2, json!([]), false));
    track(&f, &key, &p);
    correct_hp(&f, &key, &p, 0);
    add(
        &f,
        &key,
        &p,
        ConditionRule::Dying,
        Some(1),
        ConditionEventIntentView::Correction,
    );
    let request = ApplyConditionLifecycleRequest {
        boundary_key: None,
        request_id: "recover".into(),
        expected_encounter_revision: f.worker.encounter(&key).unwrap().revision,
        participant_key: p.clone(),
        action: ConditionLifecycleActionView::DyingRecovery {
            check: LifecycleCheckInputView::TableDegree {
                degree: CheckDegreeView::Failure,
            },
        },
    };
    let result = f
        .worker
        .apply_condition_lifecycle(&key, request.clone())
        .unwrap();
    let after = f.worker.encounter(&key).unwrap();
    assert_eq!(after.participants[0].mortality, MortalityStateView::Dead);
    assert!(after.participants[0].conditions.is_empty());
    assert_eq!(
        f.worker.apply_condition_lifecycle(&key, request).unwrap(),
        result
    );
    assert_eq!(f.worker.encounter(&key).unwrap(), after);
}

#[test]
fn uncertain_recovery_rejects_before_writing_but_hp_correction_stays_available() {
    let (f, key, p) = fixture(grant(
        ConditionRule::Doomed,
        2,
        json!(["self:environment:unknown"]),
        false,
    ));
    track(&f, &key, &p);
    correct_hp(&f, &key, &p, 0);
    add(
        &f,
        &key,
        &p,
        ConditionRule::Dying,
        Some(1),
        ConditionEventIntentView::Correction,
    );
    let before = f.worker.encounter(&key).unwrap();
    let request = ApplyConditionLifecycleRequest {
        boundary_key: None,
        request_id: "uncertain".into(),
        expected_encounter_revision: before.revision,
        participant_key: p.clone(),
        action: ConditionLifecycleActionView::DyingRecovery {
            check: LifecycleCheckInputView::Roll,
        },
    };
    let error = f
        .worker
        .apply_condition_lifecycle(&key, request.clone())
        .unwrap_err()
        .into_app_error();
    assert!(error.message.contains("Doomed") && error.message.contains("Maintained condition"));
    assert_eq!(f.worker.encounter(&key).unwrap(), before);
    f.worker
        .with_state(|session| {
            let intent = serde_json::json!({"action":"condition_lifecycle","request":request});
            assert!(
                session
                    .local_state()
                    .encounters()
                    .condition_event_receipt(&key, "uncertain", &intent)?
                    .is_none()
            );
            Ok(())
        })
        .unwrap();
    correct_hp(&f, &key, &p, 7);
    assert_eq!(
        f.worker.encounter(&key).unwrap().participants[0].current_hp,
        Some(7)
    );
}

#[test]
fn activating_doomed_checks_mortality_without_saving_an_inherited_condition_row() {
    let (f, key, p) = fixture(grant(ConditionRule::Doomed, 2, json!(["maintained"]), true));
    track(&f, &key, &p);
    correct_hp(&f, &key, &p, 0);
    add(
        &f,
        &key,
        &p,
        ConditionRule::Dying,
        Some(2),
        ConditionEventIntentView::Correction,
    );
    let request = toggle_request(&f, &key, &p, None);
    let receipt = f
        .worker
        .update_encounter_ability_state(&key, request.clone())
        .unwrap();
    let after = f.worker.encounter(&key).unwrap();
    assert_eq!(after.participants[0].mortality, MortalityStateView::Dead);
    assert!(after.participants[0].conditions.is_empty());
    assert_eq!(
        f.worker
            .update_encounter_ability_state(&key, request)
            .unwrap(),
        receipt
    );
    assert_eq!(f.worker.encounter(&key).unwrap(), after);
}

#[test]
fn activating_stunned_deducts_current_stock_and_preserves_its_authored_cause() {
    let (f, key, p) = fixture(grant(
        ConditionRule::Stunned,
        2,
        json!(["maintained"]),
        true,
    ));
    f.worker
        .advance_encounter_turn(
            &key,
            AdvanceEncounterTurnRequest {
                request_id: "start".into(),
                expected_revision: f.worker.encounter(&key).unwrap().revision,
            },
        )
        .unwrap();
    assert_eq!(
        f.worker
            .encounter(&key)
            .unwrap()
            .current_turn_participant_key
            .as_deref(),
        Some(p.as_str())
    );
    let request = toggle_request(&f, &key, &p, None);
    f.worker
        .update_encounter_ability_state(&key, request.clone())
        .unwrap();
    let after = f.worker.encounter(&key).unwrap();
    assert_eq!(
        after.participants[0]
            .turn_resources
            .as_ref()
            .unwrap()
            .regular_remaining,
        Some(1)
    );
    assert!(after.participants[0].conditions.is_empty());
    f.worker
        .update_encounter_ability_state(&key, request)
        .unwrap();
    assert_eq!(f.worker.encounter(&key).unwrap(), after);
}

#[test]
fn supported_source_detection_options_reach_strike_context_and_contextual_ac() {
    for (items, selection, dc, off_guard) in [
        (
            json!([{"_id":"USCoVo5LJNs12zB4","name":"Go Dark","type":"action","system":{"actionType":{"value":"action"},"rules":[
                {"key":"RollOption","option":"go-dark","suboptions":[{"label":"PF2E.ConditionTypeInvisible","value":"Compendium.pf2e.conditionitems.Item.Invisible"},{"label":"PF2E.ConditionTypeHidden","value":"Compendium.pf2e.conditionitems.Item.Hidden"}],"toggleable":true},
                {"inMemoryOnly":true,"key":"GrantItem","predicate":["go-dark"],"uuid":"{item|flags.system.rulesSelections.goDark}"}
            ]}}]),
            Some("Compendium.pf2e.conditionitems.Item.Hidden".into()),
            11,
            true,
        ),
        (
            json!([{"_id":"a5hbLlCGgZWkEv4t","name":"Smoke Exhalation","type":"action","system":{"actionType":{"value":"action"},"rules":[
                {"key":"RollOption","option":"smoke-exhalation","toggleable":true},
                {"inMemoryOnly":true,"key":"GrantItem","predicate":["smoke-exhalation"],"uuid":"Compendium.pf2e.conditionitems.Item.Concealed"}
            ]}}]),
            None,
            5,
            false,
        ),
    ] {
        let (f, key, p) = fixture(items);
        let request = toggle_request(&f, &key, &p, selection);
        f.worker
            .update_encounter_ability_state(&key, request)
            .unwrap();
        let current = f.worker.encounter(&key).unwrap();
        let observer = &current.participants[1];
        let RecordBodyView::Creature(body) = &observer.presentation.as_ref().unwrap().body else {
            panic!("creature");
        };
        let strike = body
            .activities
            .value
            .as_ref()
            .unwrap()
            .iter()
            .find(|activity| activity.title == "Claw")
            .unwrap();
        let context = f
            .worker
            .preview_encounter_strike_context(
                &key,
                EncounterStrikeContextRequest {
                    strike: EncounterStrikeOptionsRequest {
                        participant_key: observer.participant_key.clone(),
                        reference: EncounterReferenceRequest {
                            condition_id: None,
                            artifact_generation: current.artifact_generation.clone().unwrap(),
                            selection: RecordDetailRequest {
                                record_key: strike.navigation.record_key.clone(),
                                owners: strike.navigation.owners.clone(),
                                source_fingerprint: strike.navigation.source_fingerprint.clone(),
                                spell_rank: None,
                                fields: vec![],
                                passage: None,
                            },
                        },
                    },
                    target_key: Some(p.clone()),
                },
            )
            .unwrap();
        assert_eq!(context.targeting_check.unwrap().dc, dc);
        assert!(!context.action.targeting.unwrap().check_senses);
        let contextual = &body.runtime.as_ref().unwrap().contextual_armor_class;
        assert_eq!(
            contextual.iter().any(|entry| entry.opponent_key == p
                && entry
                    .value
                    .value
                    .as_ref()
                    .and_then(serde_json::Number::as_i64)
                    == Some(18)),
            off_guard
        );
        assert!(current.participants[0].conditions.is_empty());
        assert_eq!(f.worker.encounter(&key).unwrap(), current);
    }
}

#[test]
fn explicit_and_generated_conditions_activate_existing_source_mortality_rules() {
    for (predicate, applied, value) in [
        (
            "self:condition:frightened",
            ConditionRule::Frightened,
            Some(1),
        ),
        ("self:condition:prone", ConditionRule::Unconscious, None),
    ] {
        let (f, key, p) = fixture(grant(
            ConditionRule::Doomed,
            if applied == ConditionRule::Unconscious {
                4
            } else {
                2
            },
            json!([predicate]),
            false,
        ));
        track(&f, &key, &p);
        correct_hp(&f, &key, &p, 0);
        if applied != ConditionRule::Unconscious {
            add(
                &f,
                &key,
                &p,
                ConditionRule::Dying,
                Some(2),
                ConditionEventIntentView::Correction,
            );
        }
        add(
            &f,
            &key,
            &p,
            applied,
            value,
            ConditionEventIntentView::Apply,
        );
        let after = f.worker.encounter(&key).unwrap();
        assert_eq!(
            after.participants[0].mortality,
            MortalityStateView::Dead,
            "{predicate}"
        );
        assert!(
            !after.participants[0]
                .conditions
                .iter()
                .any(|row| row.condition_key.as_deref()
                    == Some(ConditionRule::Doomed.definition().record_key))
        );
    }
}

#[test]
fn removing_a_condition_can_activate_source_stunned_on_the_current_turn() {
    let (f, key, p) = fixture(grant(
        ConditionRule::Stunned,
        2,
        json!([{"not":"self:condition:frightened"}]),
        false,
    ));
    add(
        &f,
        &key,
        &p,
        ConditionRule::Frightened,
        Some(1),
        ConditionEventIntentView::Correction,
    );
    f.worker
        .advance_encounter_turn(
            &key,
            AdvanceEncounterTurnRequest {
                request_id: "start-remove".into(),
                expected_revision: f.worker.encounter(&key).unwrap().revision,
            },
        )
        .unwrap();
    let before = f.worker.encounter(&key).unwrap();
    assert_eq!(
        before.participants[0]
            .turn_resources
            .as_ref()
            .unwrap()
            .regular_remaining,
        Some(3)
    );
    let condition_id = before.participants[0].conditions[0].condition_id;
    let request = RemoveEncounterParticipantConditionRequest {
        request_id: "remove-trigger".into(),
        expected_encounter_revision: before.revision,
        intent: ConditionRemovalIntentView::RemoveEffect,
    };
    f.worker
        .remove_encounter_participant_condition(&key, &p, condition_id, request.clone())
        .unwrap();
    let after = f.worker.encounter(&key).unwrap();
    assert_eq!(
        after.participants[0]
            .turn_resources
            .as_ref()
            .unwrap()
            .regular_remaining,
        Some(1)
    );
    assert!(after.participants[0].conditions.is_empty());
    f.worker
        .remove_encounter_participant_condition(&key, &p, condition_id, request)
        .unwrap();
    assert_eq!(f.worker.encounter(&key).unwrap(), after);
}
