use super::{conditions::ConditionRule, hydration, rules};
use crate::{
    executor::RetrievalExecutor,
    test_support::{FixtureWorker, fixture_worker_with_executor},
};
use atlas_app_model::*;
use atlas_local_state::{AbilityStateValue, ParticipantAbilityState};
use atlas_search::test_support::{open_source_fixture, record};
use serde_json::json;

fn resting_creature(extra_items: Vec<serde_json::Value>) -> (FixtureWorker, EncounterSummaryView) {
    let mut source = json!({
        "_id":"aaaaaaaaaaaaaaaa","type":"npc","name":"Resting creature",
        "system":{
            "details":{"level":{"value":5}},
            "attributes":{"hp":{"max":60}},
            "abilities":{"con":{"mod":2}}
        },
        "items":[
            {"_id":"bbbbbbbbbbbbbbbb","type":"action","name":"Daily ward",
             "system":{"frequency":{"max":3,"value":0,"per":"day"}}},
            {"_id":"cccccccccccccccc","type":"action","name":"Weekly ward",
             "system":{"frequency":{"max":2,"value":0,"per":"P1W"}}}
        ]
    });
    source["items"].as_array_mut().unwrap().extend(extra_items);
    let fixture = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        move || {
            Ok(open_source_fixture(
                vec![record("actors", "Actor", source.clone())],
                false,
            )?)
        },
    ));
    let worker = &fixture.worker;
    let encounter = worker
        .create_encounter(CreateEncounterRequest {
            name: "Overnight rules".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            participant_variant: None,
            encounter_ref: encounter.encounter_key.clone(),
            record_ref: "actors:aaaaaaaaaaaaaaaa".into(),
            quantity: 1,
            initiative: Some(12),
        })
        .unwrap();
    (fixture, encounter)
}

#[test]
fn overnight_calculates_source_healing_and_restores_daily_but_preserves_weekly_uses() {
    let (fixture, encounter) = resting_creature(vec![]);
    let worker = &fixture.worker;
    let mut participant = worker
        .with_state(|session| {
            Ok(session
                .local_state()
                .encounters()
                .get_with_participants(&encounter.encounter_key)?
                .unwrap()
                .participants
                .remove(0))
        })
        .unwrap();
    participant.current_hp = Some(5);
    participant.temporary_hp = 7;
    let participants = vec![participant.clone()];
    let (_, records) = worker
        .with_workspace(
            &crate::ReadExecutionContext::uncancellable(),
            move |session| {
                hydration::prepare_participant_records(session.retrieval_mut(), &participants)
            },
        )
        .unwrap();
    let record = &records["actors:aaaaaaaaaaaaaaaa"];
    let sources = &record
        .arithmetic_baseline
        .as_ref()
        .unwrap()
        .source_conditions;
    assert_eq!(sources.uses.len(), 2);
    for ability in &sources.uses {
        participant.ability_states.push(ParticipantAbilityState {
            owner: ability.owner.clone(),
            rule_ordinal: None,
            declaration_identity: Some(ability.declaration_identity.clone()),
            source_fingerprint: sources.source_fingerprint.clone().unwrap(),
            value: AbilityStateValue::Uses { remaining: 0 },
        });
    }
    let mut row = EncounterRecoveryParticipantView {
        participant_key: participant.participant_key.clone(),
        display_name: participant.display_name.clone(),
        current_hp: participant.current_hp,
        resulting_hp: participant.current_hp,
        resulting_max_hp: participant.max_hp,
        healing: None,
        changes: vec![],
        unavailable_reason: None,
    };
    let state = rules::overnight::prepare(
        &participant,
        None,
        OvernightHealingMode::Natural,
        Some(record),
        &mut row,
        6,
    )
    .unwrap()
    .unwrap();
    assert!(row.unavailable_reason.is_none());
    assert_eq!(row.healing, Some(10));
    assert_eq!(state.current_hp, Some(15));
    assert_eq!(state.temporary_hp, 0);
    assert_eq!(state.initiative, Some(12));
    for (label, expected) in [("Daily ward", 3), ("Weekly ward", 0)] {
        let ability = sources.uses.iter().find(|a| a.label == label).unwrap();
        let state = state
            .ability_states
            .iter()
            .find(|a| a.owner == ability.owner && a.rule_ordinal.is_none())
            .unwrap();
        assert_eq!(
            state.value,
            AbilityStateValue::Uses {
                remaining: expected
            }
        );
    }
}

