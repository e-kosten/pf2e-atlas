use crate::{executor::RetrievalExecutor, test_support::fixture_worker_with_executor};
use atlas_app_model::*;
use atlas_search::test_support::{open_source_fixture, record};
use serde_json::json;

fn casting_fixture(items: serde_json::Value) -> (crate::test_support::FixtureWorker, String) {
    let source = json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","name":"Caster",
        "system":{"details":{"level":{"value":5}},"resources":{"focus":{"max":2,"value":0}}},"items":items});
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
    let encounter = fixture
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Resource policies".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    fixture
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: encounter.slug.clone(),
            record_ref: "actors:aaaaaaaaaaaaaaaa".into(),
            quantity: 1,
            initiative: None,
        })
        .unwrap();
    (fixture, encounter.slug)
}

fn cast_spell(
    fixture: &crate::test_support::FixtureWorker,
    encounter: &str,
    title: &str,
    rank: u8,
) -> crate::AppServiceResult<EncounterMutationView> {
    let current = fixture.worker.encounter(encounter).unwrap();
    let participant = &current.participants[0];
    let mut selection = participant
        .spellcasting
        .as_ref()
        .unwrap()
        .spells
        .iter()
        .find(|spell| spell.title == title)
        .unwrap()
        .navigation
        .clone();
    selection.spell_rank = Some(rank);
    fixture.worker.apply_encounter_spell_action(
        encounter,
        ApplyEncounterSpellActionRequest {
            participant_key: participant.participant_key.clone(),
            expected_revision: participant.revision,
            action: EncounterSpellAction::Cast { selection },
        },
    )
}

fn remaining_for(participant: &EncounterParticipantView, title: &str, rank: u8) -> i64 {
    let catalog = participant.spellcasting.as_ref().unwrap();
    let key = catalog
        .spells
        .iter()
        .find(|s| s.title == title)
        .unwrap()
        .ranks
        .iter()
        .find(|r| r.rank == rank)
        .unwrap()
        .resource_key
        .as_ref()
        .unwrap();
    catalog
        .resources
        .iter()
        .find(|r| &r.key == key)
        .unwrap()
        .remaining
}

#[test]
fn casting_one_prepared_rank_preserves_other_preparations() {
    let (fixture, encounter) = casting_fixture(json!([
        {"_id":"eeeeeeeeeeeeeeee","type":"spellcastingEntry","system":{"prepared":{"value":"prepared"},"slots":{
            "slot1":{"max":1,"prepared":[{"id":"ssssssssssssssss","expended":true}]},
            "slot3":{"max":1,"prepared":[{"id":"ssssssssssssssss","expended":true}]}}}},
        {"_id":"ssssssssssssssss","name":"Heal","type":"spell","system":{"level":{"value":1},"location":{"value":"eeeeeeeeeeeeeeee"}}}
    ]));
    cast_spell(&fixture, &encounter, "Heal", 3).unwrap();
    let current = fixture.worker.encounter(&encounter).unwrap();
    assert_eq!(remaining_for(&current.participants[0], "Heal", 3), 0);
    assert_eq!(remaining_for(&current.participants[0], "Heal", 1), 1);
    assert!(cast_spell(&fixture, &encounter, "Heal", 3).is_err());
    cast_spell(&fixture, &encounter, "Heal", 1).unwrap();
    let current = fixture.worker.encounter(&encounter).unwrap();
    assert_eq!(remaining_for(&current.participants[0], "Heal", 1), 0);
}

fn focus_items() -> serde_json::Value {
    json!([
        {"_id":"eeeeeeeeeeeeeeee","type":"spellcastingEntry","system":{"prepared":{"value":"focus"}}},
        {"_id":"ffffffffffffffff","type":"spellcastingEntry","system":{"prepared":{"value":"focus"}}},
        {"_id":"ssssssssssssssss","name":"First focus","type":"spell","system":{"level":{"value":1},"traits":{"value":["focus"]},"location":{"value":"eeeeeeeeeeeeeeee"}}},
        {"_id":"tttttttttttttttt","name":"Second focus","type":"spell","system":{"level":{"value":1},"traits":{"value":["focus"]},"location":{"value":"ffffffffffffffff"}}},
        {"_id":"cccccccccccccccc","name":"Focus cantrip","type":"spell","system":{"level":{"value":1},"traits":{"value":["focus","cantrip"]},"location":{"value":"eeeeeeeeeeeeeeee"}}}
    ])
}

