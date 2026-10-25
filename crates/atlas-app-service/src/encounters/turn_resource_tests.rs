use crate::test_support::{FixtureWorker, encounter_fixture_worker};
use atlas_app_model::*;
fn fixture() -> (FixtureWorker, String, String, String) {
    let f = encounter_fixture_worker();
    let key = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Resources".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter
        .encounter_key;
    for (name, initiative) in [("A", 20), ("B", 10)] {
        f.worker
            .add_encounter_manual_participant(AddEncounterManualParticipantRequest {
                encounter_ref: key.clone(),
                display_name: name.into(),
                max_hp: Some(20),
                current_hp: Some(20),
                initiative: Some(initiative),
            })
            .unwrap();
    }
    let detail = f.worker.encounter(&key).unwrap();
    let a = detail.participants[0].participant_key.clone();
    let b = detail.participants[1].participant_key.clone();
    (f, key, a, b)
}
fn add(f: &FixtureWorker, key: &str, name: &str, value: Option<i64>) -> i64 {
    let definition = super::conditions::condition_catalog()
        .conditions
        .into_iter()
        .find(|c| c.name == name)
        .unwrap();
    f.worker
        .with_write(|session| {
            Ok(session.local_state().encounters().add_condition(
                atlas_local_state::AddEncounterParticipantCondition {
                    participant_key: key.into(),
                    condition_key: Some(definition.condition_ref),
                    name: name.into(),
                    value,
                    scope: Default::default(),
                    extra_action: Default::default(),
                    immunity_override: false,
                    parameters: Default::default(),
                    origin: None,
                    source_participant_key: None,
                    duration: None,
                    note: None,
                },
            )?)
        })
        .unwrap()
        .condition_id
}
fn next(f: &FixtureWorker, key: &str, id: &str) -> AdvanceEncounterTurnRequest {
    let request = AdvanceEncounterTurnRequest {
        request_id: id.into(),
        expected_revision: f.worker.encounter(key).unwrap().revision,
    };
    f.worker
        .advance_encounter_turn(key, request.clone())
        .unwrap();
    request
}
fn change(
    f: &FixtureWorker,
    key: &str,
    p: &str,
    id: &str,
    action: TurnResourceAction,
) -> UpdateTurnResourcesRequest {
    let detail = f.worker.encounter(key).unwrap();
    let participant = detail
        .participants
        .iter()
        .find(|v| v.participant_key == p)
        .unwrap();
    let request = UpdateTurnResourcesRequest {
        request_id: id.into(),
        expected_encounter_revision: detail.revision,
        participant_key: p.into(),
        turn_key: participant
            .turn_resources
            .as_ref()
            .and_then(|v| v.turn_key.clone()),
        action,
    };
    f.worker
        .update_turn_resources(key, request.clone())
        .unwrap();
    request
}
fn resources(f: &FixtureWorker, key: &str, p: &str) -> TurnResourcesView {
    f.worker
        .encounter(key)
        .unwrap()
        .participants
        .into_iter()
        .find(|v| v.participant_key == p)
        .unwrap()
        .turn_resources
        .unwrap()
}
#[test]
fn selection_does_not_invent_a_past_allocation_or_regain_resources() {
    let (f, key, a, _) = fixture();
    assert_eq!(resources(&f, &key, &a).regular_remaining, None);
    f.worker
        .set_encounter_turn(SetEncounterTurnRequest {
            encounter_ref: key.clone(),
            participant_key: a.clone(),
        })
        .unwrap();
    assert_eq!(resources(&f, &key, &a).regular_remaining, None);
}
#[test]
fn stunned_consumption_and_allocation_are_atomic_and_retries_never_refill_spent_actions() {
    let (f, key, a, _) = fixture();
    add(&f, &a, "Stunned", Some(2));
    let advance = next(&f, &key, "start-a");
    let initial = resources(&f, &key, &a);
    assert_eq!(
        (initial.regular_total, initial.regular_remaining),
        (Some(1), Some(1))
    );
    assert_eq!(initial.reactions_remaining, Some(1));
    assert!(
        f.worker.encounter(&key).unwrap().participants[0]
            .conditions
            .is_empty()
    );
    let spend = change(
        &f,
        &key,
        &a,
        "spend-a",
        TurnResourceAction::Spend {
            resource: TurnResourceKind::Action,
        },
    );
    f.worker.update_turn_resources(&key, spend).unwrap();
    f.worker.advance_encounter_turn(&key, advance).unwrap();
    assert_eq!(
        (
            resources(&f, &key, &a).regular_total,
            resources(&f, &key, &a).regular_remaining
        ),
        (Some(1), Some(0))
    );
    assert!(!resources(&f, &key, &a).start_results.is_empty());
    assert!(
        f.worker
            .update_turn_resources(
                &key,
                UpdateTurnResourcesRequest {
                    request_id: "refill-disallowed".into(),
                    expected_encounter_revision: f.worker.encounter(&key).unwrap().revision,
                    participant_key: a.clone(),
                    turn_key: resources(&f, &key, &a).turn_key,
                    action: TurnResourceAction::Regain
                }
            )
            .is_err()
    );
}
#[test]
fn reactions_can_be_spent_off_turn_and_only_their_owners_regain_refills_them() {
    let (f, key, a, b) = fixture();
    next(&f, &key, "a1");
    next(&f, &key, "b1");
    change(
        &f,
        &key,
        &a,
        "react-a",
        TurnResourceAction::Spend {
            resource: TurnResourceKind::Reaction,
        },
    );
    assert_eq!(resources(&f, &key, &a).reactions_remaining, Some(0));
    assert_eq!(resources(&f, &key, &b).reactions_remaining, Some(1));
    next(&f, &key, "a2");
    assert_eq!(resources(&f, &key, &a).reactions_remaining, Some(1));
}
#[test]
fn mid_turn_quickened_changes_the_forecast_without_granting_an_immediate_action() {
    let (f, key, a, _) = fixture();
    next(&f, &key, "a1");
    add(&f, &a, "Quickened", None);
    let current = resources(&f, &key, &a);
    assert_eq!(current.regular_total, Some(3));
    assert_eq!(current.extra_total, Some(0));
    assert_eq!(current.forecast.adjusted_value, Some(4));
    next(&f, &key, "b1");
    next(&f, &key, "a2");
    assert_eq!(resources(&f, &key, &a).extra_remaining, Some(1));
}

