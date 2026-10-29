use super::super::conditions::ConditionRule;
use super::hp_events::{HpEventInput, plan_hp_event};
use super::numerical_tests::{condition, participant};
use super::source_conditions::{SourceConditions, source_conditions};
use atlas_local_state::{
    ConditionEventChange, DamageLethality, MortalityState, SavedHpEventContext,
};
use serde_json::json;

fn inherited(rule: ConditionRule, value: i64) -> SourceConditions {
    inherited_with_predicate(rule, value, json!([]))
}

fn inherited_with_predicate(
    rule: ConditionRule,
    value: i64,
    predicate: serde_json::Value,
) -> SourceConditions {
    let record = atlas_search::test_support::record(
        "actors",
        "Actor",
        json!({
            "_id":"aaaaaaaaaaaaaaaa","type":"npc","name":"Inherited condition fixture",
            "system":{"traits":{"value":[]},"attributes":{"immunities":[]}},
            "items":[{"_id":"bbbbbbbbbbbbbbbb","type":"feat","name":"Maintained condition",
                "system":{"actionType":{"value":"passive"},"rules":[{
                    "key":"GrantItem","inMemoryOnly":true,"uuid":"fixture:condition","predicate":predicate,
                    "alterations":[{"mode":"override","property":"badge-value","value":value}]
                }]}}]
        }),
    )
    .record;
    source_conditions(&record, |_| Ok(Some(rule))).unwrap()
}

fn lethal() -> SavedHpEventContext {
    SavedHpEventContext {
        lethality: Some(DamageLethality::Lethal),
        critical: Some(false),
        keep_current_initiative: Some(true),
        ..Default::default()
    }
}

#[test]
fn inherited_doomed_contributes_to_damage_mortality_with_overlapping_saved_sources() {
    let mut p = participant();
    p.track_dying = true;
    p.current_hp = Some(0);
    p.conditions = vec![
        condition(1, ConditionRule::Dying, Some(1)),
        condition(2, ConditionRule::Doomed, Some(1)),
    ];
    let sources = inherited(ConditionRule::Doomed, 2);
    assert_eq!(
        super::causes::condition_causes(&p, None, &sources)
            .known_value(ConditionRule::Doomed)
            .unwrap(),
        2
    );
    let changes = plan_hp_event(
        &p,
        &sources,
        &HpEventInput::ResolvedDamage { amount: 1 },
        &lethal(),
        Some(60),
        "damage",
    )
    .unwrap();
    assert_eq!(
        changes.participant_changes[0].mortality,
        MortalityState::Dead
    );
}

#[test]
fn inherited_wounded_sets_the_knockout_counter_instead_of_the_lower_saved_value() {
    let mut p = participant();
    p.track_dying = true;
    p.current_hp = Some(1);
    p.conditions = vec![condition(1, ConditionRule::Wounded, Some(1))];
    let sources = inherited(ConditionRule::Wounded, 2);
    let changes = plan_hp_event(
        &p,
        &sources,
        &HpEventInput::ResolvedDamage { amount: 1 },
        &lethal(),
        Some(60),
        "knockout",
    )
    .unwrap();
    assert!(changes.condition_changes.iter().any(|change| matches!(change, ConditionEventChange::Add(row) if row.name == "Dying" && row.value == Some(3))));
}

#[test]
fn inherited_dying_defers_regain_even_without_an_editable_dying_row() {
    let mut p = participant();
    p.track_dying = true;
    p.current_hp = Some(0);
    let sources = inherited(ConditionRule::Dying, 1);
    let (changes, _) =
        crate::encounters::turn_resources::prepare_start(&p, 1, "start", false, &sources).unwrap();
    let resources = &changes.unwrap().turn_resources[0];
    assert_eq!(
        resources.pending_reason.as_deref(),
        Some("Resolve Dying recovery before regaining actions.")
    );
    assert!(!resources.actions_open);
}

