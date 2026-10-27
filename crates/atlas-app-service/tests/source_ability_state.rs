//! Guarded source-state behavior, independent of the full-corpus review workspace.
use atlas_app_model::*;
use atlas_app_service::{AppServiceRetrievalMode, AtlasAppService, AtlasAppServiceOptions};
use atlas_foundry_model::generated::condition_definitions::ConditionId;
use atlas_search::test_support::{FixtureArtifact, open_source_fixture, record};
use serde_json::json;

fn fixture() -> (AtlasAppService, FixtureArtifact, String) {
    fixture_with_drained(false)
}
fn fixture_with_drained(initial_drained: bool) -> (AtlasAppService, FixtureArtifact, String) {
    fixture_with_items(initial_drained, vec![])
}
fn fixture_with_items(
    initial_drained: bool,
    items: Vec<serde_json::Value>,
) -> (AtlasAppService, FixtureArtifact, String) {
    let mut actor = json!({
        "_id":"aaaaaaaaaaaaaaaa","name":"State test creature","type":"npc",
        "system":{"traits":{"value":[]},"details":{"level":{"value":5}},
            "attributes":{"adjustment":null,"immunities":[],"ac":{"value":20},"hp":{"max":50}},
            "saves":{"fortitude":{"value":12},"reflex":{"value":12},"will":{"value":12}}},
        "items":[
            {"_id":"bbbbbbbbbbbbbbbb","name":"Mounted","type":"action","system":{"actionType":{"value":"passive"},"rules":[
                {"key":"RollOption","option":"mounted","toggleable":true,"value":true},
                {"key":"GrantItem","inMemoryOnly":true,"predicate":[{"not":"mounted"}],"uuid":"Compendium.pf2e.conditionitems.Item.Slowed"}]}},
            {"_id":"cccccccccccccccc","name":"Daily ability","type":"action","system":{"frequency":{"max":3,"per":"day"},"rules":[]}},
            {"_id":"dddddddddddddddd","name":"Curse choice","type":"action","system":{"actionType":{"value":"passive"},"rules":[
                {"key":"RollOption","option":"incurable-curse","toggleable":true,"alwaysActive":true,"value":true,"suboptions":[
                    {"label":"Clumsy","value":"Compendium.pf2e.conditionitems.Item.Clumsy"},
                    {"label":"Drained","value":"Compendium.pf2e.conditionitems.Item.Drained"}]},
                {"key":"GrantItem","inMemoryOnly":true,"uuid":"{item|flags.system.rulesSelections.incurableCurse}"}]}},
            {"_id":"eeeeeeeeeeeeeeee","name":"Turn ability","type":"action","system":{"frequency":{"max":1,"per":"turn"},"rules":[]}},
            {"_id":"gggggggggggggggg","name":"Round ability","type":"action","system":{"frequency":{"max":1,"per":"round"},"rules":[]}},
            {"_id":"hhhhhhhhhhhhhhhh","name":"Large capacity","type":"action","system":{"frequency":{"max":i64::MAX,"per":"day"},"rules":[]}}
        ]
    });
    if initial_drained {
        actor["items"].as_array_mut().unwrap().push(json!({"_id":"ffffffffffffffff","type":"condition","name":"Drained","system":{"slug":"drained","value":{"isValued":true,"value":2}}}));
    }
    actor["items"].as_array_mut().unwrap().extend(items);
    let mut records = vec![record("actors", "Actor", actor)];
    for rule in [
        ConditionId::Slowed,
        ConditionId::Clumsy,
        ConditionId::Drained,
        ConditionId::Prone,
    ] {
        let definition = rule.definition();
        records.push(record("conditionitems","Item",json!({"_id":definition.record_key.split_once(':').unwrap().1,"name":definition.name,"type":"condition",
            "system":{"slug":definition.slug,"value":{"isValued":definition.has_value,"value":definition.default_value}}})));
    }
    let (retrieval, artifact) = open_source_fixture(records, false).unwrap();
    drop(retrieval);
    let service = AtlasAppService::start(AtlasAppServiceOptions {
        default_search_profile: None,
        path_mode: atlas_runtime::AtlasPathMode::Repo,
        source_root: None,
        embedding_cache_root: None,
        index_path: Some(artifact.path()),
        retrieval_mode: AppServiceRetrievalMode::OnDemandNoEmbeddings,
    })
    .unwrap();
    let key = service
        .create_encounter(CreateEncounterRequest {
            name: "Source state".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter
        .encounter_key;
    service
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: key.clone(),
            record_ref: "actors:aaaaaaaaaaaaaaaa".into(),
            quantity: 1,
            participant_variant: None,
            initiative: Some(20),
        })
        .unwrap();
    (service, artifact, key)
}
fn request(
    detail: &EncounterDetailView,
    label: &str,
    id: &str,
    action: EncounterAbilityStateAction,
) -> UpdateEncounterAbilityStateRequest {
    let participant = &detail.participants[0];
    let ability = participant
        .ability_states
        .as_ref()
        .unwrap()
        .iter()
        .find(|ability| ability.label == label)
        .unwrap();
    UpdateEncounterAbilityStateRequest {
        request_id: id.into(),
        expected_encounter_revision: detail.revision,
        expected_participant_revision: participant.revision,
        participant_key: participant.participant_key.clone(),
        navigation: ability.navigation.clone(),
        rule_ordinal: ability.rule_ordinal,
        action,
    }
}
fn remaining(detail: &EncounterDetailView, label: &str) -> i64 {
    match &detail.participants[0]
        .ability_states
        .as_ref()
        .unwrap()
        .iter()
        .find(|ability| ability.label == label)
        .unwrap()
        .state
    {
        EncounterAbilityStateKindView::Uses { remaining, .. } => *remaining,
        _ => panic!("expected uses"),
    }
}