#[test]
fn focus_casts_share_the_participants_pool_across_entries() {
    let (fixture, encounter) = casting_fixture(focus_items());
    cast_spell(&fixture, &encounter, "First focus", 3).unwrap();
    let current = fixture.worker.encounter(&encounter).unwrap();
    assert_eq!(
        remaining_for(&current.participants[0], "Second focus", 3),
        1
    );
    cast_spell(&fixture, &encounter, "Second focus", 3).unwrap();
    let current = fixture.worker.encounter(&encounter).unwrap();
    assert_eq!(remaining_for(&current.participants[0], "First focus", 3), 0);
    assert!(cast_spell(&fixture, &encounter, "First focus", 3).is_err());
}

#[test]
fn cantrip_cast_preserves_exhausted_resources_and_revisions() {
    let (fixture, encounter) = casting_fixture(focus_items());
    cast_spell(&fixture, &encounter, "First focus", 3).unwrap();
    cast_spell(&fixture, &encounter, "Second focus", 3).unwrap();
    let before = fixture.worker.encounter(&encounter).unwrap();
    cast_spell(&fixture, &encounter, "Focus cantrip", 3).unwrap();
    let after = fixture.worker.encounter(&encounter).unwrap();
    assert_eq!(
        after.participants[0].revision,
        before.participants[0].revision
    );
    assert_eq!(
        after.participants[0].spellcasting,
        before.participants[0].spellcasting
    );
    assert_eq!(after.revision, before.revision);
}

#[test]
fn spontaneous_heightening_requires_signature_or_known_rank() {
    let (fixture, encounter) = casting_fixture(json!([
        {"_id":"eeeeeeeeeeeeeeee","type":"spellcastingEntry","system":{"prepared":{"value":"spontaneous"},"slots":{"slot1":{"max":2},"slot3":{"max":2}}}},
        {"_id":"ssssssssssssssss","name":"Signature","type":"spell","system":{"level":{"value":1},"location":{"value":"eeeeeeeeeeeeeeee","signature":true}}},
        {"_id":"tttttttttttttttt","name":"Ordinary","type":"spell","system":{"level":{"value":1},"location":{"value":"eeeeeeeeeeeeeeee"}}}
    ]));
    let before = fixture.worker.encounter(&encounter).unwrap();
    assert!(cast_spell(&fixture, &encounter, "Ordinary", 3).is_err());
    let rejected = fixture.worker.encounter(&encounter).unwrap();
    assert_eq!(
        rejected.participants[0].revision,
        before.participants[0].revision
    );
    assert_eq!(
        rejected.participants[0].spellcasting,
        before.participants[0].spellcasting
    );
    cast_spell(&fixture, &encounter, "Signature", 3).unwrap();
    let current = fixture.worker.encounter(&encounter).unwrap();
    assert_eq!(remaining_for(&current.participants[0], "Signature", 3), 1);
    assert_eq!(remaining_for(&current.participants[0], "Ordinary", 1), 2);
    cast_spell(&fixture, &encounter, "Ordinary", 1).unwrap();
    let current = fixture.worker.encounter(&encounter).unwrap();
    assert_eq!(remaining_for(&current.participants[0], "Signature", 1), 1);
}