#[test]
fn healing_does_not_award_wounded_when_an_inherited_dying_cause_still_applies() {
    let mut p = participant();
    p.track_dying = true;
    p.current_hp = Some(0);
    p.conditions = vec![condition(1, ConditionRule::Dying, Some(1))];
    let sources = inherited(ConditionRule::Dying, 2);
    let changes = plan_hp_event(
        &p,
        &sources,
        &HpEventInput::ResolvedHealing { amount: 5 },
        &Default::default(),
        Some(60),
        "heal",
    )
    .unwrap();
    assert_eq!(changes.participant_changes[0].current_hp, Some(5));
    assert!(
        !changes.condition_changes.iter().any(
            |change| matches!(change, ConditionEventChange::Add(row) if row.name == "Wounded")
        )
    );
}

fn possible(rule: ConditionRule, value: i64) -> SourceConditions {
    inherited_with_predicate(rule, value, json!(["self:environment:unresolved"]))
}

fn open_turn(p: &mut atlas_local_state::EncounterParticipant) {
    let (changes, _) = crate::encounters::turn_resources::prepare_start(
        p,
        1,
        "open",
        false,
        SourceConditions::empty(),
    )
    .unwrap();
    p.turn_resources = Some(changes.unwrap().turn_resources.remove(0));
}

#[test]
fn a_possible_doomed_value_blocks_only_recovery_outcomes_it_can_change() {
    use atlas_app_model::{
        CheckDegreeView as Degree, ConditionLifecycleActionView as Action,
        LifecycleCheckInputView as Input,
    };
    let mut p = participant();
    p.track_dying = true;
    p.current_hp = Some(0);
    p.conditions = vec![condition(1, ConditionRule::Dying, Some(1))];
    let sources = possible(ConditionRule::Doomed, 2);
    let failed = super::lifecycle::prepare_lifecycle(
        &p,
        &Action::DyingRecovery {
            check: Input::TableDegree {
                degree: Degree::Failure,
            },
        },
        None,
        "failed",
        &sources,
    );
    let error = match failed {
        Ok(_) => panic!("uncertain death must not be chosen"),
        Err(error) => error.into_app_error(),
    };
    assert!(error.message.contains("Doomed") && error.message.contains("Maintained condition"));
    let success = super::lifecycle::prepare_lifecycle(
        &p,
        &Action::DyingRecovery {
            check: Input::TableDegree {
                degree: Degree::Success,
            },
        },
        None,
        "success",
        &sources,
    )
    .unwrap()
    .resolve(Some(Degree::Success))
    .unwrap();
    assert!(success.condition_changes.iter().any(|change| matches!(change, ConditionEventChange::Add(row) if row.name == "Wounded" && row.value == Some(1))));
    assert!(
        super::lifecycle::prepare_lifecycle(
            &p,
            &Action::DyingRecovery { check: Input::Roll },
            None,
            "roll",
            &sources
        )
        .is_err()
    );
}

#[test]
fn a_known_fatal_doomed_value_stays_fatal_when_recovery_reduces_dying_below_zero() {
    use atlas_app_model::{
        CheckDegreeView as Degree, ConditionLifecycleActionView as Action,
        LifecycleCheckInputView as Input,
    };
    let mut p = participant();
    p.track_dying = true;
    p.current_hp = Some(0);
    p.conditions = vec![condition(1, ConditionRule::Dying, Some(1))];
    let sources = inherited(ConditionRule::Doomed, 4);
    let changes = super::lifecycle::prepare_lifecycle(
        &p,
        &Action::DyingRecovery {
            check: Input::TableDegree {
                degree: Degree::CriticalSuccess,
            },
        },
        None,
        "recover",
        &sources,
    )
    .unwrap()
    .resolve(Some(Degree::CriticalSuccess))
    .unwrap();
    assert_eq!(
        changes.participant_changes[0].mortality,
        MortalityState::Dead
    );
}

#[test]
fn healing_uses_wounded_activated_by_ending_dying() {
    let mut p = participant();
    p.track_dying = true;
    p.current_hp = Some(0);
    p.conditions = vec![condition(1, ConditionRule::Dying, Some(1))];
    let sources = inherited_with_predicate(
        ConditionRule::Wounded,
        2,
        json!([{"not":"self:condition:dying"}]),
    );
    let changes = plan_hp_event(
        &p,
        &sources,
        &HpEventInput::ResolvedHealing { amount: 5 },
        &Default::default(),
        Some(60),
        "healing",
    )
    .unwrap();
    assert!(changes.condition_changes.iter().any(|change| matches!(change, ConditionEventChange::Add(row) if row.name == "Wounded" && row.value == Some(3))));
}