#[test]
fn mid_turn_stun_consumes_only_recorded_remaining_actions_and_does_not_refill_after_clearing() {
    let (f, key, a, _) = fixture();
    next(&f, &key, "a1");
    change(
        &f,
        &key,
        &a,
        "spent",
        TurnResourceAction::Spend {
            resource: TurnResourceKind::Action,
        },
    );
    let condition_id = add(&f, &a, "Stunned", Some(1));
    f.worker
        .apply_condition_lifecycle(
            &key,
            ApplyConditionLifecycleRequest {
                request_id: "resolve-stun".into(),
                expected_encounter_revision: f.worker.encounter(&key).unwrap().revision,
                participant_key: a.clone(),
                boundary_key: None,
                action: ConditionLifecycleActionView::StunnedActionsLost {
                    condition_id,
                    actions_lost: 1,
                },
            },
        )
        .unwrap();
    assert_eq!(
        (
            resources(&f, &key, &a).regular_total,
            resources(&f, &key, &a).regular_remaining
        ),
        (Some(3), Some(1))
    );
    assert!(resources(&f, &key, &a).can_spend_actions.available);
    assert!(
        f.worker.encounter(&key).unwrap().participants[0]
            .conditions
            .is_empty()
    );
}
#[test]
fn ending_a_turn_discards_actions_without_spending_the_reaction() {
    let (f, key, a, _) = fixture();
    next(&f, &key, "a1");
    next(&f, &key, "b1");
    assert_eq!(resources(&f, &key, &a).regular_remaining, Some(0));
    assert_eq!(resources(&f, &key, &a).reactions_remaining, Some(1));
    assert!(!resources(&f, &key, &a).can_spend_actions.available);
}

