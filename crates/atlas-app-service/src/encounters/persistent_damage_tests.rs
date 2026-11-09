use crate::test_support::{FixtureWorker, encounter_fixture_worker};
use atlas_app_model::*;
use atlas_local_state::{
    AddEncounterParticipantCondition, ConditionParameters, DamageMultiplier, PersistentRecovery,
};

fn fixture() -> (FixtureWorker, String, String) {
    let f = encounter_fixture_worker();
    let key = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Persistent damage".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter
        .encounter_key;
    f.worker
        .add_encounter_manual_participant(
            &key,
            AddEncounterManualParticipantRequest {
                display_name: "PC".into(),
                max_hp: Some(20),
                current_hp: Some(5),
                initiative: Some(10),
            },
        )
        .unwrap();
    let participant = snapshot(&f, &key).participants[0].participant_key.clone();
    f.worker
        .with_write(|session| {
            let store = session.local_state()?.encounters();
            for (formula, damage_type) in [("4", "fire"), ("2", "fire"), ("3", "acid")] {
                store.add_condition(AddEncounterParticipantCondition {
                    participant_key: participant.clone(),
                    condition_key: Some("conditionitems:lDVqvLKA6eF3Df60".into()),
                    name: "Persistent Damage".into(),
                    value: None,
                    source_participant_key: None,
                    scope: Default::default(),
                    extra_action: Default::default(),
                    immunity_override: false,
                    parameters: ConditionParameters::PersistentDamage {
                        formula: formula.into(),
                        damage_type: damage_type.into(),
                        multiplier: DamageMultiplier::Normal,
                        recovery: PersistentRecovery::Standard,
                        episode_id: format!("episode-{damage_type}"),
                    },
                    origin: None,
                    duration: None,
                    note: None,
                })?;
            }
            Ok(())
        })
        .unwrap();
    (f, key, participant)
}

fn snapshot(f: &FixtureWorker, key: &str) -> atlas_local_state::EncounterWithParticipants {
    f.worker
        .with_state(|session| {
            Ok(session
                .local_state()?
                .encounters()
                .get_with_participants(key)?)
        })
        .unwrap()
        .unwrap()
}

fn batch(f: &FixtureWorker, key: &str, participant: &str) -> ApplyPersistentDamageRequest {
    ApplyPersistentDamageRequest {
        recoveries: vec![],
        boundary_key: None,
        request_id: "damage-once".into(),
        expected_encounter_revision: snapshot(f, key).encounter.revision,
        participant_key: participant.into(),
        artifact_generation: None,
        groups: vec![
            PersistentDamageGroupInput {
                episode_id: "episode-fire".into(),
                damage: PersistentDamageInput::TableResult { amount: 4 },
            },
            PersistentDamageGroupInput {
                episode_id: "episode-acid".into(),
                damage: PersistentDamageInput::TableResult { amount: 3 },
            },
        ],
        context: HpEventContextView {
            keep_current_initiative: Some(true),
            lethality: Some(DamageLethalityView::Lethal),
            critical: Some(false),
            enemy_source: Some(true),
            responsible_participant_key: None,
            base_death_threshold: Some(4),
            ..Default::default()
        },
    }
}

#[test]
fn simultaneous_types_enter_dying_once_and_retry_does_not_repeat_damage() {
    let (f, key, participant) = fixture();
    let request = batch(&f, &key, &participant);
    let result = f
        .worker
        .apply_persistent_damage(&key, request.clone())
        .unwrap();
    assert_eq!(
        result.groups.iter().map(|group| group.amount).sum::<i64>(),
        7
    );
    let fire = result
        .groups
        .iter()
        .find(|group| group.damage_type == "fire")
        .unwrap();
    let after = snapshot(&f, &key);
    let selected = after.participants[0]
        .conditions
        .iter()
        .find(|condition| condition.condition_id == fire.source_condition_id)
        .unwrap();
    assert!(
        matches!(&selected.parameters, ConditionParameters::PersistentDamage { formula, .. } if formula == "4")
    );
    assert_eq!(after.participants[0].current_hp, Some(0));
    let dying: Vec<_> = after.participants[0]
        .conditions
        .iter()
        .filter(|condition| condition.name == "Dying")
        .collect();
    assert_eq!(dying.len(), 1);
    assert_eq!(dying[0].value, Some(1));
    assert_eq!(
        f.worker.apply_persistent_damage(&key, request).unwrap(),
        result
    );
    assert_eq!(snapshot(&f, &key), after);
}

#[test]
fn invalid_group_or_stale_revision_is_atomic() {
    let (f, key, participant) = fixture();
    let before = snapshot(&f, &key);
    let mut request = batch(&f, &key, &participant);
    request.groups.pop();
    assert!(f.worker.apply_persistent_damage(&key, request).is_err());
    assert_eq!(snapshot(&f, &key), before);
    let mut request = batch(&f, &key, &participant);
    request.expected_encounter_revision -= 1;
    assert!(f.worker.apply_persistent_damage(&key, request).is_err());
    assert_eq!(snapshot(&f, &key), before);
}

#[test]
fn a_fresh_request_id_cannot_apply_the_same_boundary_twice() {
    let (f, key, participant) = fixture();
    let result = f
        .worker
        .apply_persistent_damage(&key, batch(&f, &key, &participant))
        .unwrap();
    let after = snapshot(&f, &key);
    let mut second = batch(&f, &key, &participant);
    second.request_id = "different-request-same-boundary".into();
    assert!(f.worker.apply_persistent_damage(&key, second).is_err());
    assert_eq!(snapshot(&f, &key), after);
    let review = f
        .worker
        .persistent_damage_review(&key, &participant, None)
        .unwrap();
    assert!(review.damage_applied);
    assert_eq!(review.damage_result, Some(result));
    assert_eq!(
        review
            .episodes
            .iter()
            .map(|episode| (
                episode.episode_id.as_str(),
                episode.end_turn_recovery_available
            ))
            .collect::<Vec<_>>(),
        [("episode-acid", true), ("episode-fire", true)]
    );
}

#[test]
fn end_turn_recovery_requires_damage_but_explicit_recovery_is_not_turn_limited() {
    let (f, key, participant) = fixture();
    let mut request = RecoverPersistentDamageRequest {
        resolution: PersistentRecoveryResolution::Roll,
        boundary_key: None,
        mode: PersistentRecoveryMode::EndTurn,
        request_id: "recovery-gate".into(),
        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
        participant_key: participant.clone(),
        episode_id: "episode-fire".into(),
        dc_override: Some(100),
    };
    assert!(
        f.worker
            .recover_persistent_damage(&key, request.clone())
            .is_err()
    );
    request.mode = PersistentRecoveryMode::Explicit;
    let result = f
        .worker
        .recover_persistent_damage(&key, request.clone())
        .unwrap();
    assert!(!result.success);
    assert_eq!(
        f.worker
            .recover_persistent_damage(&key, request.clone())
            .unwrap(),
        result
    );
    request.request_id = "second-assistance".into();
    request.expected_encounter_revision = snapshot(&f, &key).encounter.revision;
    let second = f
        .worker
        .recover_persistent_damage(&key, request.clone())
        .unwrap();
    assert!(!second.success);
    assert_eq!(
        f.worker.recover_persistent_damage(&key, request).unwrap(),
        second
    );
    let review = f
        .worker
        .persistent_damage_review(&key, &participant, None)
        .unwrap();
    assert!(review.recovery_results.is_empty());
}