#[test]
fn source_switches_and_uses_persist_without_materializing_editable_conditions() {
    let (service, _artifact, key) = fixture();
    let initial = service.encounter(&key).unwrap();
    assert_eq!(
        initial.participants[0]
            .turn_resources
            .as_ref()
            .unwrap()
            .forecast
            .adjusted_value,
        Some(3)
    );
    let mounted = request(
        &initial,
        "Mounted",
        "dismount",
        EncounterAbilityStateAction::SetOption {
            active: false,
            selection: None,
        },
    );
    service
        .update_encounter_ability_state(&key, mounted)
        .unwrap();
    let after = service.encounter(&key).unwrap();
    assert_eq!(
        after.participants[0]
            .turn_resources
            .as_ref()
            .unwrap()
            .forecast
            .adjusted_value,
        Some(2)
    );
    assert!(after.participants[0].conditions.is_empty());
    let spend = request(
        &after,
        "Daily ability",
        "daily-use",
        EncounterAbilityStateAction::SpendUse,
    );
    service
        .update_encounter_ability_state(&key, spend.clone())
        .unwrap();
    service
        .update_encounter_ability_state(&key, spend.clone())
        .unwrap();
    assert_eq!(
        remaining(&service.encounter(&key).unwrap(), "Daily ability"),
        2
    );
    assert_eq!(
        remaining(&service.encounter(&key).unwrap(), "Daily ability"),
        2
    );
    let mut changed_intent = spend;
    changed_intent.action = EncounterAbilityStateAction::RestoreUse;
    assert!(
        service
            .update_encounter_ability_state(&key, changed_intent)
            .is_err()
    );
    let current = service.encounter(&key).unwrap();
    let mut stale = request(
        &current,
        "Daily ability",
        "stale-source",
        EncounterAbilityStateAction::SpendUse,
    );
    stale.navigation.source_fingerprint = Some("another source".into());
    assert!(service.update_encounter_ability_state(&key, stale).is_err());
    assert_eq!(
        remaining(&service.encounter(&key).unwrap(), "Daily ability"),
        2
    );
}

