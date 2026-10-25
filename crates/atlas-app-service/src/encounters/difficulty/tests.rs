use super::*;

#[test]
fn xp_table_distinguishes_creatures_and_hazards_without_clamping() {
    for (relative, expected) in [
        (-4, 10),
        (-3, 15),
        (-2, 20),
        (-1, 30),
        (0, 40),
        (1, 60),
        (2, 80),
        (3, 120),
        (4, 160),
    ] {
        assert_eq!(xp(relative, false, None), Ok(expected));
        assert_eq!(xp(relative, true, Some(true)), Ok(expected));
        assert_eq!(xp(relative, true, Some(false)), Ok(expected / 5));
    }
    assert!(xp(5, false, None).is_err());
    assert!(xp(-5, false, None).is_err());
    assert_eq!(xp(-5, true, Some(true)), Ok(0));
    assert!(xp(0, true, None).is_err());
}
#[test]
fn party_size_uses_each_published_adjustment_including_low() {
    let values = |size| budgets(size).into_iter().map(|b| b.xp).collect::<Vec<_>>();
    assert_eq!(values(4), [40, 60, 80, 120, 160]);
    assert_eq!(values(5), [50, 80, 100, 150, 200]);
    assert_eq!(values(3), [30, 40, 60, 90, 120]);
    assert_eq!(values(1), [10, 0, 20, 30, 40]);
    assert_eq!(rating(161, &budgets(4)), "Above Extreme");
    assert_eq!(rating(70, &budgets(4)), "Moderate");
}
#[test]
fn mixed_level_suggestions_preserve_the_choice_and_outlier_budget() {
    assert_eq!(suggest_level(Some(&[5, 5, 5, 5])).0, Some(5));
    assert_eq!(suggest_level(Some(&[4, 5, 5, 5])).0, Some(5));
    assert_eq!(suggest_level(Some(&[4, 5, 5, 6])).0, Some(5));
    assert_eq!(suggest_level(Some(&[3, 4, 5, 6])).0, Some(5));
    assert_eq!(suggest_level(Some(&[3, 3, 3, 7])).0, Some(3));
    assert_eq!(outlier_bonus(&[3, 3, 3, 7], 3), 2);
    assert_eq!(outlier_bonus(&[3, 3, 3, 7], 5), 0);
    assert_eq!(suggest_level(None).0, None);
    assert_eq!(suggest_level(Some(&[])).0, None);
}