#[test]
fn a_condition_blocks_permission_without_erasing_reaction_stock() {
    let (f, key, a, _) = fixture();
    add(&f, &a, "Stunned", Some(4));
    next(&f, &key, "start-stunned");
    let blocked = resources(&f, &key, &a);
    assert_eq!(blocked.regular_remaining, Some(0));
    assert_eq!(blocked.reactions_remaining, Some(1));
    assert!(!blocked.can_spend_reactions.available);
    let condition = f.worker.encounter(&key).unwrap().participants[0].conditions[0].condition_id;
    f.worker
        .with_write(|session| {
            session
                .local_state()
                .encounters()
                .remove_condition(&a, condition)?;
            Ok(())
        })
        .unwrap();
    assert!(resources(&f, &key, &a).can_spend_reactions.available);
    change(
        &f,
        &key,
        &a,
        "react",
        TurnResourceAction::Spend {
            resource: TurnResourceKind::Reaction,
        },
    );
    let stunned = add(&f, &a, "Stunned", Some(1));
    assert!(!resources(&f, &key, &a).can_spend_reactions.available);
    assert_eq!(resources(&f, &key, &a).reactions_remaining, Some(0));
    f.worker
        .with_write(|session| {
            session
                .local_state()
                .encounters()
                .remove_condition(&a, stunned)?;
            Ok(())
        })
        .unwrap();
    assert!(resources(&f, &key, &a).can_spend_reactions.available);
    assert_eq!(resources(&f, &key, &a).reactions_remaining, Some(0));
}
#[test]
fn stale_turn_identity_and_off_turn_action_corrections_do_not_change_resources() {
    let (f, key, a, _) = fixture();
    next(&f, &key, "a1");
    let first = resources(&f, &key, &a);
    next(&f, &key, "b1");
    let off_turn = UpdateTurnResourcesRequest {
        request_id: "wrong-position".into(),
        expected_encounter_revision: f.worker.encounter(&key).unwrap().revision,
        participant_key: a.clone(),
        turn_key: first.turn_key.clone(),
        action: TurnResourceAction::Correct {
            resource: TurnResourceKind::Action,
            remaining: 2,
        },
    };
    assert!(f.worker.update_turn_resources(&key, off_turn).is_err());
    next(&f, &key, "a2");
    let stale = UpdateTurnResourcesRequest {
        request_id: "old-turn".into(),
        expected_encounter_revision: f.worker.encounter(&key).unwrap().revision,
        participant_key: a.clone(),
        turn_key: first.turn_key,
        action: TurnResourceAction::Spend {
            resource: TurnResourceKind::Reaction,
        },
    };
    assert!(f.worker.update_turn_resources(&key, stale).is_err());
    assert_eq!(resources(&f, &key, &a).regular_remaining, Some(3));
    assert_eq!(resources(&f, &key, &a).reactions_remaining, Some(1));
}
#[test]
fn new_mid_turn_stun_can_resolve_after_start_turn_stun_without_replaying_either_loss() {
    let (f, key, a, _) = fixture();
    add(&f, &a, "Stunned", Some(1));
    next(&f, &key, "a1");
    let condition_id = add(&f, &a, "Stunned", Some(1));
    let request = ApplyConditionLifecycleRequest {
        request_id: "mid-turn-after-regain".into(),
        expected_encounter_revision: f.worker.encounter(&key).unwrap().revision,
        participant_key: a.clone(),
        boundary_key: None,
        action: ConditionLifecycleActionView::StunnedActionsLost {
            condition_id,
            actions_lost: 1,
        },
    };
    f.worker
        .apply_condition_lifecycle(&key, request.clone())
        .unwrap();
    f.worker.apply_condition_lifecycle(&key, request).unwrap();
    assert_eq!(resources(&f, &key, &a).regular_remaining, Some(1));
    assert_eq!(resources(&f, &key, &a).start_results.len(), 2);
    assert_eq!(
        resources(&f, &key, &a).start_results[0],
        TurnResourceResultView {
            condition_name: "Stunned".into(),
            condition_ref: super::conditions::ConditionRule::Stunned
                .definition()
                .record_key
                .into(),
            outcome: "1 cleared · 1 action lost this turn".into(),
        }
    );
}

