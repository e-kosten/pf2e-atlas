use super::super::conditions::{ConditionRule, modeled_condition_by_rule};
use super::super::stat_block::StatisticTarget;
use super::action_context::ConfusionChoice;
use super::*;
use atlas_local_state::*;
use serde_json::json;

pub(super) fn participant() -> EncounterParticipant {
    EncounterParticipant {
        level_override: None,
        base_death_threshold: 4,
        persistent_selections: vec![],
        track_dying: false,
        mortality: Default::default(),
        spell_resources: vec![],
        expended_spells: vec![],
        revision: 0,
        hp_origin: ParticipantHpOrigin::Explicit,
        variant_origin: ParticipantVariantOrigin::Explicit,
        participant_key: "p".into(),
        record_key: None,
        participant_kind: ParticipantKind::Creature,
        participant_variant: ParticipantVariant::Normal,
        position: 0,
        display_name: "Creature".into(),
        record_title_snapshot: None,
        record_kind_snapshot: None,
        side: ParticipantSide::Enemy,
        initiative: None,
        initiative_order: 0,
        max_hp: Some(60),
        current_hp: Some(37),
        temporary_hp: 0,
        defeated: false,
        hazard_disabled: false,
        hidden: false,
        note: None,
        created_at: String::new(),
        updated_at: String::new(),
        conditions: vec![],
    }
}
pub(super) fn condition(
    id: i64,
    rule: ConditionRule,
    value: Option<i64>,
) -> EncounterParticipantCondition {
    let definition = modeled_condition_by_rule(rule).unwrap();
    EncounterParticipantCondition {
        condition_id: id,
        condition_key: Some(definition.condition_ref.into()),
        name: definition.name.into(),
        value,
        source_participant_key: None,
        scope: Default::default(),
        extra_action: Default::default(),
        immunity_override: false,
        parameters: Default::default(),
        origin: None,
        duration: None,
        note: None,
        created_at: String::new(),
        updated_at: String::new(),
    }
}
fn source_baseline() -> super::super::stat_block::StatBlockView {
    let input = atlas_search::test_support::record(
        "actors",
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","name":"Caster", "system":{"details":{"level":{"value":4}}, "attributes":{"adjustment":null,"ac":{"value":20},"hp":{"max":60}}, "abilities":{"str":{"mod":3},"dex":{"mod":2},"con":{"mod":2},"int":{"mod":4},"wis":{"mod":3},"cha":{"mod":1}}, "perception":{"mod":11,"senses":[],"vision":true}}, "items":[
            {"_id":"bbbbbbbbbbbbbbbb","type":"spellcastingEntry","name":"Arcane","system":{"ability":{"value":"int"},"spelldc":{"value":12,"dc":22}}},
            {"_id":"cccccccccccccccc","type":"spell","name":"Bolt","system":{"level":{"value":1},"location":{"value":"bbbbbbbbbbbbbbbb"},"traits":{"value":["attack"]},"ritual":null,"damage":{"first":{"formula":"1d6","type":"fire","kinds":["damage"],"category":null}}}},
            {"_id":"dddddddddddddddd","type":"melee","name":"Finesse claw","system":{"range":null,"bonus":{"value":14},"traits":{"value":["finesse"]},"damageRolls":{"first":{"damage":"1d6+4","damageType":"slashing","category":null}}}}
        ]}),
    );
    super::source_baseline(&input.record, "fixture", None).unwrap()
}
#[test]
fn typed_spell_statistics_and_damage_share_variant_and_condition_policy() {
    let baseline = source_baseline();
    let mut participant = participant();
    participant.participant_variant = ParticipantVariant::Elite;
    participant.conditions = vec![
        condition(1, ConditionRule::Sickened, Some(2)),
        condition(2, ConditionRule::Stupefied, Some(3)),
        condition(3, ConditionRule::Enfeebled, Some(2)),
        condition(4, ConditionRule::Clumsy, Some(1)),
        condition(5, ConditionRule::Drained, Some(1)),
    ];
    let original = participant.clone();
    let projected = project_participant(&participant, &baseline, true, &[]);
    let spell = projected
        .activities
        .iter()
        .find(|activity| activity.label == "Bolt")
        .unwrap();
    assert_eq!(spell.rolls[0].adjusted_value.as_i64(), Some(11));
    assert_eq!(
        spell
            .difficulty_class
            .as_ref()
            .unwrap()
            .adjusted_value
            .as_i64(),
        Some(21)
    );
    assert_eq!(spell.damage[0].adjusted_formula.as_deref(), Some("(1d6)+4"));
    let strike = projected
        .activities
        .iter()
        .find(|activity| activity.label == "Finesse claw")
        .unwrap();
    assert_eq!(strike.rolls[0].adjusted_value.as_i64(), Some(14));
    assert_eq!(strike.damage[0].adjusted_formula, None);
    assert_eq!(
        projected
            .effective_stat(&StatisticTarget::HitPointMaximum)
            .unwrap()
            .as_i64(),
        Some(55)
    );
    assert_eq!(participant, original);
    participant.conditions.reverse();
    let reordered = project_participant(&participant, &baseline, true, &[]);
    assert_eq!(
        reordered.effective_stat(&StatisticTarget::HitPointMaximum),
        projected.effective_stat(&StatisticTarget::HitPointMaximum)
    );
    assert_eq!(
        reordered.activities[1].rolls[0].adjusted_value,
        spell.rolls[0].adjusted_value
    );
}
#[test]
fn unconscious_grants_compete_without_erasing_independent_off_guard() {
    let mut participant = participant();
    participant.conditions = vec![
        condition(1, ConditionRule::Unconscious, None),
        condition(2, ConditionRule::OffGuard, None),
    ];
    let projected = project_participant(&participant, &source_baseline(), true, &[]);
    assert_eq!(
        projected
            .effective_stat(&StatisticTarget::ArmorClass)
            .unwrap()
            .as_i64(),
        Some(14)
    );
    assert_eq!(projected.senses[0].available, Some(false));
    participant.conditions.remove(0);
    let projected = project_participant(&participant, &source_baseline(), true, &[]);
    assert_eq!(
        projected
            .effective_stat(&StatisticTarget::ArmorClass)
            .unwrap()
            .as_i64(),
        Some(18)
    );
    assert_eq!(projected.senses[0].available, Some(true));
}
#[test]
fn petrified_replacement_does_not_require_or_invent_authored_defenses() {
    let input = atlas_search::test_support::record(
        "actors",
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","system":{"attributes":{"adjustment":null}}}),
    );
    let baseline = super::source_baseline(&input.record, "fixture", None).unwrap();
    assert!(
        baseline
            .effective_stat(&StatisticTarget::ArmorClass)
            .is_none()
    );
    let mut participant = participant();
    participant.conditions = vec![condition(7, ConditionRule::Petrified, None)];
    let projected = project_participant(&participant, &baseline, true, &[]);
    assert_eq!(
        projected
            .effective_stat(&StatisticTarget::ArmorClass)
            .unwrap()
            .as_i64(),
        Some(9)
    );
    assert_eq!(
        projected
            .effective_stat(&StatisticTarget::Hardness)
            .unwrap()
            .as_i64(),
        Some(8)
    );
    assert!(
        !projected
            .values
            .iter()
            .any(|value| value.target == StatisticTarget::ArmorClass)
    );
    assert_eq!(projected.replacements[0].condition_id, 7);
}
#[test]
fn scoped_restrained_suppresses_only_the_overlapping_grabbed_cause() {
    let mut participant = participant();
    let mut restrained = condition(2, ConditionRule::Restrained, None);
    restrained.scope = ConditionScope::SelectedParticipants {
        participant_keys: vec!["a".into()],
    };
    participant.conditions = vec![condition(1, ConditionRule::Grabbed, None), restrained];
    let general = super::causes::condition_causes(&participant, None);
    assert!(
        general
            .iter()
            .any(|cause| cause.source.condition_id == 1 && cause.rule == ConditionRule::OffGuard)
    );
    let targeted = super::causes::condition_causes(&participant, Some("a"));
    assert!(!targeted.iter().any(|cause| cause.source.condition_id == 1));
    assert!(
        targeted
            .iter()
            .any(|cause| cause.source.condition_id == 2 && cause.rule == ConditionRule::OffGuard)
    );
}