#[test]
fn selecting_drained_applies_only_the_increase_and_never_repeats_on_read_or_retry() {
    let (service, _artifact, key) = fixture();
    let initial = service.encounter(&key).unwrap();
    let select = request(
        &initial,
        "Curse choice",
        "select-drained",
        EncounterAbilityStateAction::SetOption {
            active: true,
            selection: Some("Compendium.pf2e.conditionitems.Item.Drained".into()),
        },
    );
    service
        .update_encounter_ability_state(&key, select.clone())
        .unwrap();
    service
        .update_encounter_ability_state(&key, select)
        .unwrap();
    for _ in 0..2 {
        let after = service.encounter(&key).unwrap();
        assert_eq!(after.participants[0].current_hp, Some(45));
        assert_eq!(
            after.participants[0]
                .effective_max_hp
                .value
                .as_ref()
                .and_then(serde_json::Number::as_i64),
            Some(45)
        );
        assert!(after.participants[0].conditions.is_empty());
    }
    let after = service.encounter(&key).unwrap();
    service.add_encounter_participant_condition(&key,serde_json::from_value(json!({"request_id":"external-drained","expected_encounter_revision":after.revision,
        "participant_key":after.participants[0].participant_key,"condition_ref":ConditionId::Drained.definition().record_key,"value":1})).unwrap()).unwrap();
    let overlapping = service.encounter(&key).unwrap();
    assert_eq!(overlapping.participants[0].current_hp, Some(45));
    assert_eq!(
        overlapping.participants[0]
            .effective_max_hp
            .value
            .as_ref()
            .and_then(serde_json::Number::as_i64),
        Some(45)
    );
}

#[test]
fn advancing_initiative_refreshes_turn_uses_and_preserves_daily_expenditure() {
    let (service, _artifact, key) = fixture();
    for (label, id) in [("Daily ability", "daily"), ("Turn ability", "turn")] {
        let detail = service.encounter(&key).unwrap();
        service
            .update_encounter_ability_state(
                &key,
                request(&detail, label, id, EncounterAbilityStateAction::SpendUse),
            )
            .unwrap();
    }
    let before = service.encounter(&key).unwrap();
    assert_eq!(remaining(&before, "Turn ability"), 0);
    let advance = AdvanceEncounterTurnRequest {
        request_id: "advance".into(),
        expected_revision: before.revision,
    };
    service
        .advance_encounter_turn(&key, advance.clone())
        .unwrap();
    service.advance_encounter_turn(&key, advance).unwrap();
    let after = service.encounter(&key).unwrap();
    assert_eq!(remaining(&after, "Turn ability"), 1);
    assert_eq!(remaining(&after, "Daily ability"), 2);
}

#[test]
fn restoring_a_full_capacity_is_rejected_without_overflow_or_a_write() {
    let (service, _artifact, key) = fixture();
    let before = service.encounter(&key).unwrap();
    let error = service
        .update_encounter_ability_state(
            &key,
            request(
                &before,
                "Large capacity",
                "restore-full-capacity",
                EncounterAbilityStateAction::RestoreUse,
            ),
        )
        .unwrap_err()
        .into_app_error();
    assert_eq!(error.code, AppErrorCode::InvalidRequest);
    let after = service.encounter(&key).unwrap();
    assert_eq!(after.revision, before.revision);
    assert_eq!(
        after.participants[0].revision,
        before.participants[0].revision
    );
    assert_eq!(remaining(&after, "Large capacity"), i64::MAX);
}