#[test]
fn lesser_possible_slowed_does_not_hide_a_known_regain_allotment() {
    let mut p = participant();
    p.conditions = vec![condition(1, ConditionRule::Slowed, Some(2))];
    let sources = possible(ConditionRule::Slowed, 1);
    assert_eq!(
        super::capabilities::RegainPlan::evaluate(&p, &sources).total(),
        Some(1)
    );
    p.conditions = vec![condition(1, ConditionRule::Quickened, None)];
    let sources = possible(ConditionRule::Quickened, 0);
    assert_eq!(
        super::capabilities::RegainPlan::evaluate(&p, &sources).total(),
        Some(4)
    );
}

#[test]
fn current_stunned_uses_effective_sources_and_preserves_immutable_remainder() {
    let mut p = participant();
    open_turn(&mut p);
    let sources = inherited(ConditionRule::Stunned, 1);
    let row = condition(1, ConditionRule::Stunned, Some(2));
    let input = atlas_local_state::AddEncounterParticipantCondition {
        participant_key: p.participant_key.clone(),
        condition_key: row.condition_key,
        name: row.name,
        value: row.value,
        source_participant_key: None,
        scope: Default::default(),
        extra_action: Default::default(),
        immunity_override: false,
        parameters: Default::default(),
        origin: None,
        duration: None,
        note: None,
    };
    let mut planned = super::events::plan_condition_change(
        &p,
        &sources,
        ConditionEventChange::Add(input),
        true,
        false,
        Some(5),
        "stun",
    )
    .unwrap();
    super::stunned_arrival::apply(&p, &sources, &mut planned).unwrap();
    assert_eq!(planned.changes.turn_resources[0].regular_remaining, 1);
    assert!(planned.changes.condition_changes.is_empty());
    assert!(planned.effects.iter().any(|effect| matches!(
        effect,
        super::events::ConditionConsequence::StunnedActionsLost {
            stunned_after: Some(1),
            ..
        }
    )));
}

#[test]
fn capped_stunned_loss_survives_unknown_higher_values_without_claiming_an_exact_value() {
    let mut p = participant();
    open_turn(&mut p);
    p.conditions = vec![condition(1, ConditionRule::Stunned, Some(3))];
    let sources = possible(ConditionRule::Stunned, 5);
    assert_eq!(
        super::capabilities::stunned_resolution_loss(&p, &sources),
        Some(3)
    );
    let transition = super::lifecycle::consume_numeric_stunned(&p, 3, &sources).unwrap();
    assert_eq!(transition.changes.turn_resources[0].regular_remaining, 0);
    assert!(matches!(
        transition.changes.turn_resources[0]
            .allocation
            .as_ref()
            .unwrap()
            .adjustments
            .last(),
        Some(atlas_local_state::TurnActionAdjustment::StunnedDuringTurn {
            value: None,
            lost: 3
        })
    ));
    p.conditions[0].value = Some(1);
    assert_eq!(
        super::capabilities::stunned_resolution_loss(&p, &sources),
        None
    );
}

#[test]
fn inherited_detection_reaches_both_targeting_and_observer_defense_with_provenance() {
    use super::causes::ConditionSourceKey;
    for (rule, dc, off_guard) in [
        (ConditionRule::Hidden, 11, true),
        (ConditionRule::Concealed, 5, false),
    ] {
        let p = participant();
        let sources = inherited(rule, 0);
        let target = super::relationships::targeting_requirements(&p, "observer", &sources);
        assert_eq!(target.flat_check_dc, Some(dc));
        assert!(!target.needs_context);
        assert!(matches!(
            &target.source_ids[..],
            [ConditionSourceKey::Authored { .. }]
        ));
        let roster = vec![p.clone()];
        let map = [(p.participant_key.clone(), sources)].into_iter().collect();
        let causes = super::relationships::observer_defense_causes("observer", &roster, &map);
        assert_eq!(!causes.is_empty(), off_guard);
        assert!(
            causes
                .iter()
                .all(|cause| cause.rule == ConditionRule::OffGuard && !cause.uncertain)
        );
    }
}