#[test]
fn explicit_effective_level_controls_drained_without_second_variant_adjustment() {
    let mut participant = participant();
    participant.participant_variant = ParticipantVariant::Elite;
    participant.level_override = Some(7);
    participant.conditions = vec![condition(1, ConditionRule::Drained, Some(2))];
    let projected = project_participant(&participant, &source_baseline(), true, &[]);
    assert_eq!(
        projected
            .adjusted_level
            .as_ref()
            .and_then(serde_json::Number::as_i64),
        Some(7)
    );
    assert_eq!(
        projected
            .effective_stat(&StatisticTarget::HitPointMaximum)
            .and_then(serde_json::Number::as_i64),
        Some(46)
    );
    assert_eq!(effective_hp_maximum(&participant, Some(4)), Some(46));
}

#[test]
fn immobilized_preserves_speed_while_encumbered_reduces_it_independently() {
    let mut baseline = source_baseline();
    baseline
        .speeds
        .push(super::super::stat_block::MovementSpeedView {
            movement_type: "land".into(),
            label: "Speed".into(),
            adjustments: vec![],
            suppressed_adjustments: vec![],
            base_value_feet: 25.into(),
            adjusted_value_feet: 25.into(),
            notes: vec![],
        });
    let mut participant = participant();
    participant.conditions = vec![condition(1, ConditionRule::Immobilized, None)];
    let projected = project_participant(&participant, &baseline, true, &[]);
    assert_eq!(projected.speeds[0].adjusted_value_feet.as_i64(), Some(25));
    participant
        .conditions
        .push(condition(2, ConditionRule::Encumbered, None));
    let projected = project_participant(&participant, &baseline, true, &[]);
    assert_eq!(projected.speeds[0].adjusted_value_feet.as_i64(), Some(15));
}

