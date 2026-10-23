//! Bounded PF2e 8.6.1 assistance: current permission and a read-only regain forecast.
use super::super::conditions::{ConditionRule, condition_rule_for_key};
use atlas_app_model::{
    ActionBudgetView, RuntimeAdjustmentView, RuntimeCapabilityView, RuntimeCountView,
    RuntimeEffectNoteView,
};
use atlas_local_state::EncounterParticipant;

pub(in crate::encounters) fn cannot_act_reason(p: &EncounterParticipant) -> Option<&'static str> {
    if p.mortality == atlas_local_state::MortalityState::Dead {
        return Some("Dead.");
    }
    if p.defeated {
        return Some("Marked defeated.");
    }
    if p.hazard_disabled {
        return Some("Hazard disabled.");
    }
    super::causes::condition_causes(p, None)
        .into_iter()
        .find_map(|cause| match cause.rule {
            ConditionRule::Stunned if cause.value != Some(0) => Some("Stunned."),
            ConditionRule::Unconscious => Some("Unconscious."),
            ConditionRule::Petrified => Some("Petrified."),
            ConditionRule::Broken => Some("Broken."),
            _ => None,
        })
}

#[derive(Debug, Clone, Copy)]
struct RegainForecast {
    slowed: i64,
    stunned: i64,
    quickened: bool,
    duration_stun: bool,
    unknown: bool,
    allotment: i64,
    lost_to_stun: i64,
    lost_to_slowed: i64,
}
fn regain_forecast(p: &EncounterParticipant) -> RegainForecast {
    let mut slowed = 0;
    let mut stunned = 0;
    let mut quickened = false;
    let mut duration_stun = false;
    let mut unknown = false;
    for cause in super::causes::condition_causes(p, None) {
        let c = cause.source;
        match cause.rule {
            ConditionRule::Quickened => quickened = true,
            ConditionRule::Slowed => match c.value {
                Some(v) if v >= 0 => slowed = slowed.max(v),
                _ => unknown = true,
            },
            ConditionRule::Stunned => match c.value {
                Some(v) if v >= 0 => stunned = stunned.max(v),
                None if c.duration.is_some() => duration_stun = true,
                _ => unknown = true,
            },
            _ => {}
        }
    }
    let allotment = 3 + i64::from(quickened);
    let lost_to_stun = if duration_stun {
        allotment
    } else {
        stunned.min(allotment)
    };
    let lost_to_slowed = (slowed.min(allotment) - lost_to_stun).max(0);
    RegainForecast {
        slowed,
        stunned,
        quickened,
        duration_stun,
        unknown,
        allotment,
        lost_to_stun,
        lost_to_slowed,
    }
}
pub(in crate::encounters) fn numeric_stunned_actions_lost(p: &EncounterParticipant) -> Option<i64> {
    let forecast = regain_forecast(p);
    if forecast.unknown {
        None
    } else if p.defeated
        || p.hazard_disabled
        || p.mortality == atlas_local_state::MortalityState::Dead
    {
        Some(0)
    } else {
        Some(forecast.lost_to_stun)
    }
}
pub(in crate::encounters) fn action_budget(p: &EncounterParticipant) -> ActionBudgetView {
    let RegainForecast {
        slowed,
        stunned,
        quickened,
        duration_stun,
        unknown,
        allotment,
        lost_to_stun,
        lost_to_slowed,
    } = regain_forecast(p);
    let inactive =
        p.defeated || p.hazard_disabled || p.mortality == atlas_local_state::MortalityState::Dead;
    let mut actions = count(
        "Actions",
        3,
        if inactive || duration_stun {
            Some(0)
        } else if unknown {
            None
        } else {
            Some(allotment - lost_to_stun - lost_to_slowed)
        },
    );
    let mut notes = Vec::new();
    if quickened {
        actions
            .adjustments
            .push(adjustment("quickened", "Quickened", 1, None));
    }
    if inactive {
        actions
            .adjustments
            .push(adjustment("inactive", "Inactive", -allotment, None));
    } else {
        if lost_to_stun > 0 {
            let label = if duration_stun {
                "Stunned (duration)".into()
            } else {
                format!("Stunned {stunned}")
            };
            actions
                .adjustments
                .push(adjustment("stunned", &label, -lost_to_stun, None));
        }
        if lost_to_slowed > 0 {
            let overlap = slowed.min(allotment).min(lost_to_stun);
            actions.adjustments.push(adjustment(
                "slowed",
                &format!("Slowed {slowed}"),
                -lost_to_slowed,
                (overlap > 0).then(|| {
                    format!(
                        "{overlap} {} already lost to Stunned.",
                        if overlap == 1 { "action" } else { "actions" }
                    )
                }),
            ));
        } else if slowed > 0 && lost_to_stun > 0 {
            actions.suppressed_adjustments.push(adjustment(
                "slowed",
                &format!("Slowed {slowed}"),
                -slowed.min(allotment),
                Some("Overridden by Stunned".into()),
            ));
        }
    }
    if unknown {
        notes.push(note(
            "unresolved",
            "Needs review",
            "A condition has no usable value or duration. Edit it before relying on the forecast.",
        ));
    }
    let reason = cannot_act_reason(p);
    let reaction_reason = reason.or_else(|| {
        super::causes::condition_causes(p, None)
            .iter()
            .any(|cause| cause.rule == ConditionRule::Confused)
            .then_some("Confused.")
    });
    let capability = |reason: Option<&str>| RuntimeCapabilityView {
        available: reason.is_none(),
        source: reason.map(|_| {
            if inactive {
                "participant_status"
            } else {
                "encounter_state"
            }
            .into()
        }),
        reason: reason.map(str::to_owned),
    };
    ActionBudgetView {
        extra_action_unavailable_reason: if !quickened {
            None
        } else if inactive {
            Some("Unavailable".into())
        } else if lost_to_stun > 0 {
            Some("Lost to Stunned".into())
        } else if lost_to_slowed > 0 {
            Some("Lost to Slowed".into())
        } else {
            None
        },
        quickened_sources: p
            .conditions
            .iter()
            .filter(|c| {
                condition_rule_for_key(c.condition_key.as_deref()) == Some(ConditionRule::Quickened)
            })
            .map(|c| super::super::projection::condition_extra_action_view(c.extra_action.clone()))
            .collect(),
        actions,
        reactions: count("Reactions", 1, Some(i64::from(reaction_reason.is_none()))),
        can_act: capability(reason),
        can_react: capability(reaction_reason),
        notes,
    }
}
fn count(label: &str, base_value: i64, adjusted_value: Option<i64>) -> RuntimeCountView {
    RuntimeCountView {
        label: label.into(),
        explanation: None,
        base_value,
        adjusted_value,
        segments: vec![],
        adjustments: vec![],
        suppressed_adjustments: vec![],
    }
}
fn adjustment(
    source: &str,
    label: &str,
    value: i64,
    reason: Option<String>,
) -> RuntimeAdjustmentView {
    RuntimeAdjustmentView {
        source: source.into(),
        label: label.into(),
        value,
        reason,
    }
}
fn note(source: &str, label: &str, reason: &str) -> RuntimeEffectNoteView {
    RuntimeEffectNoteView {
        source: source.into(),
        label: label.into(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_local_state::{
        ConditionDuration, ConditionDurationUnit, EncounterParticipantCondition,
        ParticipantHpOrigin, ParticipantKind, ParticipantSide, ParticipantVariant,
        ParticipantVariantOrigin,
    };

    fn participant() -> EncounterParticipant {
        EncounterParticipant {
            level_override: None,
            base_death_threshold: 4,
            persistent_precedence: vec![],
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
            initiative: Some(20),
            initiative_order: 0,
            max_hp: Some(20),
            current_hp: Some(20),
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
    fn condition(name: &str, value: Option<i64>) -> EncounterParticipantCondition {
        let catalog = super::super::super::conditions::condition_catalog();
        let definition = catalog.conditions.iter().find(|c| c.name == name).unwrap();
        EncounterParticipantCondition {
            parameters: Default::default(),
            origin: None,
            scope: Default::default(),
            extra_action: Default::default(),
            immunity_override: false,
            condition_id: 1,
            condition_key: Some(definition.condition_ref.clone()),
            name: name.into(),
            value,
            source_participant_key: None,
            duration: None,
            note: None,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }
    #[test]
    fn extra_action_status_follows_the_known_action_loss() {
        let mut p = participant();
        p.conditions = vec![condition("Quickened", None)];
        assert_eq!(action_budget(&p).extra_action_unavailable_reason, None);
        p.conditions.push(condition("Slowed", Some(1)));
        let budget = action_budget(&p);
        assert_eq!(
            budget.extra_action_unavailable_reason.as_deref(),
            Some("Lost to Slowed")
        );
        assert_eq!(budget.actions.adjusted_value, Some(3));
        assert!(
            budget
                .actions
                .adjustments
                .iter()
                .all(|row| row.reason.is_none())
        );
        p.conditions.push(condition("Stunned", Some(2)));
        let budget = action_budget(&p);
        assert_eq!(
            budget.extra_action_unavailable_reason.as_deref(),
            Some("Lost to Stunned")
        );
        assert_eq!(budget.actions.adjusted_value, Some(2));
        assert_eq!(budget.actions.adjustments.len(), 2);
        assert_eq!(budget.actions.suppressed_adjustments.len(), 1);
        assert_eq!(budget.actions.suppressed_adjustments[0].label, "Slowed 1");
        assert_eq!(
            budget.actions.suppressed_adjustments[0].reason.as_deref(),
            Some("Overridden by Stunned")
        );
    }

    #[test]
    fn complete_allotment_and_overlap_are_forecast_without_consuming_conditions() {
        let mut p = participant();
        p.conditions = vec![condition("Quickened", None), condition("Slowed", Some(4))];
        assert_eq!(action_budget(&p).actions.adjusted_value, Some(0));
        assert!(
            action_budget(&p).can_act.available,
            "Slowed is a regain loss, not immediate inability to act"
        );
        p.conditions = vec![condition("Stunned", Some(1)), condition("Slowed", Some(2))];
        let before = p.clone();
        for _ in 0..2 {
            let budget = action_budget(&p);
            assert_eq!(budget.actions.adjusted_value, Some(1));
            assert!(!budget.can_act.available);
            assert!(!budget.can_react.available);
            assert_eq!(
                budget
                    .actions
                    .adjustments
                    .iter()
                    .map(|a| a.value)
                    .sum::<i64>(),
                -2
            );
            assert!(budget.actions.suppressed_adjustments.is_empty());
            let slowed = budget
                .actions
                .adjustments
                .iter()
                .find(|row| row.source == "slowed")
                .unwrap();
            assert_eq!(slowed.label, "Slowed 2");
            assert_eq!(slowed.value, -1);
            assert_eq!(
                slowed.reason.as_deref(),
                Some("1 action already lost to Stunned.")
            );
        }
        assert_eq!(p, before);
    }
    #[test]
    fn duration_stun_and_unknown_stun_are_never_stunned_one() {
        let mut p = participant();
        p.conditions = vec![condition("Stunned", None)];
        assert_eq!(action_budget(&p).actions.adjusted_value, None);
        p.conditions[0].duration = Some(ConditionDuration {
            amount: 2,
            unit: ConditionDurationUnit::Minutes,
            timing: None,
        });
        assert_eq!(action_budget(&p).actions.adjusted_value, Some(0));
        p.conditions[0].duration.as_mut().unwrap().amount = 0;
        assert!(
            !action_budget(&p).can_act.available,
            "Expiry still requires explicit removal"
        );
        p.conditions[0].value = Some(0);
        assert!(
            action_budget(&p).can_act.available,
            "Explicit numeric recovery to zero permits acting"
        );
    }
    #[test]
    fn inactive_status_preserves_hp_and_blocks_current_capability() {
        let mut p = participant();
        p.defeated = true;
        assert_eq!(action_budget(&p).actions.adjusted_value, Some(0));
        assert!(!action_budget(&p).can_act.available);
        assert_eq!(
            action_budget(&p).can_act.source.as_deref(),
            Some("participant_status")
        );
        p.defeated = false;
        p.participant_kind = ParticipantKind::Hazard;
        p.hazard_disabled = true;
        assert!(!action_budget(&p).can_react.available);
        assert_eq!(
            action_budget(&p).can_react.source.as_deref(),
            Some("participant_status")
        );
        assert_eq!(p.current_hp, Some(20));
        p.hazard_disabled = false;
        assert!(action_budget(&p).can_act.available);
        p.mortality = atlas_local_state::MortalityState::Dead;
        assert_eq!(
            action_budget(&p).can_act.source.as_deref(),
            Some("participant_status")
        );
        p.mortality = atlas_local_state::MortalityState::Alive;
        p.conditions = vec![condition("Unconscious", None)];
        assert_eq!(
            action_budget(&p).can_act.source.as_deref(),
            Some("encounter_state")
        );
    }
    #[test]
    fn maintained_grants_keep_independent_sources_and_scopes() {
        use super::super::causes::condition_causes;
        let mut p = participant();
        let mut grabbed = condition("Grabbed", None);
        grabbed.condition_id = 10;
        grabbed.scope = atlas_local_state::ConditionScope::SelectedParticipants {
            participant_keys: vec!["a".into()],
        };
        let mut independent = condition("Off-Guard", None);
        independent.condition_id = 20;
        p.conditions = vec![grabbed, independent];
        let general = condition_causes(&p, None);
        assert_eq!(
            general
                .iter()
                .filter(|cause| cause.rule == ConditionRule::OffGuard)
                .map(|cause| cause.source.condition_id)
                .collect::<Vec<_>>(),
            vec![20]
        );
        let targeted = condition_causes(&p, Some("a"));
        assert_eq!(
            targeted
                .iter()
                .filter(|cause| cause.rule == ConditionRule::OffGuard)
                .map(|cause| cause.source.condition_id)
                .collect::<Vec<_>>(),
            vec![10, 20]
        );
        p.conditions.remove(0);
        assert_eq!(condition_causes(&p, Some("a")).len(), 1);
    }
    #[test]
    fn conditional_immunity_is_contextual_and_does_not_become_a_prohibition() {
        use super::super::{
            ConditionApplicabilityFacts, ConditionImmunity, condition_applicability,
        };
        let p = participant();
        let rule = super::super::super::conditions::modeled_condition_by_ref(
            "conditionitems:TBSHQspnbcqxsmjL",
        )
        .unwrap();
        let mut facts = ConditionApplicabilityFacts {
            immunities: vec![ConditionImmunity {
                slug: "frightened".into(),
                coverage: crate::encounters::rules::ImmunityCoverage::Conditional,
            }]
            .into(),
            broken_threshold: None,
            broken_threshold_known_absent: false,
        };
        assert!(matches!(
            condition_applicability(rule, &p, &facts),
            Some(atlas_app_model::EncounterConditionEligibilityView::NeedsContext { .. })
        ));
        facts.immunities.entries[0].coverage =
            crate::encounters::rules::ImmunityCoverage::Unconditional;
        assert!(matches!(
            condition_applicability(rule, &p, &facts),
            Some(atlas_app_model::EncounterConditionEligibilityView::Immune { .. })
        ));
    }
    #[test]
    fn unconscious_blocks_actions_but_paralyzed_retains_mental_actions() {
        let mut p = participant();
        p.conditions = vec![condition("Unconscious", None)];
        assert!(!action_budget(&p).can_act.available);
        p.conditions = vec![condition("Paralyzed", None)];
        assert!(action_budget(&p).can_act.available);
        p.conditions = vec![condition("Confused", None)];
        assert!(action_budget(&p).can_act.available);
        assert!(!action_budget(&p).can_react.available);
    }

    #[test]
    fn casting_disruption_uses_one_strongest_check_and_preserves_all_causes() {
        use super::super::action_context::*;
        let mut p = participant();
        p.conditions = vec![
            condition("Grabbed", None),
            condition("Deafened", None),
            condition("Stupefied", Some(2)),
        ];
        for (index, condition) in p.conditions.iter_mut().enumerate() {
            condition.condition_id = index as i64 + 1;
        }
        let facts = [
            ActionJudgment {
                source_id: None,
                fact: ActionFact::Manipulate,
                value: true,
            },
            ActionJudgment {
                source_id: None,
                fact: ActionFact::Auditory,
                value: true,
            },
            ActionJudgment {
                source_id: None,
                fact: ActionFact::Move,
                value: false,
            },
        ];
        let context = ActionContext {
            kind: ActionKind::Cast,
            reaction: Some(false),
            other_participant: None,
            facts: &facts,
        };
        let result = evaluate_action(&p, &context);
        assert!(result.issues.is_empty());
        assert_eq!(
            result.disruption,
            Some(DisruptionCheck {
                dc: 7,
                source_ids: vec![1, 2, 3]
            })
        );
        p.conditions.reverse();
        assert_eq!(evaluate_action(&p, &context).disruption, result.disruption);
    }

    #[test]
    fn paralyzed_does_not_assume_a_spell_without_manipulate_is_mental() {
        use super::super::action_context::*;
        let mut p = participant();
        p.conditions = vec![condition("Paralyzed", None)];
        let context = ActionContext {
            kind: ActionKind::Cast,
            reaction: Some(false),
            other_participant: None,
            facts: &[],
        };
        assert_eq!(
            evaluate_action(&p, &context).issues,
            vec![ActionIssue::NeedsFact {
                source_id: 1,
                fact: ActionFact::MentalOnly
            }]
        );
        let facts = [ActionJudgment {
            source_id: None,
            fact: ActionFact::MentalOnly,
            value: true,
        }];
        let allowed = ActionContext {
            facts: &facts,
            ..context
        };
        assert!(evaluate_action(&p, &allowed).issues.is_empty());
    }

    #[test]
    fn independent_fascination_sources_require_independent_judgments() {
        use super::super::action_context::*;
        let mut p = participant();
        p.conditions = vec![condition("Fascinated", None), condition("Fascinated", None)];
        p.conditions[1].condition_id = 2;
        for (index, condition) in p.conditions.iter_mut().enumerate() {
            condition.parameters = atlas_local_state::ConditionParameters::Fascinated {
                subject: Some(atlas_local_state::ConditionSubject::Other {
                    description: format!("subject {index}"),
                }),
            };
        }
        let facts = [
            ActionJudgment {
                source_id: None,
                fact: ActionFact::Concentrate,
                value: true,
            },
            ActionJudgment {
                source_id: Some(1),
                fact: ActionFact::RelatesToFascination,
                value: true,
            },
        ];
        let context = ActionContext {
            kind: ActionKind::Retch,
            reaction: Some(false),
            other_participant: None,
            facts: &facts,
        };
        assert_eq!(
            evaluate_action(&p, &context).issues,
            vec![ActionIssue::NeedsFact {
                source_id: 2,
                fact: ActionFact::RelatesToFascination
            }]
        );
    }
    #[test]
    fn hidden_relationship_changes_observer_defense_only_against_the_subject() {
        use super::super::relationships::*;
        let mut subject = participant();
        subject.participant_key = "subject".into();
        let mut hidden = condition("Hidden", None);
        hidden.scope = atlas_local_state::ConditionScope::SelectedParticipants {
            participant_keys: vec!["observer".into()],
        };
        subject.conditions.push(hidden);
        let roster = vec![subject];
        let causes = observer_defense_causes("observer", &roster);
        assert_eq!(causes.len(), 1);
        assert_eq!(causes[0].against, Some("subject"));
        assert_eq!(causes[0].rule, ConditionRule::OffGuard);
        assert!(observer_defense_causes("other", &roster).is_empty());
        assert!(observer_defense_causes("subject", &roster).is_empty());
        assert_eq!(
            targeting_requirements(&roster[0], "observer").flat_check_dc,
            Some(11)
        );
    }

    #[test]
    fn undetected_requires_a_guess_without_leaking_failure_reason() {
        use super::super::relationships::*;
        let mut subject = participant();
        subject.conditions = vec![condition("Undetected", None), condition("Concealed", None)];
        let direct = targeting_requirements(&subject, "observer");
        assert!(direct.requires_guessed_location);
        assert_eq!(direct.flat_check_dc, Some(11));
    }
}
