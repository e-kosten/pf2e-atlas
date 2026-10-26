//! Maintained grants carry their original source; scope is resolved before overrides.
use super::super::conditions::{ConditionRule, condition_rule_for_key};
use super::projection::scope_applies;
use super::source_conditions::{SourceCondition, SourceConditions};
use atlas_local_state::{EncounterParticipant, EncounterParticipantCondition};

/// Authored causes never pretend to be editable saved condition rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::encounters) enum ConditionSourceKey {
    Saved(i64),
    Authored {
        participant_key: String,
        record_key: String,
        source_fingerprint: Option<String>,
        owner: atlas_domain::OwnedContentLocator,
        ordinal: Option<usize>,
        name: String,
    },
}
impl ConditionSourceKey {
    pub fn label(&self, participants: &[EncounterParticipant]) -> String {
        match self {
            Self::Saved(id) => participants
                .iter()
                .flat_map(|p| &p.conditions)
                .find(|c| c.condition_id == *id)
                .map(|c| c.name.clone())
                .unwrap_or_default(),
            Self::Authored { name, .. } => name.clone(),
        }
    }
    pub fn key(&self) -> String {
        match self {
            Self::Saved(id) => format!("condition:{id}"),
            _ => format!("source-condition:{self:?}"),
        }
    }
    pub fn saved_id(&self) -> Option<i64> {
        match self {
            Self::Saved(id) => Some(*id),
            _ => None,
        }
    }
}
impl From<i64> for ConditionSourceKey {
    fn from(id: i64) -> Self {
        Self::Saved(id)
    }
}

#[derive(Clone, Copy)]
pub(in crate::encounters) enum ConditionSource<'a> {
    Saved(&'a EncounterParticipantCondition),
    Authored(&'a SourceCondition, Option<&'a str>, &'a str),
}
impl<'a> ConditionSource<'a> {
    pub fn saved(self) -> Option<&'a EncounterParticipantCondition> {
        match self {
            Self::Saved(source) => Some(source),
            _ => None,
        }
    }
    pub fn condition_id(self) -> Option<i64> {
        self.saved().map(|source| source.condition_id)
    }
    pub fn name(self) -> &'a str {
        match self {
            Self::Saved(source) => &source.name,
            Self::Authored(source, ..) => source.rule.definition().name,
        }
    }
    pub fn key(self) -> ConditionSourceKey {
        match self {
            Self::Saved(source) => ConditionSourceKey::Saved(source.condition_id),
            Self::Authored(source, fingerprint, participant_key) => ConditionSourceKey::Authored {
                participant_key: participant_key.to_owned(),
                record_key: source.record_key.clone(),
                source_fingerprint: fingerprint.map(str::to_owned),
                owner: source.owner.clone(),
                ordinal: source.grant_ordinal,
                name: source.rule.definition().name.into(),
            },
        }
    }
    pub fn explanation_key(self) -> String {
        match self {
            Self::Saved(source) => format!("condition:{}", source.condition_id),
            Self::Authored(..) => self.key().key(),
        }
    }
    pub fn has_duration(self) -> bool {
        self.saved().is_some_and(|source| source.duration.is_some())
    }
}

#[derive(Clone, Copy)]
pub(in crate::encounters) struct ConditionCause<'a> {
    pub source: ConditionSource<'a>,
    pub rule: ConditionRule,
    pub value: Option<i64>,
}

#[derive(Clone, Copy)]
pub(in crate::encounters) struct ScopedConditionCause<'a> {
    pub source: ConditionSource<'a>,
    pub rule: ConditionRule,
    pub value: Option<i64>,
    /// The participant against whom this cause applies; None is universal.
    pub against: Option<&'a str>,
}

pub(in crate::encounters) fn condition_causes<'a>(
    participant: &'a EncounterParticipant,
    opponent: Option<&str>,
    sources: &'a SourceConditions,
) -> Vec<ConditionCause<'a>> {
    resolve_causes(participant, opponent, &[], sources)
}

pub(in crate::encounters) fn resolve_causes<'a>(
    participant: &'a EncounterParticipant,
    opponent: Option<&str>,
    additional: &[ScopedConditionCause<'a>],
    sources: &'a SourceConditions,
) -> Vec<ConditionCause<'a>> {
    let mut causes: Vec<_> = participant
        .conditions
        .iter()
        .filter(|source| scope_applies(&source.scope, opponent))
        .filter_map(|source| {
            condition_rule_for_key(source.condition_key.as_deref()).map(|rule| ConditionCause {
                source: ConditionSource::Saved(source),
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
    // Evaluate bounded predicates against effective known conditions. Iterate only to
    // a stable state; circular activation is left unresolved rather than guessed.
    let saved = causes.clone();
    let mut prior = vec![];
    let mut seen = vec![];
    for _ in 0..=sources.entries.len() + 1 {
        let rules = suppress_and_expand(causes.clone())
            .iter()
            .map(|cause| cause.rule)
            .collect::<Vec<_>>();
        let active = sources
            .entries
            .iter()
            .enumerate()
            .filter(|(_, source)| source.active(&rules, sources, participant) == Some(true))
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        if active == prior {
            break;
        }
        if seen.contains(&active) {
            causes = saved.clone();
            break;
        }
        seen.push(active.clone());
        causes = saved.clone();
        causes.extend(active.iter().map(|index| {
            let source = &sources.entries[*index];
            ConditionCause {
                source: ConditionSource::Authored(
                    source,
                    sources.source_fingerprint.as_deref(),
                    &participant.participant_key,
                ),
                rule: source.rule,
                value: source.value,
            }
        }));
        prior = active;
    }
    suppress_and_expand(causes)
}

fn suppress_and_expand(mut causes: Vec<ConditionCause<'_>>) -> Vec<ConditionCause<'_>> {
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

    causes.sort_by_key(|cause| {
        (
            cause.source.condition_id().is_none(),
            cause.source.condition_id().unwrap_or(0),
            cause.rule,
        )
    });
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
                existing.source.key() == cause.source.key() && existing.rule == grant.target
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