fn application(
    f: &FixtureWorker,
    key: &str,
    p: &str,
    value: i64,
    intent: ConditionEventIntentView,
    id: &str,
) -> AddEncounterParticipantConditionRequest {
    AddEncounterParticipantConditionRequest {
        request_id: id.into(),
        expected_encounter_revision: f.worker.encounter(key).unwrap().revision,
        event_intent: intent,
        decisions: Default::default(),
        participant_key: p.into(),
        condition_ref: Some(
            super::conditions::condition_catalog()
                .conditions
                .into_iter()
                .find(|c| c.name == "Stunned")
                .unwrap()
                .condition_ref,
        ),
        name: None,
        value: Some(value),
        source_participant_key: None,
        scope: Default::default(),
        extra_action: Default::default(),
        immunity_override: false,
        parameters: Default::default(),
        origin: None,
        duration: None,
        note: Some("Source note".into()),
    }
}
#[test]
fn applying_stunned_mid_turn_previews_and_commits_the_same_remaining_loss_once() {
    let (f, key, a, _) = fixture();
    next(&f, &key, "a1");
    let request = application(
        &f,
        &key,
        &a,
        2,
        ConditionEventIntentView::Apply,
        "stun-arrival",
    );
    let before = f.worker.encounter(&key).unwrap();
    let preview = f
        .worker
        .preview_encounter_condition_add(&key, request.clone())
        .unwrap();
    assert!(
        preview
            .consequences
            .contains(&ConditionEventConsequenceView::StunnedActionsLost {
                participant_key: a.clone(),
                before: 3,
                after: 1,
                stunned_after: 0
            })
    );
    assert_eq!(f.worker.encounter(&key).unwrap(), before);
    f.worker
        .add_encounter_participant_condition(&key, request.clone())
        .unwrap();
    f.worker
        .add_encounter_participant_condition(&key, request)
        .unwrap();
    let d = f.worker.encounter(&key).unwrap();
    assert!(d.participants[0].conditions.is_empty());
    let r = resources(&f, &key, &a);
    assert_eq!(r.regular_remaining, Some(1));
    assert_eq!(r.reactions_remaining, Some(1));
    assert_eq!(
        r.start_results,
        [TurnResourceResultView {
            condition_name: "Stunned".into(),
            condition_ref: super::conditions::ConditionRule::Stunned
                .definition()
                .record_key
                .into(),
            outcome: "2 cleared · 2 actions lost this turn".into(),
        }]
    );
}
#[test]
fn arrival_uses_only_remaining_actions_and_keeps_the_unconsumed_source_faithfully() {
    let (f, key, a, _) = fixture();
    next(&f, &key, "a1");
    change(
        &f,
        &key,
        &a,
        "spend1",
        TurnResourceAction::Spend {
            resource: TurnResourceKind::Action,
        },
    );
    change(
        &f,
        &key,
        &a,
        "spend2",
        TurnResourceAction::Spend {
            resource: TurnResourceKind::Action,
        },
    );
    let request = application(
        &f,
        &key,
        &a,
        2,
        ConditionEventIntentView::Apply,
        "part-stun",
    );
    f.worker
        .add_encounter_participant_condition(&key, request)
        .unwrap();
    let d = f.worker.encounter(&key).unwrap();
    let c = &d.participants[0].conditions[0];
    assert!(c.condition_id > 0);
    assert_eq!(c.value, Some(1));
    assert_eq!(c.note.as_deref(), Some("Source note"));
    assert_eq!(c.origin.as_ref().unwrap().application_id, "part-stun");
    assert_eq!(resources(&f, &key, &a).regular_remaining, Some(0));
    assert!(!resources(&f, &key, &a).can_spend_reactions.available);
    next(&f, &key, "b1");
    next(&f, &key, "a2");
    assert_eq!(resources(&f, &key, &a).regular_remaining, Some(2));
}
#[test]
fn off_turn_arrival_and_correction_do_not_consume_current_resources() {
    let (f, key, a, b) = fixture();
    next(&f, &key, "a1");
    let request = application(
        &f,
        &key,
        &a,
        2,
        ConditionEventIntentView::Correction,
        "stun-correction",
    );
    f.worker
        .add_encounter_participant_condition(&key, request)
        .unwrap();
    assert_eq!(resources(&f, &key, &a).regular_remaining, Some(3));
    next(&f, &key, "b1");
    let request = application(
        &f,
        &key,
        &a,
        3,
        ConditionEventIntentView::Apply,
        "off-turn-stun",
    );
    f.worker
        .add_encounter_participant_condition(&key, request)
        .unwrap();
    assert_eq!(resources(&f, &key, &a).regular_remaining, Some(0));
    assert_eq!(resources(&f, &key, &b).regular_remaining, Some(3));
    assert_eq!(
        f.worker.encounter(&key).unwrap().participants[0]
            .conditions
            .len(),
        2
    );
}