#[test]
fn deafened_initiative_status_penalty_competes_with_frightened() {
    let mut participant = participant();
    participant.conditions = vec![condition(1, ConditionRule::Deafened, None)];
    let general = project_participant(&participant, &source_baseline(), true, &[]);
    assert_eq!(
        general
            .effective_stat(&StatisticTarget::Perception)
            .and_then(serde_json::Number::as_i64),
        Some(11)
    );
    let initiative = super::project_initiative(general, &participant);
    assert_eq!(
        initiative
            .effective_stat(&StatisticTarget::Perception)
            .and_then(serde_json::Number::as_i64),
        Some(9)
    );
    participant
        .conditions
        .push(condition(2, ConditionRule::Frightened, Some(3)));
    let initiative = super::project_initiative(
        project_participant(&participant, &source_baseline(), true, &[]),
        &participant,
    );
    assert_eq!(
        initiative
            .effective_stat(&StatisticTarget::Perception)
            .and_then(serde_json::Number::as_i64),
        Some(8)
    );
}

#[test]
fn dazzled_targeting_uses_usable_precision_and_blinded_suppresses_it() {
    use super::sensory_context::dazzled_concealment;
    let mut participant = participant();
    participant.conditions = vec![condition(1, ConditionRule::Dazzled, None)];
    let mut baseline = source_baseline();
    assert_eq!(
        dazzled_concealment(&participant, "target", Some(&baseline), None, Some(false)).applies,
        Some(true)
    );
    assert_eq!(
        dazzled_concealment(&participant, "target", None, None, None).applies,
        None
    );
    baseline.sense_facts.push(super::senses::SenseFacts {
        visual: false,
        auditory: true,
        precision: Some(super::senses::Precision::Precise),
        range_feet: Some(40.0),
        available: Some(true),
    });
    assert_eq!(
        dazzled_concealment(&participant, "target", Some(&baseline), Some(20.0), None).applies,
        None
    );
    assert_eq!(
        dazzled_concealment(
            &participant,
            "target",
            Some(&baseline),
            Some(20.0),
            Some(false)
        )
        .applies,
        Some(false)
    );
    assert_eq!(
        dazzled_concealment(&participant, "target", Some(&baseline), Some(50.0), None).applies,
        Some(true)
    );
    participant
        .conditions
        .push(condition(2, ConditionRule::Blinded, None));
    assert_eq!(
        dazzled_concealment(
            &participant,
            "target",
            Some(&baseline),
            Some(20.0),
            Some(true)
        )
        .applies,
        Some(false)
    );
}