#[test]
fn typed_protections_preserve_missing_hardness_and_source_exceptions() {
    let (f, key, _) = fixture();
    let participant = &snapshot(&f, &key).participants[0];
    let record = |kind: &str, attributes: serde_json::Value| {
        atlas_search::test_support::record("actors","Actor",serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","type":kind,"name":"Protection","system":{"traits":{"value":[]},"attributes":attributes}})).record
    };
    let npc = record(
        "npc",
        serde_json::json!({"weaknesses":[{"type":"fire","value":5}],"resistances":[{"type":"fire","value":3}]}),
    );
    let protections = super::rules::damage::persistent_protection(
        Some(&npc),
        participant,
        super::rules::source_conditions::SourceConditions::empty(),
        "fire",
    )
    .unwrap();
    assert_eq!(protections.apply(10), 12);
    let sleep = record(
        "npc",
        serde_json::json!({"immunities":[{"type":"sleep"}],"weaknesses":[],"resistances":[]}),
    );
    assert_eq!(
        super::rules::damage::persistent_protection(
            Some(&sleep),
            participant,
            super::rules::source_conditions::SourceConditions::empty(),
            "fire"
        )
        .unwrap()
        .apply(10),
        10
    );
    let construct=atlas_search::test_support::record("actors","Actor",serde_json::json!({"_id":"constructImmune1","type":"npc","name":"Construct","system":{"traits":{"value":["construct","mindless"]},"attributes":{"immunities":[],"weaknesses":[],"resistances":[]}}})).record;
    assert!(
        super::rules::damage::persistent_protection(
            Some(&construct),
            participant,
            super::rules::source_conditions::SourceConditions::empty(),
            "bleed"
        )
        .unwrap()
        .immune
    );
    assert!(
        super::rules::damage::persistent_protection(
            Some(&construct),
            participant,
            &super::rules::source_conditions::SourceConditions::unavailable(),
            "bleed",
        )
        .unwrap()
        .immune
    );
    assert!(
        super::rules::damage::persistent_protection(
            Some(&construct),
            participant,
            &super::rules::source_conditions::SourceConditions::unavailable(),
            "fire",
        )
        .is_err()
    );
    assert_eq!(
        super::rules::damage::persistent_protection(
            Some(&construct),
            participant,
            super::rules::source_conditions::SourceConditions::empty(),
            "fire"
        )
        .unwrap()
        .apply(10),
        10
    );

    let mut bleeding = participant.clone();
    if let ConditionParameters::PersistentDamage { damage_type, .. } =
        &mut bleeding.conditions[0].parameters
    {
        *damage_type = "bleed".into();
    }
    let groups = super::persistent_damage::group_views(
        &bleeding,
        &bleeding,
        Some(&construct),
        super::rules::source_conditions::SourceConditions::empty(),
    );
    assert_eq!(
        groups
            .iter()
            .find(|group| group.damage_type == "bleed")
            .unwrap()
            .immunity_note
            .as_deref(),
        Some("Immune to bleed")
    );
    assert!(
        groups
            .iter()
            .filter(|group| group.damage_type != "bleed")
            .all(|group| group.immunity_note.is_none())
    );
    assert!(
        super::persistent_damage::group_views(
            &bleeding,
            &bleeding,
            None,
            super::rules::source_conditions::SourceConditions::empty()
        )
        .iter()
        .all(|group| group.immunity_note.is_none())
    );

    let hazard = record("hazard", serde_json::json!({}));
    assert!(
        super::rules::damage::persistent_protection(
            Some(&hazard),
            participant,
            super::rules::source_conditions::SourceConditions::empty(),
            "fire"
        )
        .is_err()
    );
    let conditional = record(
        "npc",
        serde_json::json!({"resistances":[{"type":"fire","value":3,"exceptions":["magical"]}]}),
    );
    assert!(
        super::rules::damage::persistent_protection(
            Some(&conditional),
            participant,
            super::rules::source_conditions::SourceConditions::empty(),
            "fire"
        )
        .is_err()
    );
}

#[test]
fn petrified_hardness_reduces_the_simultaneous_event_once() {
    let (f, key, _) = fixture();
    f.worker
        .add_encounter_record_participant(
            &key,
            AddEncounterRecordParticipantRequest {
                participant_variant: None,
                record_ref: "actors:testCreature1000".into(),
                quantity: 1,
                initiative: Some(8),
            },
        )
        .unwrap();
    let before = snapshot(&f, &key);
    let statue = before
        .participants
        .iter()
        .find(|participant| participant.record_key.is_some())
        .unwrap();
    f.worker
        .with_write(|session| {
            let store = session.local_state()?.encounters();
            for (condition_key, parameters) in [
                ("conditionitems:dTwPJuKgBQCMxixg", ConditionParameters::None),
                (
                    "conditionitems:lDVqvLKA6eF3Df60",
                    ConditionParameters::PersistentDamage {
                        formula: "10".into(),
                        damage_type: "fire".into(),
                        multiplier: DamageMultiplier::Normal,
                        recovery: PersistentRecovery::Standard,
                        episode_id: "fire-statue".into(),
                    },
                ),
                (
                    "conditionitems:lDVqvLKA6eF3Df60",
                    ConditionParameters::PersistentDamage {
                        formula: "10".into(),
                        damage_type: "acid".into(),
                        multiplier: DamageMultiplier::Normal,
                        recovery: PersistentRecovery::Standard,
                        episode_id: "acid-statue".into(),
                    },
                ),
            ] {
                store.add_condition(AddEncounterParticipantCondition {
                    participant_key: statue.participant_key.clone(),
                    condition_key: Some(condition_key.into()),
                    name: "Fixture".into(),
                    value: None,
                    source_participant_key: None,
                    scope: Default::default(),
                    extra_action: Default::default(),
                    immunity_override: false,
                    parameters,
                    origin: None,
                    duration: None,
                    note: None,
                })?;
            }
            Ok(())
        })
        .unwrap();
    let request = ApplyPersistentDamageRequest {
        recoveries: vec![],
        boundary_key: None,
        request_id: "statue-damage".into(),
        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
        participant_key: statue.participant_key.clone(),
        artifact_generation: f.worker.encounter(&key).unwrap().artifact_generation,
        groups: ["fire-statue", "acid-statue"]
            .into_iter()
            .map(|episode| PersistentDamageGroupInput {
                episode_id: episode.into(),

                damage: PersistentDamageInput::Roll,
            })
            .collect(),
        context: Default::default(),
    };
    let result = f.worker.apply_persistent_damage(&key, request).unwrap();
    assert!(result.groups.iter().all(|group| {
        group
            .protection
            .as_ref()
            .is_some_and(|p| !p.immune && p.weakness == 0 && p.resistance == 0)
    }));
    assert_eq!(
        result.groups.iter().map(|group| group.amount).sum::<i64>(),
        20
    );
    assert_eq!((result.total, result.hardness_absorbed), (12, 8));
    assert_eq!(
        snapshot(&f, &key)
            .participants
            .iter()
            .find(|participant| participant.participant_key == statue.participant_key)
            .unwrap()
            .current_hp,
        Some(48)
    );
}

fn source_petrified_fixture(
    active: Option<bool>,
) -> (
    FixtureWorker,
    atlas_search::test_support::FixtureArtifact,
    String,
    String,
) {
    use atlas_search::test_support::{open_source_fixture, record};
    use serde_json::json;
    let petrified = super::conditions::ConditionRule::Petrified.definition();
    let id = petrified.record_key.split_once(':').unwrap().1;
    let predicate = if active.is_some() {
        json!(["self-petrify"])
    } else {
        json!(["self:environment:unknown"])
    };
    let (retrieval, artifact) = open_source_fixture(vec![
        record("actors", "Actor", json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","name":"Source statue",
            "system":{"details":{"level":{"value":5}},"traits":{"value":[]},"attributes":{"adjustment":null,"hp":{"max":60},"immunities":[],"weaknesses":[],"resistances":[]}},
            "items":[{"_id":"bbbbbbbbbbbbbbbb","type":"action","name":"Self-Petrify","system":{"actionType":{"value":"passive"},"rules":[
                {"key":"RollOption","option":"self-petrify","toggleable":true,"value":active.unwrap_or(false)},
                {"key":"GrantItem","inMemoryOnly":true,"predicate":predicate,"uuid":format!("Compendium.pf2e.conditionitems.Item.{id}")}
            ]}}]})),
        record("conditionitems", "Item", json!({"_id":id,"type":"condition","name":"Petrified","system":{"slug":"petrified","value":{"isValued":false,"value":null}}})),
    ], false).unwrap();
    drop(retrieval);
    let f = FixtureWorker {
        worker: crate::AtlasAppService::start(crate::AtlasAppServiceOptions {
            default_search_profile: None,
            path_mode: atlas_runtime::AtlasPathMode::Repo,
            source_root: None,
            embedding_cache_root: None,
            index_path: Some(artifact.path()),
            retrieval_mode: crate::AppServiceRetrievalMode::OnDemandNoQueryInference,
        })
        .unwrap(),
    };
    let key = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Source Petrified".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter
        .encounter_key;
    f.worker
        .add_encounter_record_participant(
            &key,
            AddEncounterRecordParticipantRequest {
                record_ref: "actors:aaaaaaaaaaaaaaaa".into(),
                quantity: 1,
                participant_variant: None,
                initiative: Some(10),
            },
        )
        .unwrap();
    let participant = snapshot(&f, &key).participants[0].participant_key.clone();
    f.worker
        .with_write(|session| {
            for damage_type in ["fire", "acid", "bleed"] {
                session.local_state()?.encounters().add_condition(
                    AddEncounterParticipantCondition {
                        participant_key: participant.clone(),
                        condition_key: Some("conditionitems:lDVqvLKA6eF3Df60".into()),
                        name: "Persistent Damage".into(),
                        value: None,
                        source_participant_key: None,
                        scope: Default::default(),
                        extra_action: Default::default(),
                        immunity_override: false,
                        parameters: ConditionParameters::PersistentDamage {
                            formula: "10".into(),
                            damage_type: damage_type.into(),
                            multiplier: DamageMultiplier::Normal,
                            recovery: PersistentRecovery::Standard,
                            episode_id: format!("statue-{damage_type}"),
                        },
                        origin: None,
                        duration: None,
                        note: None,
                    },
                )?;
            }
            Ok(())
        })
        .unwrap();
    (f, artifact, key, participant)
}

fn source_statue_damage(
    f: &FixtureWorker,
    key: &str,
    participant: &str,
) -> ApplyPersistentDamageRequest {
    ApplyPersistentDamageRequest {
        request_id: "source-statue-damage".into(),
        expected_encounter_revision: snapshot(f, key).encounter.revision,
        participant_key: participant.into(),
        boundary_key: None,
        artifact_generation: f.worker.encounter(key).unwrap().artifact_generation,
        groups: ["fire", "acid", "bleed"]
            .into_iter()
            .map(|damage_type| PersistentDamageGroupInput {
                episode_id: format!("statue-{damage_type}"),
                damage: PersistentDamageInput::Roll,
            })
            .collect(),
        recoveries: vec![],
        context: Default::default(),
    }
}

fn switch_source_petrified_off(f: &FixtureWorker, key: &str) {
    let detail = f.worker.encounter(key).unwrap();
    let participant = &detail.participants[0];
    let ability = participant
        .ability_states
        .as_ref()
        .unwrap()
        .first()
        .unwrap();
    f.worker
        .update_encounter_ability_state(
            key,
            &participant.participant_key,
            UpdateEncounterAbilityStateRequest {
                request_id: "switch-petrified-off".into(),
                expected_encounter_revision: detail.revision,
                expected_participant_revision: participant.revision,
                navigation: ability.navigation.clone(),
                rule_ordinal: ability.rule_ordinal,
                action: EncounterAbilityStateAction::SetOption {
                    active: false,
                    selection: None,
                },
            },
        )
        .unwrap();
}

fn damage_draft(
    preparation: &PreparedPersistentDamageView,
    episode_id: &str,
) -> PersistentDamageInput {
    let entry = preparation.damage_draft.as_ref().unwrap();
    assert_eq!(entry.episode_id, episode_id);
    let outcome = preparation
        .damage
        .groups
        .iter()
        .find(|group| group.episode_id == episode_id)
        .unwrap();
    assert_eq!(entry.draft.source_condition_id, outcome.source_condition_id);
    assert_eq!(Some(&entry.draft.roll), outcome.roll.as_ref());
    PersistentDamageInput::Prepared {
        draft: entry.draft.clone(),
    }
}

#[test]
fn source_petrified_protects_prepared_damage_and_the_final_packet_once() {
    let (f, _artifact, key, participant) = source_petrified_fixture(Some(true));
    let before = snapshot(&f, &key);
    let detail = f.worker.encounter(&key).unwrap();
    assert_eq!(
        detail.participants[0]
            .persistent_damage_groups
            .iter()
            .find(|group| group.damage_type == "bleed")
            .unwrap()
            .immunity_note
            .as_deref(),
        Some("Immune to bleed")
    );
    let review = f
        .worker
        .persistent_damage_review(&key, &participant, None)
        .unwrap();
    assert!(review.episodes.iter().any(|episode| episode.episode_id == "statue-bleed" && matches!(&episode.damage_resolution, PersistentDamageResolutionView::Known { protection } if protection.immune)));
    let mut request = source_statue_damage(&f, &key, &participant);
    let prepared = f
        .worker
        .prepare_persistent_damage(
            &key,
            PreparePersistentDamageRequest {
                request_id: "prepare-source-statue-fire".into(),
                expected_encounter_revision: before.encounter.revision,
                participant_key: participant.clone(),
                boundary_key: None,
                artifact_generation: request.artifact_generation.clone(),
                groups: vec![request.groups[0].clone()],
                recovery: None,
            },
        )
        .unwrap();
    assert_eq!(
        (prepared.damage.total, prepared.damage.hardness_absorbed),
        (2, 8)
    );
    assert_eq!(snapshot(&f, &key), before);
    request.groups[0].damage = damage_draft(&prepared, &request.groups[0].episode_id);
    let result = f
        .worker
        .apply_persistent_damage(&key, request.clone())
        .unwrap();
    assert_eq!((result.total, result.hardness_absorbed), (12, 8));
    assert_eq!(
        result
            .groups
            .iter()
            .find(|group| group.damage_type == "bleed")
            .unwrap()
            .amount,
        0
    );
    assert_eq!(snapshot(&f, &key).participants[0].current_hp, Some(48));
    assert_eq!(
        f.worker.apply_persistent_damage(&key, request).unwrap(),
        result
    );
    assert_eq!(snapshot(&f, &key).participants[0].current_hp, Some(48));
}