#[test]
fn unknown_next_turn_slowed_does_not_block_known_current_stunned_loss() {
    let (f, key, a, _) = fixture();
    next(&f, &key, "a1");
    add(&f, &a, "Slowed", None);
    let request = application(
        &f,
        &key,
        &a,
        2,
        ConditionEventIntentView::Apply,
        "stun-with-unknown-slowed",
    );
    f.worker
        .add_encounter_participant_condition(&key, request)
        .unwrap();
    let r = resources(&f, &key, &a);
    assert_eq!(r.regular_remaining, Some(1));
    assert_eq!(r.forecast.adjusted_value, None);
    let d = f.worker.encounter(&key).unwrap();
    assert_eq!(d.participants[0].conditions.len(), 1);
    assert_eq!(d.participants[0].conditions[0].name, "Slowed");
    assert_eq!(d.participants[0].conditions[0].value, None);
}

#[test]
fn applying_an_updated_stunned_source_preserves_its_identity_and_edited_metadata() {
    let (f, key, a, _) = fixture();
    next(&f, &key, "a1");
    change(
        &f,
        &key,
        &a,
        "spent-1",
        TurnResourceAction::Spend {
            resource: TurnResourceKind::Action,
        },
    );
    change(
        &f,
        &key,
        &a,
        "spent-2",
        TurnResourceAction::Spend {
            resource: TurnResourceKind::Action,
        },
    );
    let request = application(
        &f,
        &key,
        &a,
        1,
        ConditionEventIntentView::Correction,
        "original-stun",
    );
    f.worker
        .add_encounter_participant_condition(&key, request)
        .unwrap();
    let condition_id = f.worker.encounter(&key).unwrap().participants[0].conditions[0].condition_id;
    let request = UpdateEncounterParticipantConditionRequest {
        request_id: "updated-stun".into(),
        expected_encounter_revision: f.worker.encounter(&key).unwrap().revision,
        event_intent: ConditionEventIntentView::Apply,
        decisions: Default::default(),
        condition_id,
        condition_ref: None,
        name: "Stunned".into(),
        value: Some(3),
        source_participant_key: None,
        scope: Default::default(),
        extra_action: Default::default(),
        immunity_override: false,
        parameters: Default::default(),
        origin: None,
        duration: None,
        note: Some("Edited source note".into()),
    };
    let preview = f
        .worker
        .preview_encounter_condition_update(&key, &a, request.clone())
        .unwrap();
    assert!(
        preview
            .consequences
            .contains(&ConditionEventConsequenceView::StunnedActionsLost {
                participant_key: a.clone(),
                before: 1,
                after: 0,
                stunned_after: 2
            })
    );
    f.worker
        .update_encounter_participant_condition(&key, &a, request.clone())
        .unwrap();
    f.worker
        .update_encounter_participant_condition(&key, &a, request)
        .unwrap();
    let d = f.worker.encounter(&key).unwrap();
    let c = &d.participants[0].conditions[0];
    assert_eq!(c.condition_id, condition_id);
    assert_eq!(c.value, Some(2));
    assert_eq!(c.note.as_deref(), Some("Edited source note"));
    assert_eq!(c.origin.as_ref().unwrap().application_id, "original-stun");
    assert_eq!(resources(&f, &key, &a).regular_remaining, Some(0));
}