#[test]
fn casting_is_persistent_shared_within_entry_and_isolated_between_participants() {
    let source = json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","name":"Caster","system":{"details":{"level":{"value":5}},"attributes":{"hp":{"max":60}}},"items":[
        {"_id":"eeeeeeeeeeeeeeee","name":"Arcane spells","type":"spellcastingEntry","system":{"prepared":{"value":"spontaneous"},"tradition":{"value":"arcane"},"slots":{"slot3":{"max":2,"value":0}}}},
        {"_id":"ssssssssssssssss","name":"Fireball","type":"spell","system":{"level":{"value":3},"location":{"value":"eeeeeeeeeeeeeeee"}}},
        {"_id":"tttttttttttttttt","name":"Other spell","type":"spell","system":{"level":{"value":3},"location":{"value":"eeeeeeeeeeeeeeee"}}}
    ]});
    let f = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        move || {
            Ok(open_source_fixture(
                vec![record("actors", "Actor", source.clone())],
                false,
            )?)
        },
    ));
    let encounter = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Casting".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    f.worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: encounter.slug.clone(),
            record_ref: "actors:aaaaaaaaaaaaaaaa".into(),
            quantity: 2,
            initiative: None,
        })
        .unwrap();
    let read = || f.worker.encounter(&encounter.slug).unwrap();
    let current = read();
    let p = &current.participants[0];
    let catalog = p.spellcasting.as_ref().unwrap();
    assert_eq!(catalog.resources.len(), 1);
    assert_eq!(catalog.resources[0].remaining, 2); // Full allotment, not source's zero remaining.
    let key = catalog.resources[0].key.clone();
    let mut selection = catalog.spells[0].navigation.clone();
    selection.spell_rank = Some(3);
    let cast = |revision, selection| {
        f.worker.apply_encounter_spell_action(
            &encounter.slug,
            ApplyEncounterSpellActionRequest {
                participant_key: p.participant_key.clone(),
                expected_revision: revision,
                action: EncounterSpellAction::Cast { selection },
            },
        )
    };
    cast(p.revision, selection.clone()).unwrap();
    assert!(cast(p.revision, selection.clone()).is_err()); // stale must not spend twice
    let next = read();
    assert_eq!(
        next.participants[0]
            .spellcasting
            .as_ref()
            .unwrap()
            .resources[0]
            .remaining,
        1
    );
    assert_eq!(
        next.participants[1]
            .spellcasting
            .as_ref()
            .unwrap()
            .resources[0]
            .remaining,
        2
    );
    let mut other = catalog.spells[1].navigation.clone();
    other.spell_rank = Some(3);
    cast(next.participants[0].revision, other).unwrap();
    let empty = read();
    assert_eq!(
        empty.participants[0]
            .spellcasting
            .as_ref()
            .unwrap()
            .resources[0]
            .remaining,
        0
    );
    assert!(cast(empty.participants[0].revision, selection).is_err());
    f.worker
        .apply_encounter_spell_action(
            &encounter.slug,
            ApplyEncounterSpellActionRequest {
                participant_key: p.participant_key.clone(),
                expected_revision: empty.participants[0].revision,
                action: EncounterSpellAction::Undo {
                    resource_key: key.clone(),
                },
            },
        )
        .unwrap();
    let undone = read();
    assert_eq!(
        undone.participants[0]
            .spellcasting
            .as_ref()
            .unwrap()
            .resources[0]
            .remaining,
        1
    );
    assert!(
        !undone.participants[0]
            .spellcasting
            .as_ref()
            .unwrap()
            .resources[0]
            .can_undo
    );
    f.worker
        .apply_encounter_spell_action(
            &encounter.slug,
            ApplyEncounterSpellActionRequest {
                participant_key: p.participant_key.clone(),
                expected_revision: undone.participants[0].revision,
                action: EncounterSpellAction::SetEntryResources {
                    entry_id: "eeeeeeeeeeeeeeee".into(),
                    remaining: [(key.clone(), 2)].into(),
                },
            },
        )
        .unwrap();
    assert_eq!(
        read().participants[0]
            .spellcasting
            .as_ref()
            .unwrap()
            .resources[0]
            .remaining,
        2
    );
}