#[test]
fn source_petrified_off_preserves_damage_and_temporary_petrified_has_the_same_protections() {
    for (temporary, expected_total, expected_hardness) in [(false, 30, 0), (true, 12, 8)] {
        let (f, _artifact, key, participant) = source_petrified_fixture(Some(false));
        if temporary {
            f.worker
                .with_write(|session| {
                    session.local_state()?.encounters().add_condition(
                        AddEncounterParticipantCondition {
                            participant_key: participant.clone(),
                            condition_key: Some(
                                super::conditions::ConditionRule::Petrified
                                    .definition()
                                    .record_key
                                    .into(),
                            ),
                            name: "Petrified".into(),
                            value: None,
                            source_participant_key: None,
                            scope: Default::default(),
                            extra_action: Default::default(),
                            immunity_override: false,
                            parameters: ConditionParameters::None,
                            origin: None,
                            duration: None,
                            note: None,
                        },
                    )?;
                    Ok(())
                })
                .unwrap();
        }
        let result = f
            .worker
            .apply_persistent_damage(&key, source_statue_damage(&f, &key, &participant))
            .unwrap();
        assert_eq!(
            (result.total, result.hardness_absorbed),
            (expected_total, expected_hardness)
        );
    }
}

#[test]
fn prepared_persistent_damage_revalues_source_petrified_after_switching_off() {
    let (f, _artifact, key, participant) = source_petrified_fixture(Some(true));
    let mut request = source_statue_damage(&f, &key, &participant);
    let prepared = f
        .worker
        .prepare_persistent_damage(
            &key,
            PreparePersistentDamageRequest {
                request_id: "prepare-before-switching-off".into(),
                expected_encounter_revision: request.expected_encounter_revision,
                participant_key: participant.clone(),
                boundary_key: None,
                artifact_generation: request.artifact_generation.clone(),
                groups: vec![request.groups[0].clone()],
                recovery: None,
            },
        )
        .unwrap();
    switch_source_petrified_off(&f, &key);
    request.expected_encounter_revision = snapshot(&f, &key).encounter.revision;
    request.groups[0].damage = damage_draft(&prepared, &request.groups[0].episode_id);
    let result = f.worker.apply_persistent_damage(&key, request).unwrap();
    assert_eq!((result.total, result.hardness_absorbed), (30, 0));
}

#[test]
fn prepared_damage_expires_across_reference_generations_with_the_same_formula() {
    let (f, _artifact, key, participant) = source_petrified_fixture(Some(true));
    let before = snapshot(&f, &key);
    let mut apply = source_statue_damage(&f, &key, &participant);
    let prepare = |generation, groups| PreparePersistentDamageRequest {
        request_id: "prepare-across-reference-updates".into(),
        expected_encounter_revision: before.encounter.revision,
        participant_key: participant.clone(),
        boundary_key: None,
        artifact_generation: generation,
        groups,
        recovery: None,
    };
    let rolled = f
        .worker
        .prepare_persistent_damage(
            &key,
            prepare(
                apply.artifact_generation.clone(),
                vec![apply.groups[0].clone()],
            ),
        )
        .unwrap();
    let draft_context = &rolled.damage_draft.as_ref().unwrap().draft.context;
    assert_eq!(draft_context.artifact_generation, apply.artifact_generation);
    assert_eq!(draft_context.record_key, before.participants[0].record_key);

    let workspace = atlas_runtime::initialize_workspace(f.worker.workspace_path()).unwrap();
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
    apply.artifact_generation = f.worker.encounter(&key).unwrap().artifact_generation;
    assert_ne!(draft_context.artifact_generation, apply.artifact_generation);
    let stale = damage_draft(&rolled, &apply.groups[0].episode_id);
    let mut stale_preview = apply.groups[0].clone();
    stale_preview.damage = stale.clone();
    let err = f
        .worker
        .prepare_persistent_damage(
            &key,
            prepare(apply.artifact_generation.clone(), vec![stale_preview]),
        )
        .unwrap_err();
    assert!(
        err.into_app_error()
            .message
            .contains("Prepared damage source or turn changed")
    );
    let current = f
        .worker
        .prepare_persistent_damage(
            &key,
            prepare(
                apply.artifact_generation.clone(),
                vec![apply.groups[0].clone()],
            ),
        )
        .unwrap();
    assert_eq!(rolled.damage.groups[0].roll, current.damage.groups[0].roll);
    apply.groups[0].damage = stale;
    let err = f
        .worker
        .apply_persistent_damage(&key, apply.clone())
        .unwrap_err();
    assert!(
        err.into_app_error()
            .message
            .contains("Prepared damage source or turn changed")
    );
    assert_eq!(snapshot(&f, &key), before);
    apply.groups[0].damage = damage_draft(&current, &apply.groups[0].episode_id);
    assert!(f.worker.apply_persistent_damage(&key, apply).is_ok());
}

#[test]
fn outgoing_damage_keeps_frozen_episodes_and_uses_current_source_protections() {
    let (f, _artifact, key, participant) = source_petrified_fixture(Some(true));
    let boundary = outgoing_boundary(&f, &key, &participant);
    switch_source_petrified_off(&f, &key);
    f.worker
        .with_write(|session| {
            session.local_state()?.encounters().add_condition(
                AddEncounterParticipantCondition {
                    participant_key: participant.clone(),
                    condition_key: Some("conditionitems:lDVqvLKA6eF3Df60".into()),
                    name: "New persistent cold".into(),
                    value: None,
                    source_participant_key: None,
                    scope: Default::default(),
                    extra_action: Default::default(),
                    immunity_override: false,
                    parameters: ConditionParameters::PersistentDamage {
                        formula: "10".into(),
                        damage_type: "cold".into(),
                        multiplier: DamageMultiplier::Normal,
                        recovery: PersistentRecovery::Standard,
                        episode_id: "late-cold".into(),
                    },
                    origin: None,
                    duration: None,
                    note: None,
                },
            )?;
            Ok(())
        })
        .unwrap();
    let review = f
        .worker
        .persistent_damage_review(&key, &participant, Some(&boundary))
        .unwrap();
    assert_eq!(review.damage_groups.len(), 3);
    assert!(
        review
            .damage_groups
            .iter()
            .all(|group| group.episode_id != "late-cold")
    );
    assert_eq!(
        review
            .damage_groups
            .iter()
            .find(|group| group.damage_type == "bleed")
            .unwrap()
            .immunity_note,
        None
    );
    assert!(review.episodes.iter().any(|episode| episode.episode_id == "statue-bleed" && matches!(&episode.damage_resolution, PersistentDamageResolutionView::Known { protection } if !protection.immune)));
    let mut request = source_statue_damage(&f, &key, &participant);
    request.boundary_key = Some(boundary);
    let result = f.worker.apply_persistent_damage(&key, request).unwrap();
    assert_eq!((result.total, result.hardness_absorbed), (30, 0));
    assert_eq!(snapshot(&f, &key).participants[0].current_hp, Some(30));
    assert!(snapshot(&f, &key).participants[0].conditions.iter().any(|condition| matches!(&condition.parameters, ConditionParameters::PersistentDamage { episode_id, .. } if episode_id == "late-cold")));
}

#[test]
fn uncertain_source_petrified_requires_table_results_without_reapplying_protections() {
    let (f, _artifact, key, participant) = source_petrified_fixture(None);
    let before = snapshot(&f, &key);
    let review = f
        .worker
        .persistent_damage_review(&key, &participant, None)
        .unwrap();
    assert_eq!(review.episodes.len(), 3);
    assert!(review.episodes.iter().all(|episode| matches!(
        episode.damage_resolution,
        PersistentDamageResolutionView::Table { .. }
    )));
    let mut request = source_statue_damage(&f, &key, &participant);
    assert!(
        f.worker
            .apply_persistent_damage(&key, request.clone())
            .is_err()
    );
    assert_eq!(snapshot(&f, &key), before);
    for group in &mut request.groups {
        group.damage = PersistentDamageInput::TableResult { amount: 1 };
    }
    request.artifact_generation = None;
    let result = f.worker.apply_persistent_damage(&key, request).unwrap();
    assert_eq!((result.total, result.hardness_absorbed), (3, 0));
    assert!(result.groups.iter().all(|group| group.protection.is_none()));
}

#[test]
fn recovery_clears_suppressed_sources_and_recovers_receipt_after_episode_is_gone() {
    let (f, key, participant) = fixture();
    let request = RecoverPersistentDamageRequest {
        resolution: PersistentRecoveryResolution::Roll,
        boundary_key: None,
        mode: PersistentRecoveryMode::Explicit,
        request_id: "recover-fire".into(),
        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
        participant_key: participant,
        episode_id: "episode-fire".into(),
        dc_override: Some(0),
    };
    let result = f
        .worker
        .recover_persistent_damage(&key, request.clone())
        .unwrap();
    assert!(result.success);
    let after = snapshot(&f, &key);
    assert_eq!(after.participants[0].conditions.len(), 1);
    assert!(
        matches!(&after.participants[0].conditions[0].parameters,ConditionParameters::PersistentDamage {damage_type,..} if damage_type=="acid")
    );
    assert_eq!(after.participants[0].current_hp, Some(5));
    assert_eq!(
        f.worker.recover_persistent_damage(&key, request).unwrap(),
        result
    );
    assert_eq!(snapshot(&f, &key), after);
}