fn add_condition(
    fixture: &FixtureWorker,
    participant: &str,
    rule: ConditionRule,
    value: Option<i64>,
) {
    let definition = rule.definition();
    fixture
        .worker
        .with_write(|session| {
            session.local_state().encounters().add_condition(
                atlas_local_state::AddEncounterParticipantCondition {
                    participant_key: participant.into(),
                    condition_key: Some(definition.record_key.into()),
                    name: definition.name.into(),
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
            )?;
            Ok(())
        })
        .unwrap();
}

fn rest_request(participant: &str, mode: OvernightHealingMode) -> EncounterRecoveryAction {
    EncounterRecoveryAction::RestOvernight {
        healing_mode: mode,
        participants: vec![OvernightParticipantInput {
            participant_key: participant.into(),
            healing: None,
        }],
    }
}

#[test]
fn overnight_uses_constitution_modifier_without_drained_check_penalties() {
    let (fixture, encounter) = resting_creature(vec![]);
    let worker = &fixture.worker;
    let participant = worker
        .encounter(&encounter.encounter_key)
        .unwrap()
        .participants
        .remove(0);
    add_condition(
        &fixture,
        &participant.participant_key,
        ConditionRule::Drained,
        Some(2),
    );
    let before = worker.encounter(&encounter.encounter_key).unwrap();
    let preview = worker
        .preview_encounter_recovery(
            &encounter.encounter_key,
            PreviewEncounterRecoveryRequest {
                expected_encounter_revision: before.revision,
                action: rest_request(&participant.participant_key, OvernightHealingMode::Natural),
            },
        )
        .unwrap();
    assert_eq!(preview.participants[0].healing, Some(10));
}

#[test]
fn overnight_rejects_authored_dying_and_petrified_without_mutating_the_encounter() {
    for rule in [ConditionRule::Dying, ConditionRule::Petrified] {
        let definition = rule.definition();
        let (fixture, encounter) = resting_creature(vec![json!({
            "_id":"dddddddddddddddd", "type":"condition", "name":definition.name,
            "system":{"slug":definition.slug,"value":{"isValued":definition.has_value,"value":definition.default_value}}
        })]);
        let worker = &fixture.worker;
        let before = worker.encounter(&encounter.encounter_key).unwrap();
        let participant = &before.participants[0];
        assert!(participant.conditions.is_empty());
        let action = rest_request(&participant.participant_key, OvernightHealingMode::Full);
        let preview = worker
            .preview_encounter_recovery(
                &encounter.encounter_key,
                PreviewEncounterRecoveryRequest {
                    expected_encounter_revision: before.revision,
                    action: action.clone(),
                },
            )
            .unwrap();
        assert!(!preview.can_apply, "{} must prevent rest", definition.name);
        assert!(preview.participants[0].unavailable_reason.is_some());
        assert!(
            worker
                .apply_encounter_recovery(
                    &encounter.encounter_key,
                    ApplyEncounterRecoveryRequest {
                        request_id: format!("rest-with-{}", definition.slug),
                        expected_encounter_revision: before.revision,
                        action,
                    }
                )
                .is_err()
        );
        assert_eq!(worker.encounter(&encounter.encounter_key).unwrap(), before);
    }
}

#[test]
fn resting_other_participants_releases_ended_turn_claims_before_resuming() {
    let (fixture, encounter) = resting_creature(vec![]);
    let worker = &fixture.worker;
    let key = &encounter.encounter_key;
    let stunned = worker.encounter(key).unwrap().participants[0]
        .participant_key
        .clone();
    worker
        .add_encounter_manual_participant(AddEncounterManualParticipantRequest {
            encounter_ref: key.clone(),
            display_name: "Resting PC".into(),
            max_hp: Some(20),
            current_hp: Some(10),
            initiative: Some(5),
        })
        .unwrap();
    let resting = worker
        .encounter(key)
        .unwrap()
        .participants
        .iter()
        .find(|p| p.participant_key != stunned)
        .unwrap()
        .participant_key
        .clone();
    add_condition(&fixture, &stunned, ConditionRule::Stunned, Some(4));
    worker
        .advance_encounter_turn(
            key,
            AdvanceEncounterTurnRequest {
                request_id: "first-stunned-turn".into(),
                expected_revision: worker.encounter(key).unwrap().revision,
            },
        )
        .unwrap();
    let first_turn = worker.encounter(key).unwrap();
    assert_eq!(
        first_turn.current_turn_participant_key.as_deref(),
        Some(stunned.as_str())
    );
    assert_eq!(
        first_turn
            .participants
            .iter()
            .find(|p| p.participant_key == stunned)
            .unwrap()
            .conditions[0]
            .value,
        Some(1)
    );
    worker
        .apply_encounter_recovery(
            key,
            ApplyEncounterRecoveryRequest {
                request_id: "rest-other-participant".into(),
                expected_encounter_revision: first_turn.revision,
                action: rest_request(&resting, OvernightHealingMode::Full),
            },
        )
        .unwrap();
    let rested = worker.encounter(key).unwrap();
    assert!(rested.current_turn_participant_key.is_none());
    assert_eq!(
        rested.encounter.round_number,
        first_turn.encounter.round_number
    );
    worker
        .advance_encounter_turn(
            key,
            AdvanceEncounterTurnRequest {
                request_id: "resume-after-rest".into(),
                expected_revision: rested.revision,
            },
        )
        .unwrap();
    let resumed = worker.encounter(key).unwrap();
    assert_eq!(
        resumed.current_turn_participant_key.as_deref(),
        Some(stunned.as_str())
    );
    let participant = resumed
        .participants
        .iter()
        .find(|p| p.participant_key == stunned)
        .unwrap();
    assert!(participant.conditions.is_empty());
    assert_eq!(
        participant
            .turn_resources
            .as_ref()
            .unwrap()
            .regular_remaining,
        Some(2)
    );
}

#[test]
fn next_turn_retry_after_reset_or_rest_recovers_before_source_and_revision_guards() {
    for reset in [false, true] {
        let executor = RetrievalExecutor::from_fixture_workers(1, 16);
        let path = executor
            .runtime
            .as_ref()
            .unwrap()
            .index_path()
            .to_path_buf();
        let fixture = fixture_worker_with_executor(executor.clone());
        let worker = &fixture.worker;
        let key = worker
            .create_encounter(CreateEncounterRequest {
                name: "Turn receipt lifetime".into(),
                description: None,
                note: None,
            })
            .unwrap()
            .encounter
            .encounter_key;
        worker
            .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
                participant_variant: None,
                encounter_ref: key.clone(),
                record_ref: "Test Creature 1".into(),
                quantity: 1,
                initiative: Some(10),
            })
            .unwrap();
        let first = AdvanceEncounterTurnRequest {
            request_id: "first-lifetime-turn".into(),
            expected_revision: worker.encounter(&key).unwrap().revision,
        };
        worker.advance_encounter_turn(&key, first.clone()).unwrap();
        worker
            .advance_encounter_turn(
                &key,
                AdvanceEncounterTurnRequest {
                    request_id: "second-lifetime-turn".into(),
                    expected_revision: worker.encounter(&key).unwrap().revision,
                },
            )
            .unwrap();
        let current = worker.encounter(&key).unwrap();
        let action = if reset {
            EncounterRecoveryAction::ResetEncounter
        } else {
            rest_request(
                &current.participants[0].participant_key,
                OvernightHealingMode::Full,
            )
        };
        worker
            .apply_encounter_recovery(
                &key,
                ApplyEncounterRecoveryRequest {
                    request_id: "lifetime-reset-or-rest".into(),
                    expected_encounter_revision: current.revision,
                    action,
                },
            )
            .unwrap();
        let saved = worker
            .with_state(|session| {
                Ok(session
                    .local_state()
                    .encounters()
                    .get_with_participants(&key)?
                    .unwrap())
            })
            .unwrap();
        let workspace = atlas_runtime::initialize_workspace(path).unwrap();
        let write = workspace
            .write(atlas_runtime::WORKSPACE_SCHEMA_VERSION)
            .unwrap();
        write
            .connection()
            .execute(
                "UPDATE workspace SET reference_revision=reference_revision+1 WHERE id=1",
                [],
            )
            .unwrap();
        write.commit().unwrap();
        executor.fail_after_submissions(0);
        assert_eq!(
            worker
                .advance_encounter_turn(&key, first.clone())
                .unwrap()
                .encounter_key,
            key
        );
        let after = worker
            .with_state(|session| {
                Ok(session
                    .local_state()
                    .encounters()
                    .get_with_participants(&key)?
                    .unwrap())
            })
            .unwrap();
        assert_eq!(after, saved);
        assert!(
            worker
                .advance_encounter_turn(
                    &key,
                    AdvanceEncounterTurnRequest {
                        expected_revision: saved.encounter.revision,
                        ..first
                    }
                )
                .is_err()
        );
    }
}
