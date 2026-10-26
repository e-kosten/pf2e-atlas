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
            {"_id":"eeeeeeeeeeeeeeee","name":"Turn ability","type":"action","system":{"frequency":{"max":1,"per":"turn"},"rules":[]}}
        ]
    });
    if initial_drained {
        actor["items"].as_array_mut().unwrap().push(json!({"_id":"ffffffffffffffff","type":"condition","name":"Drained","system":{"slug":"drained","value":{"isValued":true,"value":2}}}));
    }
    let mut records = vec![record("actors", "Actor", actor)];
    for rule in [
        ConditionId::Slowed,
        ConditionId::Clumsy,
        ConditionId::Drained,
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