fn outgoing_boundary(f: &FixtureWorker, key: &str, participant: &str) -> String {
    f.worker
        .set_encounter_turn(
            key,
            SetEncounterTurnRequest {
                participant_key: participant.into(),
            },
        )
        .unwrap();
    f.worker
        .advance_encounter_turn(
            key,
            AdvanceEncounterTurnRequest {
                request_id: "advance-outgoing".into(),
                expected_revision: snapshot(f, key).encounter.revision,
            },
        )
        .unwrap();
    f.worker
        .encounter_turn_review(key)
        .unwrap()
        .outgoing
        .unwrap()
        .boundary_key
        .clone()
}
#[test]
fn late_recovery_never_clears_a_new_same_type_episode() {
    let (f, key, participant) = fixture();
    f.worker
        .apply_persistent_damage(&key, batch(&f, &key, &participant))
        .unwrap();
    let hp_after_damage = snapshot(&f, &key).participants[0].current_hp;
    let boundary = outgoing_boundary(&f, &key, &participant);
    let old=snapshot(&f,&key).participants[0].conditions.iter().filter(|c|matches!(&c.parameters,ConditionParameters::PersistentDamage{damage_type,..} if damage_type=="fire")).map(|c|c.condition_id).collect::<Vec<_>>();
    f.worker
        .with_write(|session| {
            let store = session.local_state()?.encounters();
            for id in old {
                store.remove_condition(&participant, id)?;
            }
            store.add_condition(AddEncounterParticipantCondition {
                participant_key: participant.clone(),
                condition_key: Some("conditionitems:lDVqvLKA6eF3Df60".into()),
                name: "Persistent Damage".into(),
                value: None,
                source_participant_key: None,
                scope: Default::default(),
                extra_action: Default::default(),
                immunity_override: false,
                parameters: ConditionParameters::PersistentDamage {
                    formula: "99".into(),
                    damage_type: "fire".into(),
                    multiplier: DamageMultiplier::Normal,
                    recovery: PersistentRecovery::SourceDependent,
                    episode_id: "new-fire".into(),
                },
                origin: None,
                duration: None,
                note: None,
            })?;
            Ok(())
        })
        .unwrap();
    let request = RecoverPersistentDamageRequest {
        resolution: PersistentRecoveryResolution::Roll,
        boundary_key: Some(boundary),
        mode: PersistentRecoveryMode::EndTurn,
        request_id: "late-success".into(),
        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
        participant_key: participant.clone(),
        episode_id: "episode-fire".into(),
        dc_override: Some(0),
    };
    let result = f
        .worker
        .recover_persistent_damage(&key, request.clone())
        .unwrap();
    assert!(result.success);
    assert!(result.roll.is_none());
    assert_eq!(
        f.worker
            .recover_persistent_damage(&key, request.clone())
            .unwrap(),
        result
    );
    let state = snapshot(&f, &key);
    assert!(state.participants[0].conditions.iter().any(|c|matches!(&c.parameters,ConditionParameters::PersistentDamage{episode_id,..} if episode_id=="new-fire")));
    assert_eq!(state.participants[0].current_hp, hp_after_damage);
    assert!(
        f.worker
            .recover_persistent_damage(
                &key,
                RecoverPersistentDamageRequest {
                    resolution: PersistentRecoveryResolution::Roll,
                    request_id: "duplicate-claim".into(),
                    expected_encounter_revision: state.encounter.revision,
                    ..request
                }
            )
            .is_err()
    );
    let manual = RecoverPersistentDamageRequest {
        resolution: PersistentRecoveryResolution::Roll,
        boundary_key: None,
        mode: PersistentRecoveryMode::Explicit,
        request_id: "unknown-dc-success".into(),
        expected_encounter_revision: state.encounter.revision,
        participant_key: participant.clone(),
        episode_id: "new-fire".into(),
        dc_override: Some(0),
    };
    let result = f
        .worker
        .recover_persistent_damage(&key, manual.clone())
        .unwrap();
    assert!(result.success);
    assert_eq!(result.dc, Some(0));
    assert!(result.roll.is_none());
    assert_eq!(
        f.worker.recover_persistent_damage(&key, manual).unwrap(),
        result
    );
}
#[test]
fn late_damage_uses_frozen_due_formulas_and_not_new_episode_values() {
    let (f, key, participant) = fixture();
    let boundary = outgoing_boundary(&f, &key, &participant);
    let mut request = batch(&f, &key, &participant);
    request.boundary_key = Some(boundary.clone());
    let due = f
        .worker
        .persistent_damage_review(&key, &participant, Some(&boundary))
        .unwrap();
    assert_eq!(due.round_number, 1);
    assert_eq!(snapshot(&f, &key).encounter.round_number, 2);
    let fire=snapshot(&f,&key).participants[0].conditions.iter().find(|c|matches!(&c.parameters,ConditionParameters::PersistentDamage{damage_type,..} if damage_type=="fire")).unwrap().condition_id;
    f.worker
        .with_write(|session| {
            session
                .local_state()?
                .encounters()
                .remove_condition(&participant, fire)?;
            Ok(())
        })
        .unwrap();
    let frozen = f
        .worker
        .encounter_turn_review(&key)
        .unwrap()
        .outgoing
        .unwrap()
        .clone();
    assert!(
        frozen
            .persistent_damage_groups
            .iter()
            .flat_map(|g| &g.sources)
            .any(|source| source.condition_id == fire && source.formula == "4")
    );
    request.expected_encounter_revision = snapshot(&f, &key).encounter.revision;
    let result = f
        .worker
        .apply_persistent_damage(&key, request.clone())
        .unwrap();
    assert_eq!(result.total, 7);
    assert_eq!(
        f.worker.apply_persistent_damage(&key, request).unwrap(),
        result
    );
    assert!(
        f.worker
            .persistent_damage_review(&key, &participant, Some(&boundary))
            .unwrap()
            .damage_applied
    );
}

#[test]
fn partial_persistent_reminder_round_trips_without_joining_automation_then_can_be_completed() {
    let (f, key, participant) = fixture();
    let initial = f.worker.encounter(&key).unwrap();
    f.worker
        .add_encounter_participant_condition(
            &key,
            &participant,
            AddEncounterParticipantConditionRequest {
                expected_reference_revision: None,
                request_id: "reminder-add".into(),
                expected_encounter_revision: initial.revision,
                condition_ref: Some("conditionitems:lDVqvLKA6eF3Df60".into()),
                name: None,
                value: None,
                source_participant_key: None,
                duration: None,
                note: Some("Burning until the blessing ends".into()),
                event_intent: Default::default(),
                decisions: Default::default(),
                scope: Default::default(),
                extra_action: Default::default(),
                immunity_override: false,
                origin: None,
                parameters: ConditionParametersInputView::PersistentDamage {
                    formula: None,
                    damage_type: Some("fire".into()),
                    multiplier: DamageMultiplierView::Normal,
                    recovery: PersistentRecoveryView::Standard,
                },
            },
        )
        .unwrap();
    let detail = f.worker.encounter(&key).unwrap();
    let actor = &detail.participants[0];
    assert_eq!(
        actor.persistent_damage_groups,
        initial.participants[0].persistent_damage_groups
    );
    let reminder = actor
        .conditions
        .iter()
        .find(|c| {
            matches!(
                c.parameters,
                ConditionParametersView::PersistentDamageReminder { .. }
            )
        })
        .unwrap();
    let saved = snapshot(&f, &key);
    assert!(
        matches!(&saved.participants[0].conditions.iter().find(|c| c.condition_id == reminder.condition_id).unwrap().parameters,
        ConditionParameters::PersistentDamageReminder { formula: None, damage_type: Some(kind), .. } if kind == "fire")
    );
    f.worker
        .update_encounter_participant_condition(
            &key,
            &participant,
            reminder.condition_id,
            UpdateEncounterParticipantConditionRequest {
                expected_reference_revision: None,
                request_id: "reminder-complete".into(),
                expected_encounter_revision: detail.revision,
                condition_ref: reminder.condition_key.clone(),
                name: reminder.name.clone(),
                value: None,
                source_participant_key: None,
                duration: None,
                note: reminder.note.clone(),
                event_intent: Default::default(),
                decisions: Default::default(),
                scope: Default::default(),
                extra_action: Default::default(),
                immunity_override: false,
                origin: None,
                parameters: ConditionParametersInputView::PersistentDamage {
                    formula: Some("1d6".into()),
                    damage_type: Some("fire".into()),
                    multiplier: DamageMultiplierView::Normal,
                    recovery: PersistentRecoveryView::Standard,
                },
            },
        )
        .unwrap();
    let completed = f.worker.encounter(&key).unwrap();
    let group = completed.participants[0]
        .persistent_damage_groups
        .iter()
        .find(|g| g.damage_type == "fire")
        .unwrap();
    assert_eq!(group.sources.len(), 3);
    assert!(
        group
            .sources
            .iter()
            .any(|s| s.condition_id == reminder.condition_id)
    );
    assert_eq!(
        completed.participants[0]
            .conditions
            .iter()
            .find(|c| c.condition_id == reminder.condition_id)
            .unwrap()
            .note,
        reminder.note
    );
}

#[test]
fn persistent_edits_preserve_or_admit_episode_identity_without_browser_authority() {
    let (f, key, participant) = fixture();
    let saved = snapshot(&f, &key);
    let source = &saved.participants[0].conditions[0];
    let edit = |id: &str, kind: &str, revision| UpdateEncounterParticipantConditionRequest {
        expected_reference_revision: None,
        request_id: id.into(),
        expected_encounter_revision: revision,
        condition_ref: source.condition_key.clone(),
        name: source.name.clone(),
        value: None,
        source_participant_key: None,
        scope: Default::default(),
        extra_action: Default::default(),
        immunity_override: false,
        origin: None,
        duration: None,
        note: Some("new note".into()),
        event_intent: Default::default(),
        decisions: Default::default(),
        parameters: ConditionParametersInputView::PersistentDamage {
            formula: Some("4".into()),
            damage_type: Some(kind.into()),
            multiplier: DamageMultiplierView::Normal,
            recovery: PersistentRecoveryView::Standard,
        },
    };
    f.worker
        .update_encounter_participant_condition(
            &key,
            &participant,
            source.condition_id,
            edit("note", "fire", saved.encounter.revision),
        )
        .unwrap();
    let after_note = snapshot(&f, &key);
    let edited = after_note.participants[0]
        .conditions
        .iter()
        .find(|row| row.condition_id == source.condition_id)
        .unwrap();
    assert!(
        matches!(&edited.parameters, ConditionParameters::PersistentDamage { episode_id, recovery: PersistentRecovery::Standard, .. } if episode_id == "episode-fire")
    );
    f.worker
        .update_encounter_participant_condition(
            &key,
            &participant,
            source.condition_id,
            edit("new-type", "cold", after_note.encounter.revision),
        )
        .unwrap();
    let after_type = snapshot(&f, &key);
    let edited = after_type.participants[0]
        .conditions
        .iter()
        .find(|row| row.condition_id == source.condition_id)
        .unwrap();
    assert!(
        matches!(&edited.parameters, ConditionParameters::PersistentDamage { episode_id, damage_type, .. } if episode_id == "new-type:cold" && damage_type == "cold")
    );
    assert!(after_type.participants[0].conditions.iter().any(|row| matches!(&row.parameters, ConditionParameters::PersistentDamage { episode_id, damage_type, .. } if episode_id == "episode-fire" && damage_type == "fire")));
}