#[test]
fn round_uses_stay_spent_until_the_owners_next_turn() {
    let (service, _artifact, key) = fixture();
    service
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: key.clone(),
            record_ref: "actors:aaaaaaaaaaaaaaaa".into(),
            quantity: 1,
            participant_variant: None,
            initiative: Some(10),
        })
        .unwrap();
    let advance = |id: &str| {
        let detail = service.encounter(&key).unwrap();
        service
            .advance_encounter_turn(
                &key,
                AdvanceEncounterTurnRequest {
                    request_id: id.into(),
                    expected_revision: detail.revision,
                },
            )
            .unwrap();
    };
    advance("start-owner");
    let initial = service.encounter(&key).unwrap();
    let owner = initial.participants[0].participant_key.clone();
    assert_eq!(
        initial.current_turn_participant_key.as_deref(),
        Some(owner.as_str())
    );
    service
        .update_encounter_ability_state(
            &key,
            request(
                &initial,
                "Round ability",
                "spend-round-use",
                EncounterAbilityStateAction::SpendUse,
            ),
        )
        .unwrap();
    assert_eq!(
        remaining(&service.encounter(&key).unwrap(), "Round ability"),
        0
    );
    advance("other-turn");
    let other_turn = service.encounter(&key).unwrap();
    assert_ne!(
        other_turn.current_turn_participant_key.as_deref(),
        Some(owner.as_str())
    );
    assert_eq!(remaining(&other_turn, "Round ability"), 0);
    advance("next-owner-turn");
    let owner_turn = service.encounter(&key).unwrap();
    assert_eq!(
        owner_turn.current_turn_participant_key.as_deref(),
        Some(owner.as_str())
    );
    assert_eq!(remaining(&owner_turn, "Round ability"), 1);
    // Retrying the same transition must not refill a use spent afterward.
    service
        .update_encounter_ability_state(
            &key,
            request(
                &owner_turn,
                "Round ability",
                "spend-next-round-use",
                EncounterAbilityStateAction::SpendUse,
            ),
        )
        .unwrap();
    service
        .advance_encounter_turn(
            &key,
            AdvanceEncounterTurnRequest {
                request_id: "next-owner-turn".into(),
                expected_revision: other_turn.revision,
            },
        )
        .unwrap();
    assert_eq!(
        remaining(&service.encounter(&key).unwrap(), "Round ability"),
        0
    );
}

#[test]
fn source_drained_initializes_current_hp_at_effective_capacity_without_charging_on_reads() {
    let (service, _artifact, key) = fixture_with_drained(true);
    for _ in 0..2 {
        let detail = service.encounter(&key).unwrap();
        let participant = &detail.participants[0];
        assert_eq!(participant.max_hp, Some(50));
        assert_eq!(participant.current_hp, Some(40));
        assert_eq!(
            participant
                .effective_max_hp
                .value
                .as_ref()
                .and_then(serde_json::Number::as_i64),
            Some(40)
        );
        assert!(participant.conditions.is_empty());
    }
}

fn participant_update(participant: &EncounterParticipantView) -> UpdateEncounterParticipantRequest {
    UpdateEncounterParticipantRequest {
        expected_revision: participant.revision,
        hp_edit: false,
        max_hp_edit: false,
        variant_edit: false,
        use_derived_max: false,
        participant_key: participant.participant_key.clone(),
        display_name: participant.display_name.clone(),
        side: participant.side,
        participant_variant: participant.participant_variant,
        initiative: participant.initiative,
        max_hp: participant.max_hp,
        current_hp: participant.current_hp,
        temporary_hp: participant.temporary_hp,
        defeated: participant.defeated,
        hazard_disabled: participant.hazard_disabled,
        hidden: participant.hidden,
        note: participant.note.clone(),
    }
}

#[test]
fn derived_maximum_reset_preserves_pristine_source_drained_capacity() {
    let (service, _artifact, key) = fixture_with_drained(true);
    let initial = service.encounter(&key).unwrap();
    let mut update = participant_update(&initial.participants[0]);
    update.use_derived_max = true;
    service.update_encounter_participant(&key, update).unwrap();
    let after = service.encounter(&key).unwrap();
    assert_eq!(
        (
            after.participants[0].max_hp,
            after.participants[0].current_hp
        ),
        (Some(50), Some(40))
    );
}

#[test]
fn pristine_variant_edit_uses_conditioned_capacity_at_the_new_level() {
    let (service, _artifact, key) = fixture_with_drained(true);
    let initial = service.encounter(&key).unwrap();
    let mut update = participant_update(&initial.participants[0]);
    update.participant_variant = EncounterParticipantVariantView::Elite;
    update.variant_edit = true;
    service.update_encounter_participant(&key, update).unwrap();
    let after = service.encounter(&key).unwrap();
    assert_eq!(
        (
            after.participants[0].max_hp,
            after.participants[0].current_hp
        ),
        (Some(70), Some(58))
    );
}

