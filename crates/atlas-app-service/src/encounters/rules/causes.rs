//! Maintained grants carry their original source; scope is resolved before overrides.
use super::super::conditions::{ConditionRule, condition_rule_for_key};
use super::projection::scope_applies;
use atlas_local_state::{EncounterParticipant, EncounterParticipantCondition};

#[derive(Clone, Copy)]
pub(in crate::encounters) struct ConditionCause<'a> {
    pub source: &'a EncounterParticipantCondition,
    pub rule: ConditionRule,
    pub value: Option<i64>,
}

#[derive(Clone, Copy)]
pub(in crate::encounters) struct ScopedConditionCause<'a> {
    pub source: &'a EncounterParticipantCondition,
    pub rule: ConditionRule,
    pub value: Option<i64>,
    /// The participant against whom this cause applies; None is universal.
    pub against: Option<&'a str>,
}

pub(in crate::encounters) fn condition_causes<'a>(
    participant: &'a EncounterParticipant,
    opponent: Option<&str>,
) -> Vec<ConditionCause<'a>> {
    resolve_causes(participant, opponent, &[])
}

pub(in crate::encounters) fn resolve_causes<'a>(
    participant: &'a EncounterParticipant,
    opponent: Option<&str>,
    additional: &[ScopedConditionCause<'a>],
) -> Vec<ConditionCause<'a>> {
    let mut causes: Vec<_> = participant
        .conditions
        .iter()
        .filter(|source| scope_applies(&source.scope, opponent))
        .filter_map(|source| {
            condition_rule_for_key(source.condition_key.as_deref()).map(|rule| ConditionCause {
                source,
                rule,
                value: source.value,
            })
        })
        .collect();
    causes.extend(
        additional
            .iter()
            .filter(|cause| cause.against.is_none() || cause.against == opponent)
            .map(|cause| ConditionCause {
                source: cause.source,
                rule: cause.rule,
                value: cause.value,
            }),
    );
    let blinded = causes.iter().any(|cause| {
        matches!(
            cause.rule,
            ConditionRule::Blinded | ConditionRule::Unconscious | ConditionRule::Dying
        )
    });
    let restrained = causes
        .iter()
        .any(|cause| cause.rule == ConditionRule::Restrained);
    causes.retain(|cause| {
        !((blinded && cause.rule == ConditionRule::Dazzled)
            || (restrained && cause.rule == ConditionRule::Grabbed))
    });
    // Finite maintained-grant graph, no saved child rows or read-time mutations.
    let mut cursor = 0;
    while cursor < causes.len() {
        let cause = causes[cursor];
        let grants: &[(ConditionRule, Option<i64>)] = match cause.rule {
            ConditionRule::Grabbed | ConditionRule::Restrained => &[
                (ConditionRule::OffGuard, None),
                (ConditionRule::Immobilized, None),
            ],
            ConditionRule::Encumbered => &[(ConditionRule::Clumsy, Some(1))],
            ConditionRule::Dying => &[(ConditionRule::Unconscious, None)],
            ConditionRule::Unconscious => &[
                (ConditionRule::Blinded, None),
                (ConditionRule::OffGuard, None),
            ],
            ConditionRule::Confused | ConditionRule::Paralyzed | ConditionRule::Prone => {
                &[(ConditionRule::OffGuard, None)]
            }
            _ => &[],
        };
        for &(rule, value) in grants {
            if !causes.iter().any(|existing| {
                existing.source.condition_id == cause.source.condition_id && existing.rule == rule
            }) {
                causes.push(ConditionCause {
                    source: cause.source,
                    rule,
                    value,
                });
            }
        }
        cursor += 1;
    }
    causes.sort_by_key(|cause| (cause.source.condition_id, cause.rule));
    causes
}