#[test]
fn table_clear_needs_no_dc_source_choice_or_damage_and_cannot_repeat_damage() {
    let (f, key, participant) = fixture();
    let before = snapshot(&f, &key).participants[0].current_hp;
    let request = RecoverPersistentDamageRequest {
        resolution: PersistentRecoveryResolution::Clear,
        boundary_key: None,
        mode: PersistentRecoveryMode::EndTurn,
        request_id: "clear-fire-at-table".into(),
        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
        participant_key: participant.clone(),
        episode_id: "episode-fire".into(),
        dc_override: None,
    };
    let receipt = f
        .worker
        .recover_persistent_damage(&key, request.clone())
        .unwrap();
    assert!(receipt.success);
    assert!(receipt.dc.is_none());
    assert!(receipt.roll.is_none());
    assert_eq!(snapshot(&f, &key).participants[0].current_hp, before);
    let review = f
        .worker
        .persistent_damage_review(&key, &participant, None)
        .unwrap();
    assert_eq!(review.damage_groups.len(), 1);
    assert_eq!(review.damage_groups[0].damage_type, "acid");
    assert_eq!(
        f.worker.recover_persistent_damage(&key, request).unwrap(),
        receipt
    );
    let mut request = batch(&f, &key, &participant);
    request
        .groups
        .retain(|group| group.episode_id == "episode-acid");
    let damage = f.worker.apply_persistent_damage(&key, request).unwrap();
    assert_eq!(damage.total, 3);
    assert_eq!(snapshot(&f, &key).participants[0].current_hp, Some(2));
}

#[test]
fn outgoing_clear_preserves_later_fire_sources_and_excludes_ended_fire_from_damage() {
    let (f, key, participant) = fixture();
    let boundary = outgoing_boundary(&f, &key, &participant);
    f.worker.with_write(|session| {
        let mut later = snapshot(&f, &key).participants[0].conditions.iter().find(|source|
            matches!(&source.parameters,ConditionParameters::PersistentDamage {damage_type,..} if damage_type == "fire")).unwrap().clone();
        later.condition_id = 0;
        if let ConditionParameters::PersistentDamage {formula, ..} = &mut later.parameters { *formula = "99".into(); }
        session.local_state()?.encounters().add_condition(atlas_local_state::AddEncounterParticipantCondition {
            participant_key: participant.clone(), condition_key: later.condition_key, name: later.name, value: None,
            source_participant_key: None, scope: Default::default(), extra_action: Default::default(), immunity_override:false,
            parameters:later.parameters, origin:None, duration:None, note:Some("After turn".into()),
        })?;
        Ok(())
    }).unwrap();
    let request = RecoverPersistentDamageRequest {
        resolution: PersistentRecoveryResolution::Clear,
        boundary_key: Some(boundary.clone()),
        mode: PersistentRecoveryMode::EndTurn,
        request_id: "clear-outgoing-fire".into(),
        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
        participant_key: participant.clone(),
        episode_id: "episode-fire".into(),
        dc_override: None,
    };
    let receipt = f
        .worker
        .recover_persistent_damage(&key, request.clone())
        .unwrap();
    let after = snapshot(&f, &key);
    assert_eq!(after.participants[0].current_hp, Some(5));
    assert_eq!(after.participants[0].conditions.len(), 2);
    assert!(
        after.participants[0]
            .conditions
            .iter()
            .any(|source| source.note.as_deref() == Some("After turn"))
    );
    let review = f
        .worker
        .persistent_damage_review(&key, &participant, Some(&boundary))
        .unwrap();
    assert_eq!(review.damage_groups.len(), 1);
    assert_eq!(review.damage_groups[0].damage_type, "acid");
    assert_eq!(
        f.worker.recover_persistent_damage(&key, request).unwrap(),
        receipt
    );
    assert_eq!(snapshot(&f, &key), after);
    let mut request = batch(&f, &key, &participant);
    request.boundary_key = Some(boundary);
    request
        .groups
        .retain(|group| group.episode_id == "episode-acid");
    let damage = f.worker.apply_persistent_damage(&key, request).unwrap();
    assert_eq!(damage.total, 3);
    assert!(
        snapshot(&f, &key).participants[0]
            .conditions
            .iter()
            .any(|source| source.note.as_deref() == Some("After turn"))
    );
}