#[test]
fn possible_detection_does_not_invent_an_exact_dc_or_known_off_guard() {
    let mut p = participant();
    p.conditions = vec![condition(1, ConditionRule::Concealed, None)];
    let sources = possible(ConditionRule::Hidden, 0);
    let target = super::relationships::targeting_requirements(&p, "observer", &sources);
    assert_eq!(target.flat_check_dc, None);
    assert!(target.needs_context);
    let roster = vec![p.clone()];
    let map = [(p.participant_key.clone(), sources)].into_iter().collect();
    let causes = super::relationships::observer_defense_causes("observer", &roster, &map);
    assert!(
        causes
            .iter()
            .any(|cause| cause.rule == ConditionRule::OffGuard && cause.uncertain)
    );
}

#[test]
fn unknown_petrified_does_not_block_direct_loss_above_zero() {
    let p = participant();
    let sources = possible(ConditionRule::Petrified, 0);
    let changes = plan_hp_event(
        &p,
        &sources,
        &HpEventInput::DirectLoss { amount: 1 },
        &Default::default(),
        None,
        "loss",
    )
    .unwrap();
    assert_eq!(changes.participant_changes[0].current_hp, Some(36));
    assert!(
        plan_hp_event(
            &p,
            &sources,
            &HpEventInput::DirectLoss { amount: 37 },
            &Default::default(),
            None,
            "zero"
        )
        .is_err()
    );
}

#[test]
fn unavailable_sources_preserve_pending_identity_and_explain_unavailable_knockout_math() {
    let mut p = participant();
    p.track_dying = true;
    p.current_hp = Some(0);
    let pending = atlas_local_state::PendingConsequence::HpLifecycle {
        participant_key: p.participant_key.clone(),
        before_hp: 5,
        after_hp: 0,
        damage_amount: 5,
        was_dying: 0,
        was_wounded: 0,
        was_doomed: 0,
        context: Default::default(),
    };
    let unavailable = SourceConditions::unavailable();
    assert!(super::hp_events::pending_is_current(&p, &unavailable, &pending).is_err());
    let options = super::hp_events::knockout_options(&p, &unavailable, None);
    assert!(options.options.is_empty());
    assert!(options.unavailable_reason.unwrap().contains("Dying"));
    p.current_hp = Some(5);
    assert!(!super::hp_events::pending_is_current(&p, &unavailable, &pending).unwrap());
}

#[test]
fn unavailable_sources_do_not_establish_an_exact_hp_cap_or_overnight_admission() {
    let mut p = participant();
    p.record_key = Some("actors:aaaaaaaaaaaaaaaa".into());
    assert_eq!(
        super::effective_hp_maximum(&p, &SourceConditions::unavailable(), Some(5)),
        None
    );
    let mut row = atlas_app_model::EncounterRecoveryParticipantView {
        participant_key: p.participant_key.clone(),
        display_name: p.display_name.clone(),
        current_hp: p.current_hp,
        resulting_hp: p.current_hp,
        resulting_max_hp: p.max_hp,
        healing: None,
        changes: vec![],
        unavailable_reason: None,
    };
    assert!(
        super::overnight::prepare(
            &p,
            Some(5),
            atlas_app_model::OvernightHealingMode::Natural,
            None,
            &mut row
        )
        .unwrap()
        .is_none()
    );
    assert!(row.unavailable_reason.unwrap().contains("Dying"));
    p.record_key = None;
    assert_eq!(
        super::effective_hp_maximum(&p, SourceConditions::empty(), Some(5)),
        Some(60)
    );
}

