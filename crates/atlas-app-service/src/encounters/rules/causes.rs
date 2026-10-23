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
    // Reachability establishes derived suppressors without erasing independent sources.
    let reachable = expand(causes.clone());
    causes.retain(|cause| {
        !reachable
            .iter()
            .any(|other| suppresses(other.rule, cause.rule))
    });
    let mut causes = expand(causes);
    let active = causes.clone();
    causes.retain(|cause| {
        !active
            .iter()
            .any(|other| suppresses(other.rule, cause.rule))
    });
    causes.sort_by_key(|cause| (cause.source.condition_id, cause.rule));
    causes
}

fn suppresses(rule: ConditionRule, other: ConditionRule) -> bool {
    // These source overrides have explicit action-budget or table-adjudication policy.
    !matches!(
        rule,
        ConditionRule::Stunned
            | ConditionRule::Helpful
            | ConditionRule::Friendly
            | ConditionRule::Indifferent
            | ConditionRule::Unfriendly
            | ConditionRule::Hostile
    ) && rule.definition().overrides.contains(&other)
}

fn maintained(
    rule: ConditionRule,
    grant: &atlas_foundry_model::generated::condition_definitions::ConditionGrant,
) -> bool {
    grant.in_memory_only == Some(true)
        || matches!(
            (rule, grant.target),
            (ConditionRule::Dying, ConditionRule::Unconscious)
                | (ConditionRule::Unconscious, ConditionRule::Blinded)
        )
}

pub(in crate::encounters) fn application_grants(
    rule: ConditionRule,
    target: ConditionRule,
) -> bool {
    rule.definition()
        .grants
        .iter()
        .any(|grant| grant.target == target && grant.in_memory_only != Some(true))
}

fn expand(mut causes: Vec<ConditionCause<'_>>) -> Vec<ConditionCause<'_>> {
    let mut cursor = 0;
    while cursor < causes.len() {
        let cause = causes[cursor];
        for grant in cause
            .rule
            .definition()
            .grants
            .iter()
            .filter(|grant| maintained(cause.rule, grant))
        {
            if !causes.iter().any(|existing| {
                existing.source.condition_id == cause.source.condition_id
                    && existing.rule == grant.target
            }) {
                causes.push(ConditionCause {
                    source: cause.source,
                    rule: grant.target,
                    value: grant.target.definition().default_value,
                });
            }
        }
        cursor += 1;
    }
    causes
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_foundry_model::generated::condition_definitions::CONDITION_DEFINITIONS;
    #[test]
    fn every_persistent_grant_has_an_explicit_atlas_lifetime() {
        let mut count = 0;
        for definition in CONDITION_DEFINITIONS {
            for grant in definition
                .grants
                .iter()
                .filter(|grant| grant.in_memory_only != Some(true))
            {
                assert!(
                    maintained(definition.id, grant)
                        || (definition.id == ConditionRule::Unconscious
                            && grant.target == ConditionRule::Prone)
                );
                count += 1;
            }
        }
        assert_eq!(count, 3);
    }
}
