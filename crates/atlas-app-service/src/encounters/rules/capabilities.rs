//! Bounded PF2e 8.6.1 assistance: current permission and a read-only regain forecast.
use super::super::conditions::{ConditionRule, condition_rule_for_key};
use super::source_conditions::SourceConditions;
use atlas_app_model::{
    ActionBudgetView, RuntimeAdjustmentView, RuntimeCapabilityView, RuntimeCountView,
    RuntimeEffectNoteView,
};
use atlas_local_state::{EncounterParticipant, TurnActionAdjustment, TurnActionAllocation};

pub(in crate::encounters) fn cannot_act_reason(
    p: &EncounterParticipant,
    sources: &SourceConditions,
) -> Option<&'static str> {
    if p.mortality == atlas_local_state::MortalityState::Dead {
        return Some("Dead.");
    }
    if p.defeated {
        return Some("Marked defeated.");
    }
    if p.hazard_disabled {
        return Some("Hazard disabled.");
    }
    super::causes::condition_causes(p, None, sources)
        .active
        .into_iter()
        .find_map(|cause| match cause.rule {
            ConditionRule::Stunned if cause.value != Some(0) => Some("Stunned."),
            ConditionRule::Unconscious => Some("Unconscious."),
            ConditionRule::Petrified => Some("Petrified."),
            ConditionRule::Broken => Some("Broken."),
            _ => None,
        })
}