#[test]
fn confused_requires_declared_viability_and_projects_self_hit_without_a_roll() {
    use super::action_context::{ActionKind, confusion_requirements};
    let mut actor = participant();
    actor.conditions = vec![condition(1, ConditionRule::Confused, None)];
    let missing = confusion_requirements(&actor, None, ActionKind::Strike, true).unwrap();
    assert!(!missing.required_facts.is_empty());
    let choice = ConfusionChoice::Offensive {
        viable_target_keys: vec!["enemy".into()],
        random_target_key: Some("bystander".into()),
        offensive: Some(true),
    };
    let invalid = confusion_requirements(&actor, Some(&choice), ActionKind::Cast, false).unwrap();
    assert_eq!(invalid.selected_target_key, None);
    assert!(!invalid.required_facts.is_empty());
    let self_hit = confusion_requirements(
        &actor,
        Some(&ConfusionChoice::NoViableTargets),
        ActionKind::Strike,
        true,
    )
    .unwrap();
    assert_eq!(self_hit.selected_target_key.as_deref(), Some("p"));
    assert!(self_hit.automatic_self_hit);
    let wasted = confusion_requirements(
        &actor,
        Some(&ConfusionChoice::UnableToAttack),
        ActionKind::Strike,
        true,
    )
    .unwrap();
    assert!(wasted.wastes_action);
    let facilitate = ConfusionChoice::Facilitate { note: None };
    assert!(
        confusion_requirements(&actor, Some(&facilitate), ActionKind::Retch, false)
            .unwrap()
            .required_facts
            .is_empty()
    );
}

#[test]
fn fascination_requires_its_subject() {
    use super::action_context::{
        ActionContext, ActionFact, ActionIssue, ActionJudgment, ActionKind, evaluate_action,
    };
    let actor = participant();
    let mut fascinated = actor;
    fascinated.conditions = vec![condition(3, ConditionRule::Fascinated, None)];
    let facts = [ActionJudgment {
        source_id: None,
        fact: ActionFact::Concentrate,
        value: true,
    }];
    let evaluate = |participant: &EncounterParticipant| {
        evaluate_action(
            participant,
            &ActionContext {
                kind: ActionKind::Retch,
                reaction: Some(false),
                other_participant: None,
                confusion: None,
                facts: &facts,
            },
        )
    };
    assert!(
        evaluate(&fascinated)
            .issues
            .iter()
            .any(|issue| matches!(issue, ActionIssue::NeedsContext { .. }))
    );
    fascinated.conditions[0].parameters = ConditionParameters::Fascinated {
        subject: Some(ConditionSubject::Other {
            description: "the glowing mirror".into(),
        }),
    };
    assert!(evaluate(&fascinated).issues.iter().any(|issue| matches!(
        issue,
        ActionIssue::NeedsFact {
            fact: ActionFact::RelatesToFascination,
            ..
        }
    )));
}

#[test]
fn fleeing_requires_its_explicit_subject() {
    use super::action_context::{
        ActionContext, ActionFact, ActionIssue, ActionJudgment, ActionKind, evaluate_action,
    };
    let mut actor = participant();
    actor.conditions = vec![condition(1, ConditionRule::Fleeing, None)];
    let facts = [ActionJudgment {
        source_id: Some(1),
        fact: ActionFact::FurthersEscape,
        value: true,
    }];
    let evaluate = |participant: &EncounterParticipant, kind| {
        evaluate_action(
            participant,
            &ActionContext {
                kind,
                reaction: Some(false),
                other_participant: None,
                confusion: None,
                facts: &facts,
            },
        )
    };
    assert!(
        evaluate(&actor, ActionKind::Retch)
            .issues
            .iter()
            .any(|issue| matches!(issue, ActionIssue::NeedsContext { .. }))
    );
    actor.conditions[0].parameters = ConditionParameters::Fleeing {
        subject: Some(ConditionSubject::Other {
            description: "the spreading fire".into(),
        }),
    };
    assert!(evaluate(&actor, ActionKind::Retch).issues.is_empty());
    let issue = crate::encounters::action_projection::issue_view(
        ActionIssue::NeedsFact {
            source_id: 1,
            fact: ActionFact::FurthersEscape,
        },
        &actor,
    );
    assert!(
        matches!(issue.subject, Some(atlas_app_model::ConditionSubjectView::Other { description }) if description == "the spreading fire")
    );
}

