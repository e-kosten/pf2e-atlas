use crate::{
    executor::RetrievalExecutor,
    test_support::{FixtureWorker, fixture_worker_with_executor},
};
use atlas_app_model::*;
use atlas_search::test_support::{input, open_input_fixture, record};
use serde_json::json;

fn fixture(revision: &'static str) -> FixtureWorker {
    fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        move || {
            let mut built = input(
                vec![record(
                    "actors",
                    "Actor",
                    json!({
                                "_id":"aaaaaaaaaaaaaaaa","type":"npc","name":"Guard",
                    "system":{"details":{"level":{"value":5}},"attributes":{"adjustment":null,"hp":{"max":60}},"perception":{"mod":13},"abilities":{"dex":{"mod":3}},"skills":{}},"items":[{"_id":"bbbbbbbbbbbbbbbb","type":"melee","name":"Bite","system":{"bonus":{"value":12},"damageRolls":{"main":{"damage":"5","damageType":"piercing"},"bleed":{"damage":"2","damageType":"bleed"}}}}]
                            }),
                )],
                false,
            );
            built.context.source_revision = Some(revision.into());
            let generation = built.context.preparation_sha256()?;
            for record in &mut built.records {
                record.preparation_context_hash = generation.clone();
            }
            Ok(open_input_fixture(built)?)
        },
    ))
}
fn encounter(f: &FixtureWorker) -> String {
    let key = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Initiative".into(),
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
            quantity: 1,
            initiative: None,
        })
        .unwrap();
    key
}
fn request(options: InitiativeOptionsView) -> RollInitiativeRequest {
    RollInitiativeRequest {
        request_id: "initiative-test".into(),
        artifact_generation: options.artifact_generation,
        expected_revision: options.expected_revision,
        participants: options
            .participants
            .into_iter()
            .map(|p| InitiativeParticipantRequest {
                participant_key: p.participant_key,
                expected_revision: p.expected_revision,
                statistic: "perception".into(),
                modifier_override: None,
                mode: RollMode::Normal,
            })
            .collect(),
    }
}
#[test]
fn changed_generation_rejects_without_mutation_and_saved_receipt_recovers() {
    let a = fixture("a");
    let key = encounter(&a);
    let original = request(a.worker.encounter_initiative_options(&key).unwrap());
    let b = &a;
    let mut changed = input(
        vec![record(
            "actors",
            "Actor",
            json!({
                "_id":"aaaaaaaaaaaaaaaa","type":"npc","name":"Guard","system":{"details":{"level":{"value":5}},"attributes":{"adjustment":null,"hp":{"max":60}},"perception":{"mod":13},"abilities":{"dex":{"mod":3}},"skills":{}}
            }),
        )],
        false,
    );
    changed.context.source_revision = Some("b".into());
    let hash = changed.context.preparation_sha256().unwrap();
    for record in &mut changed.records {
        record.preparation_context_hash = hash.clone();
    }
    atlas_search::test_support::publish_input(a.worker.workspace_path(), &changed).unwrap();
    let before = b
        .worker
        .with_state(|session| {
            Ok(session
                .local_state()
                .encounters()
                .get_with_participants(&key)?)
        })
        .unwrap()
        .unwrap();
    let error = b
        .worker
        .roll_encounter_initiative(&key, original.clone())
        .unwrap_err()
        .into_app_error();
    assert_eq!(error.code, AppErrorCode::ArtifactGenerationChanged);
    let after = b
        .worker
        .with_state(|session| {
            Ok(session
                .local_state()
                .encounters()
                .get_with_participants(&key)?)
        })
        .unwrap()
        .unwrap();
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(after).unwrap()
    );
    let current = request(a.worker.encounter_initiative_options(&key).unwrap());
    let saved = a
        .worker
        .roll_encounter_initiative(&key, current.clone())
        .unwrap();
    changed.context.source_revision = Some("c".into());
    let hash = changed.context.preparation_sha256().unwrap();
    for record in &mut changed.records {
        record.preparation_context_hash = hash.clone();
    }
    atlas_search::test_support::publish_input(a.worker.workspace_path(), &changed).unwrap();
    assert_eq!(
        a.worker.roll_encounter_initiative(&key, current).unwrap(),
        saved
    );
}

