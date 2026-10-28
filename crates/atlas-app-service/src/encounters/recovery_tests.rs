use super::{hydration, rules};
use crate::{executor::RetrievalExecutor, test_support::fixture_worker_with_executor};
use atlas_app_model::*;
use atlas_local_state::{AbilityStateValue, ParticipantAbilityState};
use atlas_search::test_support::{open_source_fixture, record};
use serde_json::json;

#[test]
fn overnight_calculates_source_healing_and_restores_daily_but_preserves_weekly_uses() {
    let source = json!({
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
                hydration::hydrate_participant_records(session.retrieval_mut(), &participants)
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
