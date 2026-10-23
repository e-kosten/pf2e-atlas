//! Condition-derived observer effects. Unknown detection stays a table decision.
use super::super::conditions::{ConditionRule, condition_rule_for_key};
use super::causes::ScopedConditionCause;
use super::projection::scope_applies;
use atlas_local_state::EncounterParticipant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::encounters) enum DetectionState {
    Hidden,
    Undetected,
}
/// Invisible alone does not establish how a particular observer detects the subject.
pub(in crate::encounters) fn detection_state(
    subject: &EncounterParticipant,
    observer: &str,
) -> Option<DetectionState> {
    let causes = super::causes::condition_causes(subject, Some(observer));
    if causes
        .iter()
        .any(|cause| cause.rule == ConditionRule::Invisible)
    {
        return None;
    }
    if causes.iter().any(|cause| {
        matches!(
            cause.rule,
            ConditionRule::Undetected | ConditionRule::Unnoticed
        )
    }) {
        Some(DetectionState::Undetected)
    } else if causes
        .iter()
        .any(|cause| cause.rule == ConditionRule::Hidden)
    {
        Some(DetectionState::Hidden)
    } else {
        None
    }
}
pub(in crate::encounters) fn observer_defense_causes<'a>(
    observer_key: &str,
    participants: &'a [EncounterParticipant],
) -> Vec<ScopedConditionCause<'a>> {
    participants
        .iter()
        .filter(|subject| subject.participant_key != observer_key)
        .filter(|subject| {
            matches!(
                detection_state(subject, observer_key),
                Some(DetectionState::Hidden | DetectionState::Undetected)
            )
        })
        .flat_map(|subject| {
            subject.conditions.iter().filter_map(move |source| {
                if !scope_applies(&source.scope, Some(observer_key)) {
                    return None;
                }
                matches!(
                    condition_rule_for_key(source.condition_key.as_deref()),
                    Some(
                        ConditionRule::Hidden
                            | ConditionRule::Undetected
                            | ConditionRule::Unnoticed
                    )
                )
                .then_some(ScopedConditionCause {
                    source,
                    rule: ConditionRule::OffGuard,
                    value: None,
                    against: Some(subject.participant_key.as_str()),
                })
            })
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::encounters) struct TargetingRequirements {
    pub flat_check_dc: Option<i64>,
    pub source_ids: Vec<i64>,
    pub requires_guessed_location: bool,
}

pub(in crate::encounters) fn targeting_requirements(
    subject: &EncounterParticipant,
    observer_key: &str,
) -> TargetingRequirements {
    let mut result = TargetingRequirements {
        flat_check_dc: None,
        source_ids: vec![],
        requires_guessed_location: false,
    };
    for source in &subject.conditions {
        if !scope_applies(&source.scope, Some(observer_key)) {
            continue;
        }
        let rule = condition_rule_for_key(source.condition_key.as_deref());
        let dc = match rule {
            Some(ConditionRule::Concealed) => 5,
            Some(ConditionRule::Hidden) => 11,
            Some(ConditionRule::Undetected | ConditionRule::Unnoticed) => {
                result.requires_guessed_location = true;
                11
            }
            _ => continue,
        };
        result.flat_check_dc = Some(result.flat_check_dc.unwrap_or(0).max(dc));
        result.source_ids.push(source.condition_id);
    }
    result.source_ids.sort_unstable();
    result
}
