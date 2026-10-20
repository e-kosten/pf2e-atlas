//! Condition-specific movement and Prone cover queries, without a map or action ledger.
use super::super::{
    conditions::ConditionRule,
    stat_block::{StatBlockView, StatisticTarget},
};
use atlas_local_state::EncounterParticipant;
use serde_json::Number;

pub(in crate::encounters) struct ForcedMovementContext {
    pub holding_in_place: Option<bool>,
    pub difficulty_class: Option<i64>,
    pub check_total: Option<i64>,
}
pub(in crate::encounters) enum ForcedMovementOutcome {
    Unrestricted,
    NeedsCheck,
    CheckFailed,
    CheckSucceeded,
    NeedsContext,
}
pub(in crate::encounters) struct ForcedMovementRequirements {
    pub outcome: ForcedMovementOutcome,
    pub difficulty_class: Option<i64>,
    pub source_ids: Vec<i64>,
    pub required_facts: Vec<String>,
}
pub(in crate::encounters) fn forced_movement(
    participant: &EncounterParticipant,
    other: Option<&str>,
    context: ForcedMovementContext,
) -> ForcedMovementRequirements {
    let causes = super::causes::condition_causes(participant, other);
    let mut source_ids = causes
        .iter()
        .filter(|cause| cause.rule == ConditionRule::Immobilized)
        .map(|cause| cause.source.condition_id)
        .collect::<Vec<_>>();
    source_ids.sort_unstable();
    source_ids.dedup();
    let mut result = ForcedMovementRequirements {
        outcome: ForcedMovementOutcome::Unrestricted,
        difficulty_class: None,
        source_ids,
        required_facts: vec![],
    };
    if result.source_ids.is_empty() {
        return result;
    }
    let known_hold = causes.iter().any(|cause| {
        matches!(
            cause.rule,
            ConditionRule::Grabbed | ConditionRule::Restrained
        )
    });
    let holding = if known_hold {
        Some(true)
    } else {
        context.holding_in_place
    };
    if holding == Some(false) {
        return result;
    }
    if holding.is_none() {
        result.outcome = ForcedMovementOutcome::NeedsContext;
        result
            .required_facts
            .push("Confirm whether something holds you in place.".into());
        return result;
    }
    let Some(dc) = context.difficulty_class else {
        result.outcome = ForcedMovementOutcome::NeedsContext;
        result
            .required_facts
            .push("Choose the holding effect’s DC or relevant defense DC.".into());
        return result;
    };
    result.difficulty_class = Some(dc);
    result.outcome = match context.check_total {
        None => ForcedMovementOutcome::NeedsCheck,
        Some(total) if total >= dc => ForcedMovementOutcome::CheckSucceeded,
        Some(_) => ForcedMovementOutcome::CheckFailed,
    };
    result
}
pub(in crate::encounters) struct ProneCoverRequirements {
    pub armor_class: Option<Number>,
    pub required_facts: Vec<String>,
}
pub(in crate::encounters) fn prone_cover(
    participant: &EncounterParticipant,
    baseline: Option<(&StatBlockView, bool)>,
    other: Option<&str>,
    against_ranged: Option<bool>,
) -> ProneCoverRequirements {
    let mut result = ProneCoverRequirements {
        armor_class: None,
        required_facts: vec![],
    };
    let prone = super::causes::condition_causes(participant, other)
        .iter()
        .any(|cause| cause.rule == ConditionRule::Prone);
    if !prone {
        result
            .required_facts
            .push("Prone cover requires being Prone.".into());
        return result;
    }
    let Some(ranged) = against_ranged else {
        result
            .required_facts
            .push("Confirm whether the incoming attack is ranged.".into());
        return result;
    };
    let Some((baseline, adjustment_applicable)) = baseline else {
        result.required_facts.push("AC is unavailable.".into());
        return result;
    };
    let mut projected = if let Some(other) = other {
        super::project_participant_for_opponent(
            participant,
            baseline,
            adjustment_applicable,
            &[],
            other,
        )
    } else {
        super::project_participant(participant, baseline, adjustment_applicable, &[])
    };
    if ranged
        && let Some(value) = projected
            .values
            .iter_mut()
            .find(|value| value.target == StatisticTarget::ArmorClass)
    {
        super::projection::apply(
            value,
            super::modifiers::Modifier {
                source: "action:take-cover".into(),
                label: "Take Cover while Prone".into(),
                modifier_type: super::modifiers::ModifierType::Circumstance,
                value: 4,
            },
        );
    }
    result.armor_class = projected
        .effective_stat(&StatisticTarget::ArmorClass)
        .cloned();
    if result.armor_class.is_none() {
        result.required_facts.push("AC is unavailable.".into());
    }
    result
}