#[test]
fn explicit_table_flanking_cause_is_scoped_to_one_attacker_without_erasing_other_causes() {
    let mut defender = participant();
    let mut flanking = condition(1, ConditionRule::OffGuard, None);
    flanking.scope = ConditionScope::SelectedParticipants {
        participant_keys: vec!["attacker-a".into()],
    };
    flanking.note = Some("Flanking".into());
    defender.conditions = vec![flanking];
    let baseline = source_baseline();
    let armor = |opponent| {
        super::project_participant(&defender, &baseline, true, &[])
            .contextual_armor_class
            .into_iter()
            .find(|ac| ac.opponent_key == opponent)
            .and_then(|ac| ac.value.value)
    };
    assert_eq!(
        armor("attacker-a"),
        Some(json!(18).as_number().unwrap().clone())
    );
    assert_eq!(armor("attacker-b"), None); // No special AC for an unaffected attacker.
    assert_eq!(
        super::project_participant(&defender, &baseline, true, &[])
            .effective_stat(&StatisticTarget::ArmorClass)
            .and_then(serde_json::Number::as_i64),
        Some(20)
    );
    defender
        .conditions
        .push(condition(2, ConditionRule::Prone, None));
    defender.conditions.remove(0);
    assert_eq!(
        super::project_participant(&defender, &baseline, true, &[])
            .effective_stat(&StatisticTarget::ArmorClass)
            .and_then(serde_json::Number::as_i64),
        Some(18)
    );
}

#[test]
fn maintained_grant_hints_describe_effects_without_saving_duplicate_conditions() {
    let mut actor = participant();
    actor.conditions = vec![
        condition(1, ConditionRule::Grabbed, None),
        condition(2, ConditionRule::Stupefied, Some(3)),
    ];
    let projected = super::project_participant(&actor, &source_baseline(), true, &[]);
    assert_eq!(
        projected
            .effective_stat(&StatisticTarget::ArmorClass)
            .and_then(serde_json::Number::as_i64),
        Some(18)
    );
    assert!(
        projected
            .unapplied_effects
            .iter()
            .any(|note| note.condition_id == Some(1)
                && note.reason
                    == "Off-Guard and Immobilized. Manipulate actions require a DC 5 flat check.")
    );
    assert!(
        projected
            .unapplied_effects
            .iter()
            .any(|note| note.label == "Stupefied"
                && note.reason == "Spellcasting requires a DC 8 flat check.")
    );
    assert!(
        !projected
            .unapplied_effects
            .iter()
            .any(|note| note.reason == "AC or movement statistics are unavailable.")
    );
    assert_eq!(actor.conditions.len(), 2);
    assert_eq!(
        super::project_participant(&actor, &source_baseline(), true, &[]),
        projected
    );
}

#[test]
fn manual_hints_resolve_suppression_and_scope_without_removing_sources() {
    let mut actor = participant();
    actor.participant_kind = ParticipantKind::Pc;
    actor.conditions = vec![
        condition(1, ConditionRule::Grabbed, None),
        condition(2, ConditionRule::Restrained, None),
        condition(3, ConditionRule::Stupefied, Some(3)),
    ];
    actor.conditions[2].scope = ConditionScope::SelectedParticipants {
        participant_keys: vec!["other".into()],
    };
    let hints = super::participant_runtime_block(&actor).unapplied_effects;
    assert!(hints.iter().any(|hint| hint.condition_id == Some(2)));
    assert!(!hints.iter().any(|hint| hint.condition_id == Some(1)));
    assert!(!hints.iter().any(|hint| hint.label == "Stupefied"));
    assert_eq!(actor.conditions.len(), 3);
    actor.conditions.remove(1);
    let hints = super::participant_runtime_block(&actor).unapplied_effects;
    assert!(
        hints
            .iter()
            .any(|hint| hint.condition_id == Some(1) && hint.reason.contains("DC 5 flat check"))
    );
    assert!(!hints.iter().any(|hint| hint.label == "Stupefied"));
    actor.conditions[1].scope = ConditionScope::Everyone;
    assert!(
        super::participant_runtime_block(&actor)
            .unapplied_effects
            .iter()
            .any(|hint| hint.label == "Stupefied"
                && hint.reason == "Spellcasting requires a DC 8 flat check.")
    );
}

#[test]
fn stupefied_casting_hint_uses_one_effective_dc_for_manual_and_source_backed_actors() {
    let mut actor = participant();
    actor.participant_kind = ParticipantKind::Pc;
    actor.conditions = vec![
        condition(1, ConditionRule::Stupefied, Some(3)),
        condition(2, ConditionRule::Stupefied, Some(5)),
    ];
    let assert_hint = |block: super::super::stat_block::StatBlockView| {
        let hints = block
            .unapplied_effects
            .iter()
            .filter(|hint| hint.label == "Stupefied")
            .collect::<Vec<_>>();
        assert_eq!(hints.len(), 1);
        assert_eq!(hints[0].reason, "Spellcasting requires a DC 10 flat check.");
    };
    assert_hint(super::participant_runtime_block(&actor));
    assert_hint(super::project_participant(
        &actor,
        &source_baseline(),
        true,
        &[],
    ));
    actor.conditions[1].value = None;
    let hints = super::participant_runtime_block(&actor).unapplied_effects;
    assert!(
        hints.iter().any(|hint| hint.label == "Stupefied"
            && hint.reason.starts_with("Enter the Stupefied value"))
    );
    assert!(
        !hints
            .iter()
            .any(|hint| hint.reason == "Spellcasting requires a DC 8 flat check.")
    );
}