#[test]
fn resolving_possible_stunned_does_not_invent_a_new_arrival_or_rewrite_stock() {
    let mut p = participant();
    open_turn(&mut p);
    let mut sources = inherited_with_predicate(
        ConditionRule::Stunned,
        2,
        json!(["self:condition:off-guard"]),
    );
    let mut possible_off_guard = possible(ConditionRule::OffGuard, 0).entries.remove(0);
    possible_off_guard.grant_ordinal = Some(1);
    sources.entries.push(possible_off_guard);
    let row = condition(1, ConditionRule::OffGuard, None);
    let input = atlas_local_state::AddEncounterParticipantCondition {
        participant_key: p.participant_key.clone(),
        condition_key: row.condition_key,
        name: row.name,
        value: None,
        source_participant_key: None,
        scope: Default::default(),
        extra_action: Default::default(),
        immunity_override: false,
        parameters: Default::default(),
        origin: None,
        duration: None,
        note: None,
    };
    let mut planned = super::events::plan_condition_change(
        &p,
        &sources,
        ConditionEventChange::Add(input),
        true,
        false,
        Some(5),
        "resolve-context",
    )
    .unwrap();
    let error = super::stunned_arrival::apply(&p, &sources, &mut planned)
        .unwrap_err()
        .into_app_error();
    assert!(
        error.message.contains("already applied") && error.message.contains("Maintained condition")
    );
    assert_eq!(p.turn_resources.as_ref().unwrap().regular_remaining, 3);
    assert!(planned.changes.turn_resources.is_empty());
}

#[test]
fn uncertain_vision_and_dazzled_do_not_become_known_available_or_absent() {
    let mut p = participant();
    let mut baseline = super::numerical_tests::source_baseline();
    baseline.source_conditions = possible(ConditionRule::Blinded, 0);
    let projected = super::project_participant(&p, &baseline, false, &[]);
    assert!(
        projected
            .senses
            .iter()
            .any(|sense| sense.sense_type == "vision" && sense.available.is_none())
    );
    p.conditions = vec![condition(1, ConditionRule::Dazzled, None)];
    assert_eq!(
        super::sensory_context::dazzled_concealment(&p, "other", Some(&baseline), None, None)
            .applies,
        None
    );
}

#[test]
fn uncertain_deafened_marks_only_relevant_initiative_and_stupefied_hint_stays_unknown() {
    use crate::encounters::stat_block::StatisticTarget;
    let mut p = participant();
    let mut baseline = super::numerical_tests::source_baseline();
    baseline.source_conditions = possible(ConditionRule::Deafened, 0);
    let ordinary = super::project_participant(&p, &baseline, false, &[]);
    assert!(
        ordinary
            .effective_stat(&StatisticTarget::Perception)
            .is_some()
    );
    let initiative = super::project_initiative(ordinary, &p);
    assert!(
        initiative
            .effective_stat(&StatisticTarget::Perception)
            .is_none()
    );
    p.conditions = vec![condition(1, ConditionRule::Stupefied, Some(1))];
    assert_eq!(
        super::action_context::stupefied_casting_hint(
            &p,
            None,
            &possible(ConditionRule::Stupefied, 3)
        ),
        Some(Err(()))
    );
    assert_eq!(
        super::action_context::stupefied_casting_hint(
            &p,
            None,
            &possible(ConditionRule::Stupefied, 1)
        ),
        Some(Ok(6))
    );
}

#[test]
fn changing_scope_rechecks_source_predicates_against_the_full_replacement() {
    let mut p = participant();
    let mut row = condition(1, ConditionRule::OffGuard, None);
    row.scope = atlas_local_state::ConditionScope::SelectedParticipants {
        participant_keys: vec!["opponent".into()],
    };
    p.conditions = vec![row.clone()];
    let sources = inherited_with_predicate(
        ConditionRule::Drained,
        1,
        json!(["self:condition:off-guard"]),
    );
    let mut replacement = super::hp_events::replace_counter(&p, &row, 0);
    replacement.value = None;
    replacement.scope = atlas_local_state::ConditionScope::Everyone;
    replacement.note = Some("new scope".into());
    let planned = super::events::plan_condition_change(
        &p,
        &sources,
        ConditionEventChange::Replace(replacement),
        true,
        false,
        Some(5),
        "scope",
    )
    .unwrap();
    assert_eq!(planned.changes.participant_changes[0].current_hp, Some(32));
    assert!(
        planned
            .effects
            .iter()
            .any(|effect| matches!(effect, super::events::ConditionConsequence::DirectHpLoss(5)))
    );
    let after =
        super::events::subject_after_changes(&p, &planned.changes.condition_changes).unwrap();
    assert_eq!(after.conditions[0].note.as_deref(), Some("new scope"));
}
