//! Bounded PF2e 8.6.1 assistance: current permission and a read-only regain forecast.
use super::conditions::{ConditionRule, condition_rule_for_key};
use atlas_app_model::{
    ActionBudgetView, RuntimeAdjustmentView, RuntimeCapabilityView, RuntimeCountView,
    RuntimeEffectNoteView,
};
use atlas_local_state::EncounterParticipant;

pub(super) fn cannot_act_reason(p: &EncounterParticipant) -> Option<&'static str> {
    if p.defeated {
        return Some("Marked defeated.");
    }
    if p.hazard_disabled {
        return Some("Hazard disabled.");
    }
    p.conditions
        .iter()
        .any(|c| {
            condition_rule_for_key(c.condition_key.as_deref()) == Some(ConditionRule::Stunned)
                && c.value != Some(0)
        })
        .then_some("Stunned.")
}

pub(super) fn action_budget(p: &EncounterParticipant) -> ActionBudgetView {
    let mut slowed = 0;
    let mut stunned = 0;
    let mut quickened = false;
    let mut duration_stun = false;
    let mut unknown = false;
    for c in &p.conditions {
        match condition_rule_for_key(c.condition_key.as_deref()) {
            Some(ConditionRule::Quickened) => quickened = true,
            Some(ConditionRule::Slowed) => match c.value {
                Some(v) if v >= 0 => slowed = slowed.max(v),
                _ => unknown = true,
            },
            Some(ConditionRule::Stunned) => match c.value {
                Some(v) if v >= 0 => stunned = stunned.max(v),
                None if c.duration.is_some() => duration_stun = true,
                _ => unknown = true,
            },
            _ => {}
        }
    }
    let allotment = 3 + i64::from(quickened);
    let inactive = p.defeated || p.hazard_disabled;
    let lost_to_stun = if duration_stun {
        allotment
    } else {
        stunned.min(allotment)
    };
    let lost_to_slowed = (slowed.min(allotment) - lost_to_stun).max(0);
    let mut actions = count(
        "Next regain",
        3,
        if inactive || duration_stun {
            Some(0)
        } else if unknown {
            None
        } else {
            Some(allotment - lost_to_stun - lost_to_slowed)
        },
    );
    actions.explanation = Some("Actions regained at your next turn. Spending and Stunned recovery remain manual. PF2e 8.6.1.".into());
    let mut notes = Vec::new();
    if quickened {
        actions.adjustments.push(adjustment(
            "quickened",
            "Quickened",
            1,
            "At most one restricted extra action. Choose which actions to lose; any retained extra action keeps its restriction.",
        ));
    }
    if inactive {
        actions.adjustments.push(adjustment(
            "inactive",
            "Inactive",
            -allotment,
            "Reactivate explicitly before participating.",
        ));
    } else {
        if lost_to_stun > 0 {
            actions.adjustments.push(adjustment(
                "stunned",
                "Stunned",
                -lost_to_stun,
                if duration_stun {
                    "Duration Stunned prevents all actions until explicitly removed."
                } else {
                    "Forecast only; edit the condition to record recovery."
                },
            ));
        }
        if lost_to_slowed > 0 {
            actions.adjustments.push(adjustment(
                "slowed",
                "Slowed",
                -lost_to_slowed,
                "Only the loss beyond Stunned applies.",
            ));
        }
        let overlap = slowed.min(allotment).min(lost_to_stun);
        if overlap > 0 {
            actions.suppressed_adjustments.push(adjustment(
                "slowed",
                "Overlapping Slowed loss",
                -overlap,
                "Already counted in the Stunned loss.",
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
    let capability = || RuntimeCapabilityView {
        available: reason.is_none(),
        source: reason.map(|_| "encounter_state".into()),
        reason: reason.map(str::to_owned),
    };
    ActionBudgetView {
        actions,
        reactions: count("Reaction allowance", 1, Some(i64::from(reason.is_none()))),
        can_act: capability(),
        can_react: capability(),
        notes,
    }
}
fn count(label: &str, base_value: i64, adjusted_value: Option<i64>) -> RuntimeCountView {
    RuntimeCountView {
        label: label.into(),
        explanation: Some(
            "Allowance under the tracked conditions; reactions spent are not tracked.".into(),
        ),
        base_value,
        adjusted_value,
        segments: vec![],
        adjustments: vec![],
        suppressed_adjustments: vec![],
    }
}
fn adjustment(source: &str, label: &str, value: i64, reason: &str) -> RuntimeAdjustmentView {
    RuntimeAdjustmentView {
        source: source.into(),
        label: label.into(),
        value,
        reason: Some(reason.into()),
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
        let catalog = super::super::conditions::condition_catalog();
        let definition = catalog.conditions.iter().find(|c| c.name == name).unwrap();
        EncounterParticipantCondition {
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
            assert_eq!(budget.actions.suppressed_adjustments[0].value, -1);
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
        p.defeated = false;
        p.participant_kind = ParticipantKind::Hazard;
        p.hazard_disabled = true;
        assert!(!action_budget(&p).can_react.available);
        assert_eq!(p.current_hp, Some(20));
        p.hazard_disabled = false;
        assert!(action_budget(&p).can_act.available);
    }
}
