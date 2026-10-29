use super::*;

#[test]
fn xp_table_preserves_supported_values_and_floors_lower_level_creatures() {
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
    assert_eq!(xp(-5, false, None), Ok(10));
    assert_eq!(xp(-20, false, None), Ok(10));
    assert!(xp(-5, true, None).is_err());
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
        .preview_encounter_difficulty(
            &encounter.slug,
            PreviewEncounterDifficultyRequest {
                expected_revision: request.expected_revision,
                reference: request.reference,
                addition: None,
            },
        )
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
            PreviewEncounterDifficultyRequest {
                addition: None,
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

#[test]
fn curated_rules_open_the_exact_journal_pages() {
    use atlas_search::test_support::{open_source_fixture, record};
    use serde_json::json;
    let f = crate::test_support::fixture_worker_with_executor(
        crate::executor::RetrievalExecutor::from_test_fixture_factory(1, 16, || {
            Ok(open_source_fixture(
                vec![record(
                    "journals",
                    "JournalEntry",
                    json!({"_id":"S55aqwWIzpQRFhcq","name":"GM Screen","pages":[{"_id":"WZgFEMDSP9TvYc4u","name":"Encounter Budget","type":"text","text":{"content":"<p>Budget rules.</p>","format":1}},{"_id":"ZcddTU02joqYuuoV","name":"XP Awards","type":"text","text":{"content":"<p>XP rules.</p>","format":1}}]}),
                )],
                false,
            )?)
        }),
    );
    let references = rule_references();
    assert_eq!(
        references
            .iter()
            .map(|rule| rule.title.as_str())
            .collect::<Vec<_>>(),
        ["Encounter Budget", "XP Awards"]
    );
    for rule in references {
        let selected = f
            .worker
            .record_detail_at(atlas_app_model::RecordDetailRequest {
                spell_rank: None,
                record_key: rule.navigation.record_key,
                owners: rule.navigation.owners.clone(),
                fields: vec![],
                passage: None,
                source_fingerprint: None,
            })
            .unwrap();
        assert!(matches!(
            selected.presentation.body,
            atlas_app_model::RecordBodyView::JournalPage(_)
        ));
        assert_eq!(selected.presentation.identity.title, rule.title);
        assert_eq!(selected.selected.owners, rule.navigation.owners);
    }
}

#[test]
fn add_preview_matches_committed_quantity_variants_and_hazards_without_writing() {
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
                        json!({"_id":"elite00000000000","name":"Elite creature","type":"npc","system":{"details":{"level":{"value":5}},"attributes":{"adjustment":"elite","hp":{"max":60}}}}),
                    ),
                    record(
                        "actors",
                        "Actor",
                        json!({"_id":"low0000000000000","name":"Low-level creature","type":"npc","system":{"details":{"level":{"value":0}},"attributes":{"hp":{"max":6}}}}),
                    ),
                    record(
                        "actors",
                        "Actor",
                        json!({"_id":"simple0000000000","name":"Simple trap","type":"hazard","system":{"details":{"level":{"value":5},"isComplex":false}}}),
                    ),
                    record(
                        "actors",
                        "Actor",
                        json!({"_id":"unknown000000000","name":"Unknown trap","type":"hazard","system":{"details":{"level":{"value":5}}}}),
                    ),
                ],
                false,
            )?)
        }),
    );
    let encounter = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Add preview".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let before = f.worker.encounter(&encounter.encounter_key).unwrap();
    let reference = EncounterDifficultyReferenceView {
        party_level: Some(5),
        party_size: Some(5),
    };
    for (key, variant, expected) in [
        ("actors:elite00000000000", None, Some(120)),
        (
            "actors:elite00000000000",
            Some(EncounterParticipantVariantView::Normal),
            Some(80),
        ),
        (
            "actors:elite00000000000",
            Some(EncounterParticipantVariantView::Weak),
            Some(60),
        ),
        ("actors:low0000000000000", None, Some(20)),
        ("actors:simple0000000000", None, Some(16)),
        ("actors:unknown000000000", None, None),
    ] {
        let preview = f
            .worker
            .preview_encounter_difficulty(
                &encounter.encounter_key,
                PreviewEncounterDifficultyRequest {
                    expected_revision: before.revision,
                    reference,
                    addition: Some(EncounterDifficultyAddition {
                        record_ref: key.into(),
                        quantity: 2,
                        participant_variant: variant,
                    }),
                },
            )
            .unwrap();
        assert_eq!(preview.total_xp, expected);
        assert_eq!(preview.opposition[0].participant_key, None);
        assert_eq!(
            f.worker.encounter(&encounter.encounter_key).unwrap(),
            before
        );
    }
    let mut request = PreviewEncounterDifficultyRequest {
        expected_revision: before.revision,
        reference,
        addition: Some(EncounterDifficultyAddition {
            record_ref: "actors:elite00000000000".into(),
            quantity: 2,
            participant_variant: None,
        }),
    };
    let preview = f
        .worker
        .preview_encounter_difficulty(&encounter.encounter_key, request.clone())
        .unwrap();
    assert_eq!(preview.xp_per_pc, Some(96));
    assert!(!preview.award_note.as_deref().unwrap().contains("Trivial"));
    request.addition.as_mut().unwrap().quantity = 0;
    assert!(
        f.worker
            .preview_encounter_difficulty(&encounter.encounter_key, request.clone())
            .is_err()
    );
    request.addition.as_mut().unwrap().quantity = 51;
    assert!(
        f.worker
            .preview_encounter_difficulty(&encounter.encounter_key, request.clone())
            .is_err()
    );
    request.addition.as_mut().unwrap().quantity = 2;
    f.worker
        .set_encounter_difficulty(
            &encounter.encounter_key,
            SetEncounterDifficultyRequest {
                expected_revision: before.revision,
                reference,
            },
        )
        .unwrap();
    f.worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: encounter.encounter_key.clone(),
            record_ref: "actors:elite00000000000".into(),
            quantity: 2,
            participant_variant: None,
            initiative: None,
        })
        .unwrap();
    let after = f.worker.encounter(&encounter.encounter_key).unwrap();
    assert_eq!(
        (
            after.difficulty.total_xp,
            after.difficulty.rating,
            after.difficulty.xp_per_pc
        ),
        (preview.total_xp, preview.rating, preview.xp_per_pc)
    );
    assert_eq!(after.participants.len(), 2);
    assert_eq!(
        f.worker
            .preview_encounter_difficulty(&encounter.encounter_key, request)
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::EncounterTurnConflict
    );
    let low_addition = EncounterDifficultyAddition {
        record_ref: "actors:low0000000000000".into(),
        quantity: 3,
        participant_variant: None,
    };
    let low_preview = f
        .worker
        .preview_encounter_difficulty(
            &encounter.encounter_key,
            PreviewEncounterDifficultyRequest {
                expected_revision: after.revision,
                reference,
                addition: Some(low_addition.clone()),
            },
        )
        .unwrap();
    assert_eq!(low_preview.total_xp, Some(150));
    assert_eq!(low_preview.opposition.last().unwrap().xp, Some(30));
    f.worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: encounter.encounter_key.clone(),
            record_ref: low_addition.record_ref,
            quantity: low_addition.quantity,
            participant_variant: None,
            initiative: None,
        })
        .unwrap();
    let committed = f.worker.encounter(&encounter.encounter_key).unwrap();
    assert_eq!(committed.difficulty.total_xp, low_preview.total_xp);
    assert_eq!(committed.difficulty.rating, low_preview.rating);
    assert_eq!(committed.difficulty.xp_per_pc, low_preview.xp_per_pc);
}

#[test]
fn awards_are_size_adjusted_suggestions_separate_from_difficulty_budgets() {
    for (size, encounter_xp, expected) in [
        (5, 100, 80),
        (3, 60, 80),
        (4, 80, 80),
        (6, 103, 68),
        (3, 10, 0),
    ] {
        let mut view = project(
            EncounterDifficultyReference {
                party_level: Some(5),
                party_size: Some(size),
            },
            &[],
            |_| None,
        );
        view.opposition.push(EncounterDifficultyOpponentView {
            participant_key: None,
            name: "Opposition".into(),
            kind: "Creature".into(),
            level: Some(5),
            xp: Some(encounter_xp),
            unavailable_reason: None,
        });
        recalculate(&mut view, &[]);
        assert_eq!(view.xp_per_pc, Some(expected));
        assert_eq!(view.total_xp, Some(encounter_xp));
    }
    let unknown = project(Default::default(), &[], |_| None);
    assert_eq!(unknown.xp_per_pc, None);
}