#[test]
fn source_effective_levels_hazard_xp_and_saved_overrides_are_projected_together() {
    use atlas_app_model::*;
    use atlas_search::test_support::{open_source_fixture, record};
    use serde_json::json;
    let f = crate::test_support::fixture_worker_with_executor(
        crate::executor::RetrievalExecutor::from_test_fixture_factory(1, 16, || {
            Ok(open_source_fixture(
                vec![
                    record(
                        "actors",
                        "Actor",
                        json!({"_id":"aaaaaaaaaaaaaaaa","name":"Test creature","type":"npc","system":{"details":{"level":{"value":5}},"attributes":{"adjustment":null,"hp":{"max":60}}}}),
                    ),
                    record(
                        "actors",
                        "Actor",
                        json!({"_id":"bbbbbbbbbbbbbbbb","name":"Simple trap","type":"hazard","system":{"details":{"level":{"value":5},"isComplex":false}}}),
                    ),
                    record(
                        "actors",
                        "Actor",
                        json!({"_id":"cccccccccccccccc","name":"Complex trap","type":"hazard","system":{"details":{"level":{"value":5},"isComplex":true}}}),
                    ),
                    record(
                        "actors",
                        "Actor",
                        json!({"_id":"dddddddddddddddd","name":"Unknown trap","type":"hazard","system":{"details":{"level":{"value":5}}}}),
                    ),
                ],
                false,
            )?)
        }),
    );
    let encounter = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Difficulty integration".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    for (key, variant) in [
        (
            "actors:aaaaaaaaaaaaaaaa",
            Some(EncounterParticipantVariantView::Elite),
        ),
        ("actors:bbbbbbbbbbbbbbbb", None),
        ("actors:cccccccccccccccc", None),
    ] {
        f.worker
            .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
                encounter_ref: encounter.slug.clone(),
                record_ref: key.into(),
                quantity: 1,
                initiative: None,
                participant_variant: variant,
            })
            .unwrap();
    }
    let detail = f.worker.encounter(&encounter.slug).unwrap();
    assert_eq!(detail.difficulty.total_xp, None);
    let request = SetEncounterDifficultyRequest {
        expected_revision: detail.revision,
        reference: EncounterDifficultyReferenceView {
            party_level: Some(5),
            party_size: Some(4),
        },
    };
    let preview = f
        .worker
        .preview_encounter_difficulty(&encounter.slug, request.clone())
        .unwrap();
    assert_eq!(
        preview.opposition.iter().map(|o| o.xp).collect::<Vec<_>>(),
        [Some(60), Some(8), Some(40)]
    );
    assert_eq!(preview.total_xp, Some(108));
    assert_eq!(preview.rating.as_deref(), Some("Severe"));
    assert_eq!(
        f.worker.encounter(&encounter.slug).unwrap().revision,
        detail.revision
    );
    f.worker
        .set_encounter_difficulty(&encounter.slug, request.clone())
        .unwrap();
    assert_eq!(
        f.worker
            .set_encounter_difficulty(&encounter.slug, request)
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::EncounterTurnConflict
    );
    let saved = f.worker.encounter(&encounter.slug).unwrap();
    for p in &saved.participants {
        let mut update = super::super::tests::update(p);
        update.defeated = p.participant_kind == EncounterParticipantKindView::Creature;
        update.hazard_disabled = p.participant_kind == EncounterParticipantKindView::Hazard;
        f.worker
            .update_encounter_participant(&encounter.slug, update)
            .unwrap();
    }
    let after = f.worker.encounter(&encounter.slug).unwrap();
    assert_eq!(after.difficulty, saved.difficulty);
    f.worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: encounter.slug.clone(),
            record_ref: "actors:dddddddddddddddd".into(),
            quantity: 1,
            initiative: None,
            participant_variant: None,
        })
        .unwrap();
    let unavailable = f.worker.encounter(&encounter.slug).unwrap().difficulty;
    assert_eq!(unavailable.total_xp, None);
    assert_eq!(unavailable.known_xp, 108);
    assert!(
        unavailable
            .opposition
            .last()
            .unwrap()
            .unavailable_reason
            .as_deref()
            .unwrap()
            .contains("complexity")
    );
}

#[test]
fn party_snapshots_supply_mixed_references_and_unknown_levels_stay_unknown() {
    use atlas_app_model::*;
    let f = crate::test_support::encounter_fixture_worker();
    let key = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Mixed party difficulty".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter
        .encounter_key;
    let party = f
        .worker
        .create_party(CreatePartyRequest {
            name: "Mixed party".into(),
            members: [3, 3, 3, 7]
                .into_iter()
                .enumerate()
                .map(|(i, level)| PartyMemberInput {
                    member_key: None,
                    name: format!("PC {i}"),
                    level: Some(level),
                })
                .collect(),
        })
        .unwrap();
    f.worker
        .add_encounter_party(
            &key,
            AddEncounterPartyRequest {
                party_key: party.party_key,
                expected_revision: party.revision,
            },
        )
        .unwrap();
    let before = f.worker.encounter(&key).unwrap();
    assert_eq!(
        (
            before.difficulty.pc_count,
            before.difficulty.party_level,
            before.difficulty.party_size
        ),
        (4, Some(3), Some(6))
    );
    f.worker
        .add_encounter_manual_participant(AddEncounterManualParticipantRequest {
            encounter_ref: key.clone(),
            display_name: "Unknown level".into(),
            max_hp: None,
            current_hp: None,
            initiative: None,
        })
        .unwrap();
    let unknown = f.worker.encounter(&key).unwrap();
    assert_eq!(
        (
            unknown.difficulty.party_level,
            unknown.difficulty.party_size
        ),
        (None, Some(5))
    );
    let preview = f
        .worker
        .preview_encounter_difficulty(
            &key,
            SetEncounterDifficultyRequest {
                expected_revision: unknown.revision,
                reference: EncounterDifficultyReferenceView {
                    party_level: Some(5),
                    party_size: Some(4),
                },
            },
        )
        .unwrap();
    assert_eq!(
        (preview.party_level, preview.party_size),
        (Some(5), Some(4))
    );
    assert_eq!(
        f.worker.encounter(&key).unwrap().difficulty,
        unknown.difficulty
    );
}