#[derive(Debug, Clone)]
pub(in crate::encounters) struct RegainPlan {
    slowed: i64,
    stunned: i64,
    quickened: bool,
    duration_stun: bool,
    missing: Option<MissingRegainFact>,
    inactive: bool,
    extra_uses: Vec<atlas_local_state::ConditionExtraAction>,
    allotment: i64,
    lost_to_stun: i64,
    lost_to_slowed: i64,
}
#[derive(Debug, Clone)]
enum MissingRegainFact {
    SourceUnavailable(String),
    SlowedValue,
    ConditionContext,
}
impl MissingRegainFact {
    fn reason(&self) -> &str {
        match self {
            Self::SourceUnavailable(reason) => reason,
            Self::SlowedValue => "The source record does not supply a usable Slowed value.",
            Self::ConditionContext => "An action condition's value or activation needs context.",
        }
    }
}
impl RegainPlan {
    pub(in crate::encounters) fn evaluate(
        p: &EncounterParticipant,
        sources: &SourceConditions,
    ) -> Self {
        let mut slowed = 0;
        let mut stunned = 0;
        let mut quickened = false;
        let mut duration_stun = false;
        let resolution = super::causes::condition_causes(p, None, sources);
        let duration_known = resolution.active.iter().any(|cause| {
            cause.rule == ConditionRule::Stunned
                && cause.value.is_none()
                && cause.source.has_duration()
        });
        let mut unknown = resolution.known_value(ConditionRule::Slowed).is_err()
            || resolution.known_presence(ConditionRule::Quickened).is_err()
            || !duration_known && resolution.known_value(ConditionRule::Stunned).is_err();
        for cause in resolution.active {
            let (rule, value, has_duration) =
                (cause.rule, cause.value, cause.source.has_duration());
            match rule {
                ConditionRule::Quickened => quickened = true,
                ConditionRule::Slowed => match value {
                    Some(v) if v >= 0 => slowed = slowed.max(v),
                    _ => unknown = true,
                },
                ConditionRule::Stunned => match value {
                    Some(v) if v >= 0 => stunned = stunned.max(v),
                    None if has_duration => duration_stun = true,
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
        let missing = unknown.then(|| {
            sources
                .unavailable_reason
                .as_ref()
                .map(|reason| MissingRegainFact::SourceUnavailable(reason.clone()))
                .unwrap_or_else(|| {
                    if sources
                        .entries
                        .iter()
                        .any(|c| c.rule == ConditionRule::Slowed && c.value.is_none())
                    {
                        MissingRegainFact::SlowedValue
                    } else {
                        MissingRegainFact::ConditionContext
                    }
                })
        });
        let extra_uses = p
            .conditions
            .iter()
            .filter(|c| {
                condition_rule_for_key(c.condition_key.as_deref()) == Some(ConditionRule::Quickened)
            })
            .map(|c| c.extra_action.clone())
            .chain(
                sources
                    .active_entries(p)
                    .into_iter()
                    .filter(|c| c.rule == ConditionRule::Quickened)
                    .map(|_| atlas_local_state::ConditionExtraAction::Unspecified),
            )
            .collect();
        Self {
            slowed,
            stunned,
            quickened,
            duration_stun,
            missing,
            inactive: p.defeated
                || p.hazard_disabled
                || p.mortality == atlas_local_state::MortalityState::Dead,
            extra_uses,
            allotment,
            lost_to_stun,
            lost_to_slowed,
        }
    }
    pub(in crate::encounters) fn total(&self) -> Option<i64> {
        if self.inactive || self.duration_stun {
            Some(0)
        } else if self.missing.is_some() {
            None
        } else {
            Some(self.allotment - self.lost_to_stun - self.lost_to_slowed)
        }
    }
    pub(in crate::encounters) fn missing_reason(&self) -> Option<&str> {
        self.missing.as_ref().map(MissingRegainFact::reason)
    }
    pub(in crate::encounters) fn stunned_loss(&self) -> Option<i64> {
        if self.missing.is_some() {
            None
        } else if self.inactive {
            Some(0)
        } else {
            Some(self.lost_to_stun)
        }
    }
    pub(in crate::encounters) fn allocation(&self) -> Option<TurnActionAllocation> {
        self.missing.is_none().then(|| self.evaluated_allocation())
    }
    pub(in crate::encounters) fn extra(&self) -> i64 {
        i64::from(
            self.total().is_some_and(|total| total > 0)
                && !self.extra_uses.is_empty()
                && self.extra_unavailable_reason().is_none(),
        )
    }
    pub(in crate::encounters) fn extra_uses(&self) -> Vec<atlas_local_state::ConditionExtraAction> {
        self.extra_uses.clone()
    }
    fn extra_unavailable_reason(&self) -> Option<&'static str> {
        if !self.quickened {
            None
        } else if self.inactive {
            Some("Unavailable")
        } else if self.lost_to_stun > 0 {
            Some("Lost to Stunned")
        } else if self.lost_to_slowed > 0 {
            Some("Lost to Slowed")
        } else {
            None
        }
    }
}
/// Explicit Stunned resolution uses a recorded remaining pool when this turn has begun.
pub(in crate::encounters) fn stunned_resolution_loss(
    p: &EncounterParticipant,
    sources: &SourceConditions,
) -> Option<i64> {
    if p.defeated || p.hazard_disabled || p.mortality == atlas_local_state::MortalityState::Dead {
        return Some(0);
    }
    if let Some(resources) = p
        .turn_resources
        .as_ref()
        .filter(|r| r.pending_reason.is_none() && r.actions_open)
    {
        let remaining = resources.regular_remaining + resources.extra_remaining;
        let conditions = super::causes::condition_causes(p, None, sources);
        if conditions.active.iter().any(|cause| {
            cause.rule == ConditionRule::Stunned
                && cause.value.is_none()
                && cause.source.has_duration()
        }) {
            return Some(remaining);
        }
        return conditions
            .known_capped_value(ConditionRule::Stunned, remaining)
            .ok();
    }
    RegainPlan::evaluate(p, sources).stunned_loss()
}
pub(in crate::encounters) fn action_budget(
    p: &EncounterParticipant,
    sources: &SourceConditions,
) -> ActionBudgetView {
    let plan = RegainPlan::evaluate(p, sources);
    let inactive = plan.inactive;
    let mut actions = allocation_count(&plan.evaluated_allocation());
    actions.adjusted_value = plan.total();
    let mut notes = Vec::new();
    if let Some(reason) = plan.missing_reason() {
        actions.explanation = Some(reason.into());
        notes.push(note("unresolved", "Needs review", reason));
    }
    let reason = cannot_act_reason(p, sources);
    let reaction_reason = reason.or_else(|| {
        super::causes::condition_causes(p, None, sources)
            .active
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
        extra_action_unavailable_reason: plan.extra_unavailable_reason().map(str::to_owned),
        quickened_sources: plan
            .extra_uses
            .into_iter()
            .map(super::super::projection::condition_extra_action_view)
            .collect(),
        actions,
        reactions: count("Reactions", 1, Some(i64::from(reaction_reason.is_none()))),
        can_act: capability(reason),
        can_react: capability(reaction_reason),
        notes,
    }
}
impl RegainPlan {
    fn evaluated_allocation(&self) -> TurnActionAllocation {
        let mut adjustments = vec![];
        if self.quickened {
            adjustments.push(TurnActionAdjustment::Quickened { gained: 1 });
        }
        if self.inactive {
            adjustments.push(TurnActionAdjustment::Inactive {
                lost: self.allotment,
            });
        } else {
            if self.lost_to_stun > 0 {
                adjustments.push(TurnActionAdjustment::Stunned {
                    value: (!self.duration_stun).then_some(self.stunned),
                    lost: self.lost_to_stun,
                });
            }
            if self.slowed > 0 {
                adjustments.push(TurnActionAdjustment::Slowed {
                    value: self.slowed,
                    lost: self.lost_to_slowed,
                    overlap: self.slowed.min(self.allotment).min(self.lost_to_stun),
                });
            }
        }
        TurnActionAllocation {
            base: 3,
            adjustments,
        }
    }
}

/// Presentation of recorded facts never reads today's conditions or source data.
pub(in crate::encounters) fn allocation_count(
    allocation: &TurnActionAllocation,
) -> RuntimeCountView {
    let total = allocation.base
        + allocation
            .adjustments
            .iter()
            .map(|change| match *change {
                TurnActionAdjustment::Quickened { gained } => gained,
                TurnActionAdjustment::Stunned { lost, .. }
                | TurnActionAdjustment::Slowed { lost, .. }
                | TurnActionAdjustment::Inactive { lost } => -lost,
                TurnActionAdjustment::StunnedDuringTurn { .. } => 0,
            })
            .sum::<i64>();
    let mut actions = count("Actions", allocation.base, Some(total));
    for change in &allocation.adjustments {
        match *change {
            TurnActionAdjustment::Quickened { gained } => {
                actions
                    .adjustments
                    .push(adjustment("quickened", "Quickened", gained, None));
            }
            TurnActionAdjustment::Inactive { lost } => {
                actions
                    .adjustments
                    .push(adjustment("inactive", "Inactive", -lost, None));
            }
            TurnActionAdjustment::Stunned { value, lost } => {
                let label = value
                    .map(|value| format!("Stunned {value}"))
                    .unwrap_or_else(|| "Stunned (duration)".into());
                actions
                    .adjustments
                    .push(adjustment("stunned", &label, -lost, None));
            }
            TurnActionAdjustment::Slowed {
                value,
                lost,
                overlap,
            } => {
                let label = format!("Slowed {value}");
                if lost > 0 {
                    actions.adjustments.push(adjustment(
                        "slowed",
                        &label,
                        -lost,
                        (overlap > 0).then(|| {
                            format!(
                                "{overlap} {} already lost to Stunned.",
                                if overlap == 1 { "action" } else { "actions" }
                            )
                        }),
                    ));
                } else if overlap > 0 {
                    actions.suppressed_adjustments.push(adjustment(
                        "slowed",
                        &label,
                        -overlap,
                        Some("Overlaps Stunned; no additional action lost.".into()),
                    ));
                }
            }
            TurnActionAdjustment::StunnedDuringTurn { .. } => {}
        }
    }
    actions
}

pub(in crate::encounters) fn allocation_changes(
    allocation: &TurnActionAllocation,
) -> Vec<RuntimeAdjustmentView> {
    allocation
        .adjustments
        .iter()
        .enumerate()
        .filter_map(|(index, change)| {
            if let TurnActionAdjustment::StunnedDuringTurn { value, lost } = change {
                Some(adjustment(
                    &format!("stunned-during-{index}"),
                    &value
                        .map(|value| format!("Stunned {value}"))
                        .unwrap_or_else(|| "Stunned".into()),
                    -*lost,
                    None,
                ))
            } else {
                None
            }
        })
        .collect()
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
            ability_states: vec![],
            turn_resources: None,
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
        assert_eq!(
            action_budget(&p, &Default::default()).extra_action_unavailable_reason,
            None
        );
        p.conditions.push(condition("Slowed", Some(1)));
        let budget = action_budget(&p, &Default::default());
        assert_eq!(
            budget.extra_action_unavailable_reason.as_deref(),
            Some("Lost to Slowed")
        );
        assert_eq!(budget.actions.adjusted_value, Some(3));
        assert_eq!(
            budget
                .actions
                .adjustments
                .iter()
                .map(|row| (row.source.as_str(), row.value, row.reason.as_deref()))
                .collect::<Vec<_>>(),
            [("quickened", 1, None), ("slowed", -1, None)]
        );
        p.conditions.push(condition("Stunned", Some(2)));
        let budget = action_budget(&p, &Default::default());
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
            Some("Overlaps Stunned; no additional action lost.")
        );
    }

    #[test]
    fn complete_allotment_and_overlap_are_forecast_without_consuming_conditions() {
        let mut p = participant();
        p.conditions = vec![condition("Quickened", None), condition("Slowed", Some(4))];
        assert_eq!(
            action_budget(&p, &Default::default())
                .actions
                .adjusted_value,
            Some(0)
        );
        assert!(
            action_budget(&p, &Default::default()).can_act.available,
            "Slowed is a regain loss, not immediate inability to act"
        );
        p.conditions = vec![condition("Stunned", Some(1)), condition("Slowed", Some(2))];
        let before = p.clone();
        for _ in 0..2 {
            let budget = action_budget(&p, &Default::default());
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
        assert_eq!(
            action_budget(&p, &Default::default())
                .actions
                .adjusted_value,
            None
        );
        p.conditions[0].duration = Some(ConditionDuration {
            amount: 2,
            unit: ConditionDurationUnit::Minutes,
            timing: None,
        });
        assert_eq!(
            action_budget(&p, &Default::default())
                .actions
                .adjusted_value,
            Some(0)
        );
        p.conditions[0].duration.as_mut().unwrap().amount = 0;
        assert!(
            !action_budget(&p, &Default::default()).can_act.available,
            "Expiry still requires explicit removal"
        );
        p.conditions[0].value = Some(0);
        assert!(
            action_budget(&p, &Default::default()).can_act.available,
            "Explicit numeric recovery to zero permits acting"
        );
    }
    #[test]
    fn known_zero_regain_keeps_unknown_attribution_separate() {
        let mut p = participant();
        let sources = SourceConditions::unavailable();
        let plan = RegainPlan::evaluate(&p, &sources);
        assert_eq!(plan.total(), None);
        assert!(plan.allocation().is_none());
        p.conditions = vec![condition("Stunned", None)];
        p.conditions[0].duration = Some(ConditionDuration {
            amount: 1,
            unit: ConditionDurationUnit::Minutes,
            timing: None,
        });
        let plan = RegainPlan::evaluate(&p, &sources);
        assert_eq!(plan.total(), Some(0));
        assert_eq!(plan.extra(), 0);
        assert_eq!(plan.stunned_loss(), None);
        assert!(plan.allocation().is_none());
        assert!(plan.missing_reason().is_some());
        p.conditions.clear();
        p.defeated = true;
        let plan = RegainPlan::evaluate(&p, &sources);
        assert_eq!(plan.total(), Some(0));
        assert!(plan.allocation().is_none());
    }
    #[test]
    fn inactive_status_preserves_hp_and_blocks_current_capability() {
        let mut p = participant();
        p.defeated = true;
        assert_eq!(
            action_budget(&p, &Default::default())
                .actions
                .adjusted_value,
            Some(0)
        );
        assert!(!action_budget(&p, &Default::default()).can_act.available);
        assert_eq!(
            action_budget(&p, &Default::default())
                .can_act
                .source
                .as_deref(),
            Some("participant_status")
        );
        p.defeated = false;
        p.participant_kind = ParticipantKind::Hazard;
        p.hazard_disabled = true;
        assert!(!action_budget(&p, &Default::default()).can_react.available);
        assert_eq!(
            action_budget(&p, &Default::default())
                .can_react
                .source
                .as_deref(),
            Some("participant_status")
        );
        assert_eq!(p.current_hp, Some(20));
        p.hazard_disabled = false;
        assert!(action_budget(&p, &Default::default()).can_act.available);
        p.mortality = atlas_local_state::MortalityState::Dead;
        assert_eq!(
            action_budget(&p, &Default::default())
                .can_act
                .source
                .as_deref(),
            Some("participant_status")
        );
        p.mortality = atlas_local_state::MortalityState::Alive;
        p.conditions = vec![condition("Unconscious", None)];
        assert_eq!(
            action_budget(&p, &Default::default())
                .can_act
                .source
                .as_deref(),
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
        let general = condition_causes(
            &p,
            None,
            crate::encounters::rules::source_conditions::SourceConditions::empty(),
        )
        .active;
        assert_eq!(
            general
                .iter()
                .filter(|cause| cause.rule == ConditionRule::OffGuard)
                .filter_map(|cause| cause.source.condition_id())
                .collect::<Vec<_>>(),
            vec![20]
        );
        let targeted = condition_causes(
            &p,
            Some("a"),
            crate::encounters::rules::source_conditions::SourceConditions::empty(),
        )
        .active;
        assert_eq!(
            targeted
                .iter()
                .filter(|cause| cause.rule == ConditionRule::OffGuard)
                .filter_map(|cause| cause.source.condition_id())
                .collect::<Vec<_>>(),
            vec![10, 20]
        );
        p.conditions.remove(0);
        assert_eq!(
            condition_causes(
                &p,
                Some("a"),
                crate::encounters::rules::source_conditions::SourceConditions::empty()
            )
            .active
            .iter()
            .map(|cause| (cause.rule, cause.source.condition_id()))
            .collect::<Vec<_>>(),
            [(ConditionRule::OffGuard, Some(20))]
        );
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
            condition_applicability(rule, &p, &facts, SourceConditions::empty()),
            Some(atlas_app_model::EncounterConditionEligibilityView::NeedsContext { .. })
        ));
        facts.immunities.entries[0].coverage =
            crate::encounters::rules::ImmunityCoverage::Unconditional;
        assert!(matches!(
            condition_applicability(rule, &p, &facts, SourceConditions::empty()),
            Some(atlas_app_model::EncounterConditionEligibilityView::Immune { .. })
        ));
    }
    #[test]
    fn unconscious_blocks_actions_but_paralyzed_retains_mental_actions() {
        let mut p = participant();
        p.conditions = vec![condition("Unconscious", None)];
        assert!(!action_budget(&p, &Default::default()).can_act.available);
        p.conditions = vec![condition("Paralyzed", None)];
        assert!(action_budget(&p, &Default::default()).can_act.available);
        p.conditions = vec![condition("Confused", None)];
        assert!(action_budget(&p, &Default::default()).can_act.available);
        assert!(!action_budget(&p, &Default::default()).can_react.available);
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
            KnownActionFact {
                fact: ActionFact::Manipulate,
                value: true,
            },
            KnownActionFact {
                fact: ActionFact::Auditory,
                value: true,
            },
            KnownActionFact {
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
        let result = evaluate_action(
            &p,
            crate::encounters::rules::source_conditions::SourceConditions::empty(),
            &context,
        );
        assert!(result.issues.is_empty());
        assert_eq!(
            result.disruption,
            Some(DisruptionCheck {
                dc: 7,
                source_ids: vec![1.into(), 2.into(), 3.into()]
            })
        );
        p.conditions.reverse();
        assert_eq!(
            evaluate_action(
                &p,
                crate::encounters::rules::source_conditions::SourceConditions::empty(),
                &context
            )
            .disruption,
            result.disruption
        );
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
            evaluate_action(
                &p,
                crate::encounters::rules::source_conditions::SourceConditions::empty(),
                &context
            )
            .issues,
            vec![ActionIssue::NeedsFact {
                source_id: crate::encounters::rules::causes::ConditionSourceKey::Saved(1),
                fact: ActionFact::MentalOnly
            }]
        );
        let facts = [KnownActionFact {
            fact: ActionFact::MentalOnly,
            value: true,
        }];
        let allowed = ActionContext {
            facts: &facts,
            ..context
        };
        assert!(
            evaluate_action(
                &p,
                crate::encounters::rules::source_conditions::SourceConditions::empty(),
                &allowed
            )
            .issues
            .is_empty()
        );
    }

    #[test]
    fn independent_fascination_sources_keep_independent_reminders() {
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
        let facts = [KnownActionFact {
            fact: ActionFact::Concentrate,
            value: true,
        }];
        let context = ActionContext {
            kind: ActionKind::Retch,
            reaction: Some(false),
            other_participant: None,
            facts: &facts,
        };
        assert_eq!(
            evaluate_action(
                &p,
                crate::encounters::rules::source_conditions::SourceConditions::empty(),
                &context
            )
            .issues,
            vec![1, 2]
                .into_iter()
                .map(|id| ActionIssue::NeedsFact {
                    source_id: crate::encounters::rules::causes::ConditionSourceKey::Saved(id),
                    fact: ActionFact::RelatesToFascination,
                })
                .collect::<Vec<_>>()
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
        let source_sets = Default::default();
        let causes = observer_defense_causes("observer", &roster, &source_sets);
        assert_eq!(causes.len(), 1);
        assert_eq!(causes[0].against, Some("subject"));
        assert_eq!(causes[0].rule, ConditionRule::OffGuard);
        assert!(observer_defense_causes("other", &roster, &source_sets).is_empty());
        assert!(observer_defense_causes("subject", &roster, &source_sets).is_empty());
        assert_eq!(
            targeting_requirements(
                &roster[0],
                "observer",
                crate::encounters::rules::source_conditions::SourceConditions::empty()
            )
            .flat_check_dc,
            Some(11)
        );
    }

    #[test]
    fn undetected_requires_a_guess_without_leaking_failure_reason() {
        use super::super::relationships::*;
        let mut subject = participant();
        subject.conditions = vec![condition("Undetected", None), condition("Concealed", None)];
        let direct = targeting_requirements(
            &subject,
            "observer",
            crate::encounters::rules::source_conditions::SourceConditions::empty(),
        );
        assert!(direct.requires_guessed_location);
        assert_eq!(direct.flat_check_dc, Some(11));
    }
}