#[test]
fn controller_instructions_derive_from_active_sources_and_conflicts_stay_unknown() {
    use super::action_context::control_direction;
    let mut actor = participant();
    let mut controlled = condition(1, ConditionRule::Controlled, None);
    controlled.source_participant_key = Some("controller".into());
    controlled.note = Some("Defend the door".into());
    actor.conditions.push(controlled.clone());
    assert_eq!(
        control_direction(&actor, &controlled),
        Some(("controller".into(), Some("Defend the door".into())))
    );
    let mut competing = controlled.clone();
    competing.condition_id = 2;
    actor.conditions.push(competing);
    assert!(control_direction(&actor, &controlled).is_some());
    actor.conditions[1].note = Some("Leave the door".into());
    assert!(control_direction(&actor, &controlled).is_none());
    actor.conditions[1].source_participant_key = None;
    assert!(control_direction(&actor, &controlled).is_none());
    actor.conditions.pop();
    assert!(control_direction(&actor, &controlled).is_some());
}

#[test]
fn retch_judgments_are_independent_scoped_and_order_independent() {
    use atlas_app_model::{EncounterActionFactView as F, EncounterActionJudgmentView as J};
    let mut actor = participant();
    actor.conditions = vec![
        condition(1, ConditionRule::Fleeing, None),
        condition(2, ConditionRule::Fleeing, None),
        condition(3, ConditionRule::Controlled, None),
    ];
    for source in &mut actor.conditions[..2] {
        source.parameters = ConditionParameters::Fleeing {
            subject: Some(ConditionSubject::Other {
                description: format!("threat {}", source.condition_id),
            }),
        };
    }
    actor.conditions[2].source_participant_key = Some("controller".into());
    let mut answers = vec![
        J {
            source_condition_id: Some(1),
            fact: F::FurthersEscape,
            value: true,
        },
        J {
            source_condition_id: Some(2),
            fact: F::FurthersEscape,
            value: true,
        },
        J {
            source_condition_id: Some(3),
            fact: F::FollowsControllerInstructions,
            value: true,
        },
    ];
    let evaluate = |answers: &[J]| super::super::retch::evaluate(&actor, answers, None);
    assert_eq!(evaluate(&[]).unwrap().issues.len(), 3);
    for _ in 0..answers.len() {
        assert!(evaluate(&answers).unwrap().issues.is_empty());
        answers.rotate_left(1);
    }
    assert_eq!(evaluate(&answers[..2]).unwrap().issues.len(), 1);
    // Repeated identical answers are harmless; a contradiction is never accepted.
    answers.push(answers[0].clone());
    assert!(evaluate(&answers).unwrap().issues.is_empty());
    answers.last_mut().unwrap().value = false;
    assert!(evaluate(&answers).is_err());
    let unrelated = J {
        source_condition_id: Some(99),
        fact: F::FurthersEscape,
        value: true,
    };
    assert!(evaluate(&[unrelated]).is_err());
    assert!(
        evaluate(&[J {
            source_condition_id: None,
            fact: F::FurthersEscape,
            value: true
        }])
        .is_err()
    );
    assert!(
        evaluate(&[J {
            source_condition_id: Some(1),
            fact: F::Manipulate,
            value: true
        }])
        .is_err()
    );
    assert!(
        evaluate(&[J {
            source_condition_id: Some(1),
            fact: F::Manipulate,
            value: false
        }])
        .is_err()
    );
    let prohibited = evaluate(&[J {
        source_condition_id: Some(1),
        fact: F::FurthersEscape,
        value: false,
    }])
    .unwrap();
    assert!(prohibited.issues.iter().any(|issue| matches!(
        issue,
        super::action_context::ActionIssue::Prohibited {
            source_id: 1,
            rule: ConditionRule::Fleeing
        }
    )));
}