#[test]
fn persistent_precedence_is_atomic_and_retains_both_ambiguous_fallback_pairs() {
    use PersistentDamageSourceRefView as Choice;
    let (f, key, participant) = fixture();
    for source in snapshot(&f, &key).participants[0].conditions.clone() {
        if matches!(&source.parameters, ConditionParameters::PersistentDamage {damage_type, ..} if damage_type == "fire")
        {
            f.worker
                .remove_encounter_participant_condition(
                    &key,
                    &participant,
                    source.condition_id,
                    RemoveEncounterParticipantConditionRequest {
                        request_id: format!("remove-{}", source.condition_id),
                        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
                        intent: Default::default(),
                    },
                )
                .unwrap();
        }
    }
    let request = |id: &str, formula: &str| AddEncounterParticipantConditionRequest {
        expected_reference_revision: None,
        request_id: id.into(),
        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
        condition_ref: Some("conditionitems:lDVqvLKA6eF3Df60".into()),
        name: None,
        value: None,
        source_participant_key: None,
        duration: None,
        note: None,
        event_intent: Default::default(),
        decisions: Default::default(),
        scope: Default::default(),
        extra_action: Default::default(),
        immunity_override: false,
        origin: None,
        parameters: ConditionParametersInputView::PersistentDamage {
            formula: Some(formula.into()),
            damage_type: Some("fire".into()),
            multiplier: DamageMultiplierView::Normal,
            recovery: PersistentRecoveryView::Standard,
        },
    };
    let first = request("d12", "1d12");
    assert!(
        f.worker
            .preview_encounter_condition_add(&key, &participant, first.clone())
            .unwrap()
            .persistent_precedence
            .is_none()
    );
    f.worker
        .add_encounter_participant_condition(&key, &participant, first)
        .unwrap();
    let d12 = f.worker.encounter(&key).unwrap().participants[0].conditions.iter().find(|c| matches!(&c.parameters, ConditionParametersView::PersistentDamage{formula,..} if formula == "1d12")).unwrap().condition_id;
    let mut second = request("2d6", "2d6");
    let before = snapshot(&f, &key);
    let choice = f
        .worker
        .preview_encounter_condition_add(&key, &participant, second.clone())
        .unwrap()
        .persistent_precedence
        .unwrap();
    assert_eq!(choice.sources[0].source, Choice::Edited);
    assert_eq!(choice.sources.len(), 2);
    assert_eq!(
        snapshot(&f, &key).encounter.revision,
        before.encounter.revision
    );
    let acid = before.participants[0].conditions.iter().find(|c| matches!(&c.parameters,ConditionParameters::PersistentDamage{damage_type,..} if damage_type == "acid")).unwrap().condition_id;
    second.decisions.persistent_precedence = Some(vec![
        Choice::Existing { condition_id: acid },
        Choice::Edited,
    ]);
    assert!(
        f.worker
            .add_encounter_participant_condition(&key, &participant, second.clone())
            .is_err()
    );
    assert_eq!(
        snapshot(&f, &key).encounter.revision,
        before.encounter.revision
    );
    second.decisions.persistent_precedence =
        Some(vec![Choice::Existing { condition_id: d12 }, Choice::Edited]);
    let receipt = f
        .worker
        .add_encounter_participant_condition(&key, &participant, second.clone())
        .unwrap();
    assert_eq!(
        f.worker
            .add_encounter_participant_condition(&key, &participant, second)
            .unwrap(),
        receipt
    );
    let detail = f.worker.encounter(&key).unwrap();
    let fire = detail.participants[0]
        .persistent_damage_groups
        .iter()
        .find(|g| g.damage_type == "fire")
        .unwrap();
    assert_eq!(fire.selected_source_id, d12);
    let d6 = fire
        .sources
        .iter()
        .find(|s| s.formula == "2d6")
        .unwrap()
        .condition_id;
    let source = detail.participants[0]
        .conditions
        .iter()
        .find(|c| c.condition_id == d6)
        .unwrap();
    let edit = UpdateEncounterParticipantConditionRequest {
        expected_reference_revision: None,
        request_id: "edit-choice".into(),
        expected_encounter_revision: detail.revision,
        condition_ref: source.condition_key.clone(),
        name: source.name.clone(),
        value: source.value,
        source_participant_key: None,
        scope: source.scope.clone(),
        extra_action: source.extra_action.clone(),
        immunity_override: false,
        origin: source.origin.clone(),
        duration: None,
        note: None,
        event_intent: Default::default(),
        decisions: Default::default(),
        parameters: ConditionParametersInputView::PersistentDamage {
            formula: Some("2d8".into()),
            damage_type: Some("fire".into()),
            multiplier: DamageMultiplierView::Normal,
            recovery: PersistentRecoveryView::Standard,
        },
    };
    assert_eq!(
        f.worker
            .preview_encounter_condition_update(&key, &participant, d6, edit.clone())
            .unwrap()
            .persistent_precedence
            .unwrap()
            .sources[0]
            .source,
        Choice::Edited
    );
    f.worker
        .update_encounter_participant_condition(&key, &participant, d6, edit)
        .unwrap();
    let detail = f.worker.encounter(&key).unwrap();
    assert_eq!(
        detail.participants[0]
            .persistent_damage_groups
            .iter()
            .find(|g| g.damage_type == "fire")
            .unwrap()
            .selected_source_id,
        d6
    );
    f.worker
        .remove_encounter_participant_condition(
            &key,
            &participant,
            d6,
            RemoveEncounterParticipantConditionRequest {
                request_id: "remove-selected".into(),
                expected_encounter_revision: detail.revision,
                intent: Default::default(),
            },
        )
        .unwrap();
    assert_eq!(
        f.worker.encounter(&key).unwrap().participants[0]
            .persistent_damage_groups
            .iter()
            .find(|g| g.damage_type == "fire")
            .unwrap()
            .selected_source_id,
        d12
    );
    // Rebuild the lower ambiguous pair with the GM's preferred 1d12 first.
    let mut lower = request("lower-pair", "2d6");
    lower.decisions.persistent_precedence =
        Some(vec![Choice::Existing { condition_id: d12 }, Choice::Edited]);
    f.worker
        .add_encounter_participant_condition(&key, &participant, lower)
        .unwrap();
    f.worker
        .add_encounter_participant_condition(&key, &participant, request("higher-d12", "2d12"))
        .unwrap();
    let detail = f.worker.encounter(&key).unwrap();
    let higher_d12 = detail.participants[0]
        .persistent_damage_groups
        .iter()
        .find(|g| g.damage_type == "fire")
        .unwrap()
        .sources
        .iter()
        .find(|s| s.formula == "2d12")
        .unwrap()
        .condition_id;
    let lower_d6 = detail.participants[0]
        .persistent_damage_groups
        .iter()
        .find(|g| g.damage_type == "fire")
        .unwrap()
        .sources
        .iter()
        .find(|s| s.formula == "2d6")
        .unwrap()
        .condition_id;
    let mut higher = request("higher-pair", "4d6");
    // Reject duplicate/missing members without changing the recorded state.
    let revision = higher.expected_encounter_revision;
    higher.decisions.persistent_precedence = Some(vec![Choice::Edited, Choice::Edited]);
    assert!(
        f.worker
            .add_encounter_participant_condition(&key, &participant, higher.clone())
            .is_err()
    );
    assert_eq!(snapshot(&f, &key).encounter.revision, revision);
    higher.decisions.persistent_precedence = Some(vec![
        Choice::Existing {
            condition_id: higher_d12,
        },
        Choice::Edited,
        Choice::Existing { condition_id: d12 },
        Choice::Existing {
            condition_id: lower_d6,
        },
    ]);
    f.worker
        .add_encounter_participant_condition(&key, &participant, higher)
        .unwrap();
    let assert_order = |expected: &[&str]| {
        let detail = f.worker.encounter(&key).unwrap();
        let fire = detail.participants[0]
            .persistent_damage_groups
            .iter()
            .find(|g| g.damage_type == "fire")
            .unwrap();
        assert_eq!(
            fire.sources
                .iter()
                .map(|s| s.formula.as_str())
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(fire.selected_source_id, fire.sources[0].condition_id);
    };
    assert_order(&["2d12", "4d6", "1d12", "2d6"]);
    // A uniquely strongest source still exposes precedence for ambiguous fallbacks.
    let strongest = request("uniquely-strongest", "6d12");
    let preview = f
        .worker
        .preview_encounter_condition_add(&key, &participant, strongest.clone())
        .unwrap()
        .persistent_precedence
        .unwrap();
    assert_eq!(
        preview
            .sources
            .iter()
            .map(|s| s.formula.as_str())
            .collect::<Vec<_>>(),
        ["6d12", "2d12", "4d6", "1d12", "2d6"]
    );
    f.worker
        .add_encounter_participant_condition(&key, &participant, strongest)
        .unwrap();
    for formula in ["6d12", "2d12", "4d6"] {
        let detail = f.worker.encounter(&key).unwrap();
        let source = detail.participants[0]
            .persistent_damage_groups
            .iter()
            .find(|g| g.damage_type == "fire")
            .unwrap()
            .sources
            .iter()
            .find(|s| s.formula == formula)
            .unwrap()
            .condition_id;
        f.worker
            .remove_encounter_participant_condition(
                &key,
                &participant,
                source,
                RemoveEncounterParticipantConditionRequest {
                    request_id: format!("expire-{source}"),
                    expected_encounter_revision: detail.revision,
                    intent: Default::default(),
                },
            )
            .unwrap();
    }
    // The higher-average 2d6 must not replace the GM's retained lower-pair preference.
    assert_order(&["1d12", "2d6"]);
    let next = request("new-stronger-source", "2d8");
    let preview = f
        .worker
        .preview_encounter_condition_add(&key, &participant, next.clone())
        .unwrap()
        .persistent_precedence
        .unwrap();
    assert_eq!(
        preview
            .sources
            .iter()
            .map(|s| s.formula.as_str())
            .collect::<Vec<_>>(),
        ["2d8", "1d12", "2d6"]
    );
    f.worker
        .add_encounter_participant_condition(&key, &participant, next)
        .unwrap();
    assert_order(&["2d8", "1d12", "2d6"]);
}

#[test]
fn stale_recovery_cannot_clear_an_episode() {
    let (f, key, participant) = fixture();
    let before = snapshot(&f, &key);
    assert!(
        f.worker
            .recover_persistent_damage(
                &key,
                RecoverPersistentDamageRequest {
                    resolution: PersistentRecoveryResolution::Roll,
                    boundary_key: None,
                    mode: PersistentRecoveryMode::Explicit,
                    request_id: "stale-recovery".into(),
                    expected_encounter_revision: before.encounter.revision - 1,
                    participant_key: participant,
                    episode_id: "episode-fire".into(),
                    dc_override: Some(1),
                }
            )
            .is_err()
    );
    assert_eq!(snapshot(&f, &key), before);
}

#[test]
fn prepared_recovery_keeps_damage_due_and_commits_all_changes_once() {
    let (f, key, participant) = fixture();
    let before = snapshot(&f, &key);
    let mut damage = batch(&f, &key, &participant);
    damage.artifact_generation = f.worker.encounter(&key).unwrap().artifact_generation;
    let prepare = PreparePersistentDamageRequest {
        request_id: "prepare-recovery".into(),
        expected_encounter_revision: before.encounter.revision,
        participant_key: participant.clone(),
        boundary_key: None,
        artifact_generation: damage.artifact_generation.clone(),
        groups: vec![damage.groups[0].clone()],
        recovery: Some(PreparedPersistentRecoveryInput {
            episode_id: "episode-fire".into(),
            dc_override: Some(0),
        }),
    };
    let result = f
        .worker
        .prepare_persistent_damage(&key, prepare.clone())
        .unwrap();
    assert!(!result.complete);
    assert_eq!(result.damage.total, 4);
    assert!(result.damage_draft.is_none());
    assert!(result.recovery.as_ref().unwrap().success);
    assert_eq!(
        snapshot(&f, &key),
        before,
        "Preparation must preserve HP, conditions and revisions"
    );
    let mut another_draft = prepare;
    another_draft.request_id = "another-recovery-roll".into();
    assert!(
        f.worker
            .prepare_persistent_damage(&key, another_draft)
            .is_ok()
    );
    let review = f
        .worker
        .persistent_damage_review(&key, &participant, None)
        .unwrap();
    assert_eq!(review.damage_groups.len(), 2);
    assert!(review.damage_result.is_none());
    assert!(review.recovery_results.is_empty());
    damage.recoveries = vec![
        PersistentRecoveryInput::Prepared {
            episode_id: "episode-fire".into(),
            draft: result.recovery_draft.unwrap(),
        },
        PersistentRecoveryInput::Clear {
            episode_id: "episode-acid".into(),
            draft: None,
        },
    ];
    let applied = f
        .worker
        .apply_persistent_damage(&key, damage.clone())
        .unwrap();
    assert_eq!(applied.total, 7);
    let after = snapshot(&f, &key);
    assert_eq!(after.participants[0].current_hp, Some(0));
    assert_eq!(
        after.participants[0]
            .conditions
            .iter()
            .find(|c| c.name == "Dying")
            .unwrap()
            .value,
        Some(1)
    );
    assert!(
        !after.participants[0]
            .conditions
            .iter()
            .any(|c| matches!(c.parameters, ConditionParameters::PersistentDamage { .. }))
    );
    let review = f
        .worker
        .persistent_damage_review(&key, &participant, None)
        .unwrap();
    assert_eq!(review.recovery_results.len(), 2);
    assert_eq!(
        f.worker.apply_persistent_damage(&key, damage).unwrap(),
        applied
    );
    assert_eq!(snapshot(&f, &key), after);
}

#[test]
fn prepared_rolls_keep_shared_hardness_and_reject_changed_subjects() {
    let (f, key, _) = fixture();
    f.worker
        .add_encounter_record_participant(
            &key,
            AddEncounterRecordParticipantRequest {
                participant_variant: None,
                record_ref: "actors:testCreature1000".into(),
                quantity: 1,
                initiative: Some(8),
            },
        )
        .unwrap();
    let actor = snapshot(&f, &key)
        .participants
        .into_iter()
        .find(|p| p.record_key.is_some())
        .unwrap();
    f.worker
        .with_write(|session| {
            let store = session.local_state()?.encounters();
            for (condition_key, parameters) in [
                ("conditionitems:dTwPJuKgBQCMxixg", ConditionParameters::None),
                (
                    "conditionitems:lDVqvLKA6eF3Df60",
                    ConditionParameters::PersistentDamage {
                        formula: "10".into(),
                        damage_type: "fire".into(),
                        multiplier: DamageMultiplier::Normal,
                        recovery: PersistentRecovery::Standard,
                        episode_id: "fire-statue".into(),
                    },
                ),
                (
                    "conditionitems:lDVqvLKA6eF3Df60",
                    ConditionParameters::PersistentDamage {
                        formula: "10".into(),
                        damage_type: "acid".into(),
                        multiplier: DamageMultiplier::Normal,
                        recovery: PersistentRecovery::Standard,
                        episode_id: "acid-statue".into(),
                    },
                ),
            ] {
                store.add_condition(AddEncounterParticipantCondition {
                    participant_key: actor.participant_key.clone(),
                    condition_key: Some(condition_key.into()),
                    name: "Fixture".into(),
                    value: None,
                    source_participant_key: None,
                    scope: Default::default(),
                    extra_action: Default::default(),
                    immunity_override: false,
                    parameters,
                    origin: None,
                    duration: None,
                    note: None,
                })?;
            }
            Ok(())
        })
        .unwrap();
    let before = snapshot(&f, &key);
    let generation = f.worker.encounter(&key).unwrap().artifact_generation;
    let mut groups = Vec::new();
    for (episode, subtotal, complete) in [("fire-statue", 2, false), ("acid-statue", 12, true)] {
        let mut inputs = groups.clone();
        inputs.push(PersistentDamageGroupInput {
            episode_id: episode.into(),
            damage: PersistentDamageInput::Roll,
        });
        let request = PreparePersistentDamageRequest {
            request_id: format!("prepare-{episode}"),
            expected_encounter_revision: before.encounter.revision,
            participant_key: actor.participant_key.clone(),
            boundary_key: None,
            artifact_generation: generation.clone(),
            groups: inputs,
            recovery: None,
        };
        let result = f
            .worker
            .prepare_persistent_damage(&key, request.clone())
            .unwrap();
        assert_eq!((result.damage.total, result.complete), (subtotal, complete));
        assert_eq!(
            f.worker.prepare_persistent_damage(&key, request).unwrap(),
            result
        );
        groups.push(PersistentDamageGroupInput {
            episode_id: episode.into(),
            damage: damage_draft(&result, episode),
        });
    }
    let carried = PreparePersistentDamageRequest {
        request_id: "preview-carried-rolls".into(),
        expected_encounter_revision: before.encounter.revision,
        participant_key: actor.participant_key.clone(),
        boundary_key: None,
        artifact_generation: generation.clone(),
        groups: groups.clone(),
        recovery: None,
    };
    let preview = f
        .worker
        .prepare_persistent_damage(&key, carried.clone())
        .unwrap();
    assert!(preview.complete);
    assert_eq!(preview.damage.total, 12);
    assert!(preview.damage_draft.is_none());
    let mut wrong_episode = carried;
    let (first, rest) = wrong_episode.groups.split_at_mut(1);
    std::mem::swap(&mut first[0].damage, &mut rest[0].damage);
    assert!(
        f.worker
            .prepare_persistent_damage(&key, wrong_episode)
            .is_err()
    );
    assert_eq!(snapshot(&f, &key), before);
    let review = f
        .worker
        .persistent_damage_review(&key, &actor.participant_key, None)
        .unwrap();
    assert!(review.damage_result.is_none());
    assert!(!review.damage_applied);
    let request = ApplyPersistentDamageRequest {
        request_id: "apply-prepared-statue".into(),
        expected_encounter_revision: before.encounter.revision,
        participant_key: actor.participant_key.clone(),
        boundary_key: None,
        artifact_generation: generation,
        groups,
        recoveries: vec![],
        context: Default::default(),
    };
    let mut wrong_episode = request.clone();
    let (first, rest) = wrong_episode.groups.split_at_mut(1);
    std::mem::swap(&mut first[0].damage, &mut rest[0].damage);
    assert!(
        f.worker
            .apply_persistent_damage(&key, wrong_episode)
            .is_err()
    );
    assert_eq!(snapshot(&f, &key), before);
    for change in [
        |draft: &mut PersistentDamageDraft| draft.context.participant_key = "another actor".into(),
        |draft: &mut PersistentDamageDraft| draft.context.round_number += 1,
        |draft: &mut PersistentDamageDraft| {
            draft.context.record_key = Some("actors:wrong-record".into());
        },
        |draft: &mut PersistentDamageDraft| draft.source_condition_id += 1,
        |draft: &mut PersistentDamageDraft| draft.roll.formula = "5+5".into(),
        |draft: &mut PersistentDamageDraft| draft.roll.total += 1,
    ] {
        let mut invalid = request.clone();
        let PersistentDamageInput::Prepared { draft } = &mut invalid.groups[0].damage else {
            panic!("expected a browser draft");
        };
        change(draft);
        assert!(f.worker.apply_persistent_damage(&key, invalid).is_err());
        assert_eq!(snapshot(&f, &key), before);
    }
    let result = f
        .worker
        .apply_persistent_damage(&key, request.clone())
        .unwrap();
    assert_eq!((result.total, result.hardness_absorbed), (12, 8));
    assert!(
        result
            .groups
            .iter()
            .all(|g| g.roll.is_some() && !g.table_resolved)
    );
    assert_eq!(
        f.worker
            .apply_persistent_damage(&key, request.clone())
            .unwrap(),
        result
    );
    let mut mismatch = request.clone();
    let PersistentDamageInput::Prepared { draft } = &mut mismatch.groups[0].damage else {
        panic!("expected a browser draft");
    };
    draft.roll.total += 1;
    assert!(f.worker.apply_persistent_damage(&key, mismatch).is_err());
    let mut stale = request;
    stale.request_id = "different-boundary-request".into();
    stale.expected_encounter_revision = snapshot(&f, &key).encounter.revision;
    assert!(f.worker.apply_persistent_damage(&key, stale).is_err());
}

#[test]
fn table_clear_retains_a_failed_prepared_check_without_repeating_damage() {
    let rolled_failure = atlas_dice::RollOutcome {
        formula: "1d20".into(),
        total: 4,
        expression: atlas_dice::RollExpression::Dice {
            dice: vec![atlas_dice::DieOutcome {
                sides: 20,
                value: 4,
                kept: true,
            }],
            total: 4,
        },
    };
    for (dc, roll) in [(100, None), (15, Some(rolled_failure))] {
        let (f, key, participant) = fixture();
        let before = snapshot(&f, &key);
        let mut damage = batch(&f, &key, &participant);
        let prepared = f
            .worker
            .prepare_persistent_damage(
                &key,
                PreparePersistentDamageRequest {
                    request_id: "failed-recovery".into(),
                    expected_encounter_revision: before.encounter.revision,
                    participant_key: participant.clone(),
                    boundary_key: None,
                    artifact_generation: None,
                    groups: vec![damage.groups[0].clone()],
                    recovery: Some(PreparedPersistentRecoveryInput {
                        episode_id: "episode-fire".into(),
                        dc_override: Some(100),
                    }),
                },
            )
            .unwrap()
            .recovery_draft
            .unwrap();
        let draft = PersistentRecoveryDraft {
            dc,
            dc_override: Some(dc),
            roll: roll.clone(),
            ..prepared
        };
        assert_eq!(snapshot(&f, &key), before);
        for recoveries in [
            vec![PersistentRecoveryInput::Prepared {
                episode_id: "episode-fire".into(),
                draft: PersistentRecoveryDraft {
                    dc: dc - 1,
                    ..draft.clone()
                },
            }],
            vec![
                PersistentRecoveryInput::Clear {
                    episode_id: "episode-fire".into(),
                    draft: None,
                },
                PersistentRecoveryInput::Prepared {
                    episode_id: "episode-fire".into(),
                    draft: draft.clone(),
                },
            ],
        ] {
            let mut invalid = damage.clone();
            invalid.recoveries = recoveries;
            assert!(f.worker.apply_persistent_damage(&key, invalid).is_err());
            assert_eq!(snapshot(&f, &key), before);
        }
        damage.recoveries = vec![PersistentRecoveryInput::Clear {
            episode_id: "episode-fire".into(),
            draft: Some(draft),
        }];
        let applied = f
            .worker
            .apply_persistent_damage(&key, damage.clone())
            .unwrap();
        assert_eq!(applied.total, 7);
        let after = snapshot(&f, &key);
        assert_eq!(after.participants[0].current_hp, Some(0));
        assert!(after.participants[0].conditions.iter().all(|condition| {
            !matches!(&condition.parameters, ConditionParameters::PersistentDamage { damage_type, .. } if damage_type == "fire")
        }));
        assert!(after.participants[0].conditions.iter().any(|condition| {
            matches!(&condition.parameters, ConditionParameters::PersistentDamage { damage_type, .. } if damage_type == "acid")
        }));
        let review = f
            .worker
            .persistent_damage_review(&key, &participant, None)
            .unwrap();
        let retained = review
            .recovery_results
            .iter()
            .find(|result| result.episode_id == "episode-fire")
            .unwrap();
        assert_eq!(
            (retained.dc, &retained.roll, retained.success),
            (Some(dc), &roll, false)
        );
        assert_eq!(
            f.worker.apply_persistent_damage(&key, damage).unwrap(),
            applied
        );
        assert_eq!(snapshot(&f, &key), after);
        assert_eq!(
            f.worker
                .persistent_damage_review(&key, &participant, None)
                .unwrap(),
            review
        );
    }
}

#[test]
fn independent_drafts_allow_only_one_damage_application_per_boundary() {
    let (f, key, participant) = fixture();
    let before = snapshot(&f, &key);
    let prepare = |id: &str, dc| {
        f.worker
            .prepare_persistent_damage(
                &key,
                PreparePersistentDamageRequest {
                    request_id: id.into(),
                    expected_encounter_revision: before.encounter.revision,
                    participant_key: participant.clone(),
                    boundary_key: None,
                    artifact_generation: None,
                    groups: vec![batch(&f, &key, &participant).groups.remove(0)],
                    recovery: Some(PreparedPersistentRecoveryInput {
                        episode_id: "episode-fire".into(),
                        dc_override: Some(dc),
                    }),
                },
            )
            .unwrap()
            .recovery_draft
            .unwrap()
    };
    let mut first = batch(&f, &key, &participant);
    first.recoveries = vec![PersistentRecoveryInput::Prepared {
        episode_id: "episode-fire".into(),
        draft: prepare("first-prep", 0),
    }];
    let mut second = batch(&f, &key, &participant);
    second.request_id = "second-apply".into();
    second.recoveries = vec![PersistentRecoveryInput::Prepared {
        episode_id: "episode-fire".into(),
        draft: prepare("second-prep", 100),
    }];
    assert_eq!(snapshot(&f, &key), before);
    let barrier = std::sync::Barrier::new(2);
    let (applied_first, applied_second) = std::thread::scope(|scope| {
        let apply = scope.spawn(|| {
            barrier.wait();
            f.worker.apply_persistent_damage(&key, first.clone())
        });
        let other = scope.spawn(|| {
            barrier.wait();
            f.worker.apply_persistent_damage(&key, second.clone())
        });
        (apply.join().unwrap(), other.join().unwrap())
    });
    assert_ne!(applied_first.is_ok(), applied_second.is_ok());
    let (winner, result) = match applied_first {
        Ok(result) => (first, result),
        Err(_) => (second, applied_second.unwrap()),
    };
    assert_eq!(
        f.worker.apply_persistent_damage(&key, winner).unwrap(),
        result
    );
    let review = f
        .worker
        .persistent_damage_review(&key, &participant, None)
        .unwrap();
    assert_eq!(review.damage_result, Some(result));
    assert_eq!(review.recovery_results.len(), 1);
}

#[test]
fn source_and_saved_petrified_die_at_zero_with_or_without_dying_tracking() {
    use serde_json::json;
    for source_owned in [false, true] {
        for track in [false, true] {
            let (f, _artifact, key, participant) = source_petrified_fixture(Some(source_owned));
            if !source_owned {
                f.worker.add_encounter_participant_condition(&key, &participant, serde_json::from_value(json!({
                    "request_id":"saved-statue", "expected_encounter_revision":f.worker.encounter(&key).unwrap().revision,"condition_ref":super::conditions::ConditionRule::Petrified.definition().record_key
                })).unwrap()).unwrap();
            }
            f.worker.set_encounter_life_settings(&key,&participant,serde_json::from_value(json!({
                "request_id":"track-statue","expected_encounter_revision":f.worker.encounter(&key).unwrap().revision,"track_dying":track
            })).unwrap()).unwrap();
            let before = f.worker.encounter(&key).unwrap();
            let request = serde_json::from_value(json!({"request_id":"destroy-statue","expected_revision":before.participants[0].revision,"action":{"kind":"damage","amount":"60"},"context":{"lethality":"lethal","critical":false,"keep_current_initiative":true}})).unwrap();
            f.worker
                .apply_encounter_hp_action(&key, &participant, request)
                .unwrap();
            let after = snapshot(&f, &key).participants.remove(0);
            assert_eq!(after.current_hp, Some(0));
            assert_eq!(after.mortality, atlas_local_state::MortalityState::Dead);
            assert!(
                !after
                    .conditions
                    .iter()
                    .any(
                        |c| super::conditions::condition_rule_for_key(c.condition_key.as_deref())
                            == Some(super::conditions::ConditionRule::Dying)
                    )
            );
        }
    }
}

#[test]
fn table_persistent_damage_uses_source_petrified_mortality_and_replays_before_reference_reads() {
    let (f, artifact, key, participant) = source_petrified_fixture(Some(true));
    let mut request = source_statue_damage(&f, &key, &participant);
    for group in &mut request.groups {
        group.damage = PersistentDamageInput::TableResult { amount: 20 };
    }
    let result = f
        .worker
        .apply_persistent_damage(&key, request.clone())
        .unwrap();
    assert_eq!(result.total, 60);
    assert_eq!(
        snapshot(&f, &key).participants[0].mortality,
        atlas_local_state::MortalityState::Dead
    );
    atlas_search::test_support::invalidate_reference(
        artifact.path(),
        atlas_runtime::WORKSPACE_SCHEMA_VERSION,
    )
    .unwrap();
    assert_eq!(
        f.worker.apply_persistent_damage(&key, request).unwrap(),
        result
    );
}

#[test]
fn uncertain_petrified_does_not_silently_use_ordinary_mortality() {
    use serde_json::json;
    let (f, _artifact, key, participant) = source_petrified_fixture(None);
    let before = f.worker.encounter(&key).unwrap();
    let request = serde_json::from_value(json!({"request_id":"unknown-statue","expected_revision":before.participants[0].revision,"action":{"kind":"damage","amount":"60"}})).unwrap();
    assert!(
        f.worker
            .apply_encounter_hp_action(&key, &participant, request)
            .is_err()
    );
    assert_eq!(snapshot(&f, &key).participants[0].current_hp, Some(60));
}

#[test]
fn source_condition_spend_matches_live_permissions_and_preserves_corrections() {
    let (f, _artifact, key, participant) = source_petrified_fixture(Some(false));
    f.worker
        .advance_encounter_turn(
            &key,
            AdvanceEncounterTurnRequest {
                request_id: "review-start".into(),
                expected_revision: f.worker.encounter(&key).unwrap().revision,
            },
        )
        .unwrap();
    let detail = f.worker.encounter(&key).unwrap();
    let actor = &detail.participants[0];
    let ability = actor.ability_states.as_ref().unwrap().first().unwrap();
    f.worker
        .update_encounter_ability_state(
            &key,
            &participant,
            UpdateEncounterAbilityStateRequest {
                request_id: "review-petrify".into(),
                expected_encounter_revision: detail.revision,
                expected_participant_revision: actor.revision,
                navigation: ability.navigation.clone(),
                rule_ordinal: ability.rule_ordinal,
                action: EncounterAbilityStateAction::SetOption {
                    active: true,
                    selection: None,
                },
            },
        )
        .unwrap();
    for resource in [TurnResourceKind::Action, TurnResourceKind::Reaction] {
        let detail = f.worker.encounter(&key).unwrap();
        let before = detail.participants[0].turn_resources.as_ref().unwrap();
        let permission = if resource == TurnResourceKind::Reaction {
            &before.can_spend_reactions
        } else {
            &before.can_spend_actions
        };
        assert!(
            !permission.available,
            "live source projection must prohibit spending"
        );
        let result = f.worker.update_turn_resources(
            &key,
            &participant,
            UpdateTurnResourcesRequest {
                request_id: format!("review-spend-{resource:?}"),
                expected_encounter_revision: detail.revision,
                turn_key: before.turn_key.clone(),
                action: TurnResourceAction::Spend { resource },
            },
        );
        assert!(result.unwrap_err().to_string().contains("Petrified"));
        let unchanged = f.worker.encounter(&key).unwrap();
        assert_eq!(unchanged.revision, detail.revision);
        let after = unchanged.participants[0].turn_resources.as_ref().unwrap();
        assert_eq!(after.regular_remaining, before.regular_remaining);
        assert_eq!(after.reactions_remaining, before.reactions_remaining);
        let correction = UpdateTurnResourcesRequest {
            request_id: format!("correct-{resource:?}"),
            expected_encounter_revision: detail.revision,
            turn_key: before.turn_key.clone(),
            action: TurnResourceAction::Correct {
                resource,
                remaining: 0,
            },
        };
        f.worker
            .update_turn_resources(&key, &participant, correction)
            .unwrap();
        let corrected = f.worker.encounter(&key).unwrap();
        f.worker
            .update_turn_resources(
                &key,
                &participant,
                UpdateTurnResourcesRequest {
                    request_id: format!("restore-{resource:?}"),
                    expected_encounter_revision: corrected.revision,
                    turn_key: before.turn_key.clone(),
                    action: TurnResourceAction::Restore { resource },
                },
            )
            .unwrap();
        let restored = f.worker.encounter(&key).unwrap();
        let resources = restored.participants[0].turn_resources.as_ref().unwrap();
        assert_eq!(
            if resource == TurnResourceKind::Reaction {
                resources.reactions_remaining
            } else {
                resources.regular_remaining
            },
            Some(1)
        );
    }
}

#[test]
fn source_spend_recovers_before_reference_loading_and_keeps_unavailable_source_manual_tracking() {
    let (f, artifact, key, participant) = source_petrified_fixture(Some(false));
    f.worker
        .advance_encounter_turn(
            &key,
            AdvanceEncounterTurnRequest {
                request_id: "source-start".into(),
                expected_revision: f.worker.encounter(&key).unwrap().revision,
            },
        )
        .unwrap();
    let detail = f.worker.encounter(&key).unwrap();
    let request = UpdateTurnResourcesRequest {
        request_id: "source-spend".into(),
        expected_encounter_revision: detail.revision,
        turn_key: detail.participants[0]
            .turn_resources
            .as_ref()
            .unwrap()
            .turn_key
            .clone(),
        action: TurnResourceAction::Spend {
            resource: TurnResourceKind::Action,
        },
    };
    let result = f
        .worker
        .update_turn_resources(&key, &participant, request.clone())
        .unwrap();
    let spent = snapshot(&f, &key);
    atlas_search::test_support::invalidate_reference(
        artifact.path(),
        atlas_runtime::WORKSPACE_SCHEMA_VERSION,
    )
    .unwrap();
    assert_eq!(
        f.worker
            .update_turn_resources(&key, &participant, request.clone())
            .unwrap(),
        result
    );
    assert_eq!(snapshot(&f, &key), spent);
    let manual = UpdateTurnResourcesRequest {
        request_id: "source-unavailable-spend".into(),
        expected_encounter_revision: spent.encounter.revision,
        ..request
    };
    f.worker
        .update_turn_resources(&key, &participant, manual)
        .unwrap();
    let after = snapshot(&f, &key);
    assert_eq!(
        after.participants[0]
            .turn_resources
            .as_ref()
            .unwrap()
            .regular_remaining,
        1
    );
}

#[test]
fn encounter_portability_resumes_outgoing_review_without_repeating_committed_damage() {
    let (source, key, participant) = fixture();
    let boundary = outgoing_boundary(&source, &key, &participant);
    let mut damage = batch(&source, &key, &participant);
    damage.boundary_key = Some(boundary.clone());
    let committed = source
        .worker
        .apply_persistent_damage(&key, damage.clone())
        .unwrap();
    assert_eq!(committed.total, 7);
    let recovery = RecoverPersistentDamageRequest {
        request_id: "outgoing-recover-fire".into(),
        resolution: PersistentRecoveryResolution::Roll,
        mode: PersistentRecoveryMode::EndTurn,
        expected_encounter_revision: snapshot(&source, &key).encounter.revision,
        participant_key: participant.clone(),
        episode_id: "episode-fire".into(),
        boundary_key: Some(boundary.clone()),
        dc_override: Some(0),
    };
    assert!(
        source
            .worker
            .recover_persistent_damage(&key, recovery)
            .unwrap()
            .success
    );
    let before = source
        .worker
        .encounter_turn_review(&key)
        .unwrap()
        .outgoing
        .unwrap();
    let bytes = source.worker.export_encounter(&key).unwrap();
    let fresh = encounter_fixture_worker();
    for (worker, slug) in [
        (&source.worker, Some("portable-copy".into())),
        (&fresh.worker, None),
    ] {
        let copy = worker.import_encounter(bytes.as_slice(), slug).unwrap();
        let restored = worker.encounter(&copy.encounter_key).unwrap();
        let copy_participant = &restored.participants[0].participant_key;
        let after = worker
            .encounter_turn_review(&copy.encounter_key)
            .unwrap()
            .outgoing
            .unwrap();
        assert_ne!(after.boundary_key, boundary);
        assert_eq!(after.recovered_episode_ids, vec!["episode-fire"]);
        assert_eq!(after.recovered_episode_ids, before.recovered_episode_ids);
        assert_ne!(copy_participant, &participant);
        assert_eq!(
            after
                .persistent_damage_groups
                .iter()
                .map(|g| &g.episode_id)
                .collect::<Vec<_>>(),
            before
                .persistent_damage_groups
                .iter()
                .map(|g| &g.episode_id)
                .collect::<Vec<_>>()
        );
        assert!(
            worker
                .persistent_damage_review(
                    &copy.encounter_key,
                    copy_participant,
                    Some(&after.boundary_key)
                )
                .unwrap()
                .damage_applied
        );

        // A new request for the same committed effect fails without changing HP.
        let mut repeated = damage.clone();
        repeated.request_id = "new-request-for-committed-damage".into();
        repeated.participant_key.clone_from(copy_participant);
        repeated.boundary_key = Some(after.boundary_key);
        repeated.expected_encounter_revision = restored.revision;
        assert!(
            worker
                .apply_persistent_damage(&copy.encounter_key, repeated)
                .is_err()
        );
        assert_eq!(
            worker.encounter(&copy.encounter_key).unwrap().revision,
            restored.revision
        );

        let advance = AdvanceEncounterTurnRequest {
            request_id: "advance-restored".into(),
            expected_revision: restored.revision,
        };
        worker
            .advance_encounter_turn(&copy.encounter_key, advance.clone())
            .unwrap();
        let once = worker.encounter(&copy.encounter_key).unwrap();
        worker
            .advance_encounter_turn(&copy.encounter_key, advance)
            .unwrap();
        assert_eq!(
            worker.encounter(&copy.encounter_key).unwrap().revision,
            once.revision
        );
        assert_eq!(
            once.encounter.round_number,
            restored.encounter.round_number + 1
        );
    }
    assert_eq!(source.worker.export_encounter(&key).unwrap(), bytes);
}