#[test]
fn running_variant_edit_preserves_damage_below_conditioned_capacity() {
    let (service, _artifact, key) = fixture_with_drained(true);
    let initial = service.encounter(&key).unwrap();
    service
        .update_encounter(UpdateEncounterRequest {
            encounter_key: key.clone(),
            slug: initial.encounter.slug.clone(),
            name: initial.encounter.name.clone(),
            description: None,
            note: None,
            status: EncounterStatusView::Running,
        })
        .unwrap();
    let mut damage = participant_update(&initial.participants[0]);
    damage.current_hp = Some(35);
    damage.hp_edit = true;
    service.update_encounter_participant(&key, damage).unwrap();
    let damaged = service.encounter(&key).unwrap();
    let mut update = participant_update(&damaged.participants[0]);
    update.participant_variant = EncounterParticipantVariantView::Elite;
    update.variant_edit = true;
    service.update_encounter_participant(&key, update).unwrap();
    let after = service.encounter(&key).unwrap();
    assert_eq!(
        (
            after.participants[0].max_hp,
            after.participants[0].current_hp
        ),
        (Some(70), Some(53))
    );
}

fn predicate_drained_fixture(negated: bool) -> (AtlasAppService, FixtureArtifact, String) {
    let predicate = if negated {
        json!([{"not":"self:condition:prone"}])
    } else {
        json!(["self:condition:prone"])
    };
    fixture_with_items(
        false,
        vec![
            json!({"_id":"iiiiiiiiiiiiiiii","name":"Conditional drain","type":"action","system":{"actionType":{"value":"passive"},"rules":[
                {"key":"GrantItem","inMemoryOnly":true,"predicate":predicate,"uuid":"Compendium.pf2e.conditionitems.Item.Drained"}
            ]}}),
        ],
    )
}

fn prone_request(
    detail: &EncounterDetailView,
    id: &str,
) -> AddEncounterParticipantConditionRequest {
    serde_json::from_value(json!({
        "request_id": id,
        "expected_encounter_revision":detail.revision,
        "participant_key":detail.participants[0].participant_key,
        "condition_ref":ConditionId::Prone.definition().record_key
    }))
    .unwrap()
}

#[test]
fn predicate_activation_charges_drained_once_and_deactivation_never_heals() {
    let (service, _artifact, key) = predicate_drained_fixture(false);
    let initial = service.encounter(&key).unwrap();
    assert_eq!(initial.participants[0].current_hp, Some(50));
    let add = prone_request(&initial, "prone-activates-drained");
    service
        .add_encounter_participant_condition(&key, add.clone())
        .unwrap();
    service
        .add_encounter_participant_condition(&key, add)
        .unwrap();
    let active = service.encounter(&key).unwrap();
    assert_eq!(active.participants[0].current_hp, Some(45));
    assert_eq!(
        active.participants[0]
            .effective_max_hp
            .value
            .as_ref()
            .and_then(serde_json::Number::as_i64),
        Some(45)
    );
    service
        .remove_encounter_participant_condition(
            &key,
            &active.participants[0].participant_key,
            active.participants[0].conditions[0].condition_id,
            RemoveEncounterParticipantConditionRequest {
                request_id: "end-prone".into(),
                expected_encounter_revision: active.revision,
                intent: ConditionRemovalIntentView::RemoveEffect,
            },
        )
        .unwrap();
    let inactive = service.encounter(&key).unwrap();
    assert_eq!(inactive.participants[0].current_hp, Some(45));
    assert_eq!(
        inactive.participants[0]
            .effective_max_hp
            .value
            .as_ref()
            .and_then(serde_json::Number::as_i64),
        Some(50)
    );
}