#[test]
fn missing_innate_counts_use_rank_independent_markers_without_inventing_limits() {
    let source = json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","name":"Innate caster","system":{"details":{"level":{"value":9}}},"items":[
        {"_id":"eeeeeeeeeeeeeeee","name":"Innate spells","type":"spellcastingEntry","system":{"prepared":{"value":"innate"}}},
        {"_id":"ssssssssssssssss","name":"Spell (At Will)","type":"spell","system":{"level":{"value":1},"location":{"value":"eeeeeeeeeeeeeeee","heightenedLevel":5}}},
        {"_id":"tttttttttttttttt","name":"Limited spell","type":"spell","system":{"level":{"value":1},"location":{"value":"eeeeeeeeeeeeeeee","uses":{"max":3,"value":0}}}}
    ]});
    let f = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        move || {
            Ok(open_source_fixture(
                vec![record("actors", "Actor", source.clone())],
                false,
            )?)
        },
    ));
    let encounter = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Innate casting".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    f.worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: encounter.slug.clone(),
            record_ref: "actors:aaaaaaaaaaaaaaaa".into(),
            quantity: 2,
            initiative: None,
        })
        .unwrap();
    let read = || f.worker.encounter(&encounter.slug).unwrap();
    let initial = read();
    let participant = &initial.participants[0];
    let catalog = participant.spellcasting.as_ref().unwrap();
    assert_eq!(catalog.resources.len(), 1);
    assert_eq!(catalog.resources[0].remaining, 3);
    let spell = catalog
        .spells
        .iter()
        .find(|s| s.title == "Spell (At Will)")
        .unwrap();
    assert_eq!(spell.expended, Some(false));
    assert!(spell.ranks.iter().all(|r| r.resource_key.is_none()));
    let mut selection = spell.navigation.clone();
    selection.spell_rank = Some(5);
    let mark = |revision, selection, expended| {
        f.worker.apply_encounter_spell_action(
            &encounter.slug,
            ApplyEncounterSpellActionRequest {
                participant_key: participant.participant_key.clone(),
                expected_revision: revision,
                action: EncounterSpellAction::SetExpended {
                    selection,
                    expended,
                },
            },
        )
    };
    mark(participant.revision, selection.clone(), true).unwrap();
    assert!(mark(participant.revision, selection.clone(), false).is_err());
    let marked = read();
    let find_marker = |p: &EncounterParticipantView| {
        p.spellcasting
            .as_ref()
            .unwrap()
            .spells
            .iter()
            .find(|s| s.title == "Spell (At Will)")
            .unwrap()
            .expended
    };
    assert_eq!(find_marker(&marked.participants[0]), Some(true));
    assert_eq!(find_marker(&marked.participants[1]), Some(false));
    assert_eq!(
        marked.participants[0]
            .spellcasting
            .as_ref()
            .unwrap()
            .resources[0]
            .remaining,
        3
    );
    selection.spell_rank = Some(2);
    mark(marked.participants[0].revision, selection, false).unwrap();
    let restored = read();
    assert_eq!(find_marker(&restored.participants[0]), Some(false));
    let limited = catalog
        .spells
        .iter()
        .find(|s| s.title == "Limited spell")
        .unwrap();
    assert_eq!(limited.expended, None);
    assert!(
        mark(
            restored.participants[0].revision,
            limited.navigation.clone(),
            true
        )
        .is_err()
    );
}

#[test]
fn resource_editor_saves_one_complete_entry_atomically() {
    let source = json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","name":"Caster","system":{"details":{"level":{"value":5}}},"items":[
        {"_id":"eeeeeeeeeeeeeeee","name":"Innate spells","type":"spellcastingEntry","system":{"prepared":{"value":"innate"}}},
        {"_id":"ssssssssssssssss","name":"First","type":"spell","system":{"level":{"value":1},"location":{"value":"eeeeeeeeeeeeeeee","uses":{"max":3}}}},
        {"_id":"tttttttttttttttt","name":"Second","type":"spell","system":{"level":{"value":1},"location":{"value":"eeeeeeeeeeeeeeee","uses":{"max":2}}}}
    ]});
    let f = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        move || {
            Ok(open_source_fixture(
                vec![record("actors", "Actor", source.clone())],
                false,
            )?)
        },
    ));
    let encounter = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Resource editor".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    f.worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: encounter.slug.clone(),
            record_ref: "actors:aaaaaaaaaaaaaaaa".into(),
            quantity: 1,
            initiative: None,
        })
        .unwrap();
    let read = || f.worker.encounter(&encounter.slug).unwrap();
    let initial = read();
    let p = &initial.participants[0];
    let resources = &p.spellcasting.as_ref().unwrap().resources;
    let save = |remaining| {
        f.worker.apply_encounter_spell_action(
            &encounter.slug,
            ApplyEncounterSpellActionRequest {
                participant_key: p.participant_key.clone(),
                expected_revision: p.revision,
                action: EncounterSpellAction::SetEntryResources {
                    entry_id: "eeeeeeeeeeeeeeee".into(),
                    remaining,
                },
            },
        )
    };
    let invalid = [
        (resources[0].key.clone(), 0),
        (resources[1].key.clone(), 100),
    ]
    .into();
    assert!(save(invalid).is_err());
    assert_eq!(read().participants[0].spellcasting, p.spellcasting);
    assert_eq!(read().participants[0].revision, p.revision);
    assert!(save([(resources[0].key.clone(), 0)].into()).is_err());
    let valid = resources.iter().map(|r| (r.key.clone(), 1)).collect();
    save(valid).unwrap();
    let saved = read();
    assert!(
        saved.participants[0]
            .spellcasting
            .as_ref()
            .unwrap()
            .resources
            .iter()
            .all(|r| r.remaining == 1)
    );
    assert_eq!(saved.participants[0].revision, p.revision + 1);
    assert!(
        save(
            resources
                .iter()
                .map(|r| (r.key.clone(), r.capacity))
                .collect()
        )
        .is_err()
    );
}