#[test]
fn unlisted_npc_skill_uses_ability_and_condition_before_roll() {
    let f = fixture("a");
    let key = encounter(&f);
    let initial = f.worker.encounter_initiative_options(&key).unwrap();
    let stealth = |o: &InitiativeOptionsView| {
        o.participants[0]
            .statistics
            .iter()
            .find(|s| s.key == "skill.stealth")
            .unwrap()
            .modifier
    };
    assert_eq!(stealth(&initial), 3);
    f.worker
        .with_write(|session| {
            Ok(session.local_state().encounters().add_condition(
                atlas_local_state::AddEncounterParticipantCondition {
                    participant_key: initial.participants[0].participant_key.clone(),
                    condition_key: Some("conditionitems:TBSHQspnbcqxsmjL".into()),
                    name: "Frightened".into(),
                    value: Some(2),
                    source_participant_key: None,
                    duration: None,
                    note: None,
                },
            )?)
        })
        .unwrap();
    let current = f.worker.encounter_initiative_options(&key).unwrap();
    assert_eq!(stealth(&current), 1);
    let mut input = request(current);
    input.participants[0].statistic = "skill.stealth".into();
    let result = f.worker.roll_encounter_initiative(&key, input).unwrap();
    assert_eq!(result.rolls[0].outcome.formula, "1d20+1");
}
#[test]
fn explicit_manual_modifier_does_not_require_reference_hydration() {
    let f = fixture("a");
    let key = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Manual".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter
        .encounter_key;
    f.worker
        .add_encounter_manual_participant(AddEncounterManualParticipantRequest {
            encounter_ref: key.clone(),
            display_name: "PC".into(),
            max_hp: None,
            current_hp: None,
            initiative: None,
        })
        .unwrap();
    let mut input = request(f.worker.encounter_initiative_options(&key).unwrap());
    assert!(input.artifact_generation.is_none());
    input.participants[0].modifier_override = Some(7);
    assert_eq!(
        f.worker
            .roll_encounter_initiative(&key, input)
            .unwrap()
            .rolls[0]
            .outcome
            .formula,
        "1d20+7"
    );
}

#[test]
fn implicit_bleed_is_projected_and_rolled_as_persistent() {
    let f = fixture("a");
    let key = encounter(&f);
    let detail = f.worker.encounter(&key).unwrap();
    let participant = &detail.participants[0];
    let RecordBodyView::Creature(actor) = &participant.presentation.as_ref().unwrap().body else {
        panic!("creature");
    };
    let activity = &actor.activities.value.as_ref().unwrap()[0];
    let bleed = activity
        .damage
        .value
        .as_ref()
        .unwrap()
        .iter()
        .find(|c| c.damage_type.value.as_deref() == Some("bleed"))
        .unwrap();
    assert_eq!(bleed.category.value.as_deref(), Some("persistent"));
    let request = RollEncounterStrikeRequest {
        request_id: "bleed".into(),
        expected_revision: f
            .worker
            .encounter_initiative_options(&key)
            .unwrap()
            .expected_revision,
        participant_key: participant.participant_key.clone(),
        expected_participant_revision: participant.revision,
        reference: EncounterReferenceRequest {
            artifact_generation: detail.artifact_generation.unwrap(),
            selection: RecordDetailRequest {
                record_key: activity.navigation.record_key.clone(),
                owners: activity.navigation.owners.clone(),
                fields: vec![],
                passage: None,
                source_fingerprint: None,
                spell_rank: None,
            },
        },
        attack_number: 1,
        mode: RollMode::Normal,
        modifier_override: None,
        roll_kind: StrikeRollKind::Damage,
    };
    let result = f
        .worker
        .roll_encounter_strike(&key, request.clone())
        .unwrap();
    assert_eq!(result.damage_total, Some(5));
    assert_eq!(result.persistent_total, Some(2));

    atlas_search::test_support::invalidate_reference(f.worker.workspace_path()).unwrap();
    let recovered = f.worker.roll_encounter_strike(&key, request).unwrap();
    assert_eq!(recovered, result);
}