#[test]
fn removing_a_predicate_condition_charges_new_drained_once_with_known_level() {
    let (service, _artifact, key) = predicate_drained_fixture(true);
    let initial = service.encounter(&key).unwrap();
    assert_eq!(initial.participants[0].current_hp, Some(45));
    service
        .add_encounter_participant_condition(&key, prone_request(&initial, "disable-drained"))
        .unwrap();
    let inactive = service.encounter(&key).unwrap();
    assert_eq!(inactive.participants[0].current_hp, Some(45));
    let remove = RemoveEncounterParticipantConditionRequest {
        request_id: "reactivate-drained".into(),
        expected_encounter_revision: inactive.revision,
        intent: ConditionRemovalIntentView::RemoveEffect,
    };
    let condition_id = inactive.participants[0].conditions[0].condition_id;
    let participant_key = &inactive.participants[0].participant_key;
    service
        .remove_encounter_participant_condition(&key, participant_key, condition_id, remove.clone())
        .unwrap();
    service
        .remove_encounter_participant_condition(&key, participant_key, condition_id, remove)
        .unwrap();
    let active = service.encounter(&key).unwrap();
    assert_eq!(active.participants[0].current_hp, Some(40));
}

#[test]
fn corrected_predicate_activation_changes_capacity_without_charging_hp() {
    let (service, _artifact, key) = predicate_drained_fixture(false);
    let initial = service.encounter(&key).unwrap();
    let mut correction = prone_request(&initial, "correct-prone");
    correction.event_intent = ConditionEventIntentView::Correction;
    service
        .add_encounter_participant_condition(&key, correction)
        .unwrap();
    let active = service.encounter(&key).unwrap();
    assert_eq!(active.participants[0].current_hp, Some(50));
    assert_eq!(
        active.participants[0]
            .effective_max_hp
            .value
            .as_ref()
            .and_then(serde_json::Number::as_i64),
        Some(45)
    );
}

#[test]
fn unknown_source_condition_cannot_roll_an_authored_strike_subtotal() {
    let (service, _artifact, key) = fixture_with_items(
        false,
        vec![
            json!({"_id":"iiiiiiiiiiiiiiii","name":"Contextual form","type":"action","system":{"actionType":{"value":"passive"},"rules":[
                {"key":"GrantItem","inMemoryOnly":true,"predicate":["form:battle"],"uuid":"Compendium.pf2e.conditionitems.Item.Clumsy","alterations":[{"property":"badge-value","mode":"override","value":2}]}
            ]}}),
            json!({"_id":"jjjjjjjjjjjjjjjj","name":"Finesse claw","type":"melee","system":{"range":null,"bonus":{"value":14},"traits":{"value":["finesse"]},"damageRolls":{"first":{"damage":"1d6+4","damageType":"slashing","category":null}}}}),
        ],
    );
    let initial = service.encounter(&key).unwrap();
    let participant = &initial.participants[0];
    let error = service
        .roll_encounter_strike(
            &key,
            RollEncounterStrikeRequest {
                request_id: "unknown-clumsy-strike".into(),
                expected_revision: initial.revision,
                participant_key: participant.participant_key.clone(),
                expected_participant_revision: participant.revision,
                reference: EncounterReferenceRequest {
                    artifact_generation: initial.artifact_generation.clone().unwrap(),
                    selection: RecordDetailRequest {
                        spell_rank: None,
                        record_key: participant.record_key.clone().unwrap(),
                        owners: vec![atlas_domain::OwnedContentLocator {
                            collection: "/items".into(),
                            identity: atlas_domain::OwnedContentIdentity::Stable(
                                "jjjjjjjjjjjjjjjj".into(),
                            ),
                        }],
                        fields: vec![],
                        passage: None,
                        source_fingerprint: None,
                    },
                },
                attack_number: 1,
                mode: RollMode::Normal,
                modifier_override: None,
                roll_kind: StrikeRollKind::Attack,
            },
        )
        .unwrap_err()
        .into_app_error();
    assert_eq!(error.code, AppErrorCode::InvalidRequest);
    assert!(
        error.message.contains("activation needs context"),
        "{}",
        error.message
    );
    let after = service.encounter(&key).unwrap();
    assert_eq!(after.revision, initial.revision);
    assert_eq!(after.participants[0].revision, participant.revision);
    assert_eq!(after.participants[0].current_hp, participant.current_hp);
}
