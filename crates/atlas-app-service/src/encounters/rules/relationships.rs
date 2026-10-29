//! Condition-derived observer effects. Unknown detection stays a table decision.
use super::super::conditions::ConditionRule;
use super::causes::ScopedConditionCause;
use super::source_conditions::SourceConditions;
use atlas_local_state::EncounterParticipant;
use std::collections::BTreeMap;

pub(in crate::encounters) fn observer_defense_causes<'a>(
    observer_key: &str,
    participants: &'a [EncounterParticipant],
    sources: &'a BTreeMap<String, SourceConditions>,
) -> Vec<ScopedConditionCause<'a>> {
    participants
        .iter()
        .filter(|subject| subject.participant_key != observer_key)
        .flat_map(|subject| {
            let source = sources
                .get(&subject.participant_key)
                .unwrap_or(SourceConditions::empty());
            let conditions = super::causes::condition_causes(subject, Some(observer_key), source);
            conditions
                .active
                .into_iter()
                .map(|cause| (cause, false))
                .chain(conditions.uncertain.into_iter().map(|cause| (cause, true)))
                .filter(|(cause, _)| {
                    matches!(
                        cause.rule,
                        ConditionRule::Hidden
                            | ConditionRule::Undetected
                            | ConditionRule::Unnoticed
                    )
                })
                .map(move |(cause, uncertain)| ScopedConditionCause {
                    source: cause.source,
                    rule: ConditionRule::OffGuard,
                    value: None,
                    against: Some(subject.participant_key.as_str()),
                    uncertain,
                })
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::encounters) struct TargetingRequirements {
    pub flat_check_dc: Option<i64>,
    pub source_ids: Vec<super::causes::ConditionSourceKey>,
    pub needs_context: bool,
    pub requires_guessed_location: bool,
}

pub(in crate::encounters) fn targeting_requirements(
    subject: &EncounterParticipant,
    observer_key: &str,
    sources: &SourceConditions,
) -> TargetingRequirements {
    let conditions = super::causes::condition_causes(subject, Some(observer_key), sources);
    let mut result = TargetingRequirements {
        flat_check_dc: None,
        source_ids: vec![],
        requires_guessed_location: false,
        needs_context: false,
    };
    for cause in &conditions.active {
        let dc = match cause.rule {
            ConditionRule::Concealed => 5,
            ConditionRule::Hidden => 11,
            ConditionRule::Undetected | ConditionRule::Unnoticed => {
                result.requires_guessed_location = true;
                11
            }
            _ => continue,
        };
        result.flat_check_dc = Some(result.flat_check_dc.unwrap_or(0).max(dc));
        result.source_ids.push(cause.source.key());
    }
    for (rule, possible_dc) in [
        (ConditionRule::Concealed, 5),
        (ConditionRule::Hidden, 11),
        (ConditionRule::Undetected, 11),
        (ConditionRule::Unnoticed, 11),
    ] {
        if conditions.known_presence(rule).is_err() {
            result.needs_context = true;
            if result.flat_check_dc.is_none_or(|known| possible_dc > known) {
                result.flat_check_dc = None;
            }
        }
    }
    result
        .source_ids
        .sort_by_key(super::causes::ConditionSourceKey::key);
    result.source_ids.dedup();
    result
}
