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

pub(in crate::encounters) struct ConditionResolution<'a> {
    pub active: Vec<ConditionCause<'a>>,
    pub uncertain: Vec<ConditionCause<'a>>,
    source_unavailable: bool,
}
impl ConditionResolution<'_> {
    pub fn known_presence(&self, rule: ConditionRule) -> crate::AppServiceResult<bool> {
        if self.active.iter().any(|cause| cause.rule == rule) {
            return Ok(true);
        }
        if self.source_unavailable || self.uncertain.iter().any(|cause| cause.rule == rule) {
            return Err(crate::AppServiceError::invalid_request(format!(
                "Check whether {} applies before resolving this event.",
                rule.definition().name
            )));
        }
        Ok(false)
    }
    pub fn known_value(&self, rule: ConditionRule) -> crate::AppServiceResult<i64> {
        let mut value = 0;
        for cause in self.active.iter().filter(|cause| cause.rule == rule) {
            value = value.max(cause.value.filter(|value| *value >= 0).ok_or_else(|| {
                crate::AppServiceError::invalid_request(format!(
                    "Set the {} value before resolving this event.",
                    rule.definition().name
                ))
            })?);
        }
        if self.source_unavailable
            || self.uncertain.iter().any(|cause| {
                cause.rule == rule && cause.value.is_none_or(|possible| possible > value)
            })
        {
            return Err(crate::AppServiceError::invalid_request(format!(
                "Check the {} value and source before resolving this event.",
                rule.definition().name
            )));
        }
        Ok(value)
    }
}

pub(in crate::encounters) fn condition_causes<'a>(
    participant: &'a EncounterParticipant,
    opponent: Option<&str>,
    sources: &'a SourceConditions,
) -> ConditionResolution<'a> {
    resolve_causes(participant, opponent, &[], sources)
}

pub(in crate::encounters) fn resolve_causes<'a>(
    participant: &'a EncounterParticipant,
    opponent: Option<&str>,
    additional: &[ScopedConditionCause<'a>],
    sources: &'a SourceConditions,
) -> ConditionResolution<'a> {
    let saved: Vec<_> = participant
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
    let mut base = ConditionResolution {
        active: saved,
        uncertain: vec![],
        source_unavailable: sources.unavailable_reason.is_some(),
    };
    for cause in additional
        .iter()
        .filter(|cause| cause.against.is_none() || cause.against == opponent)
    {
        admit_derived(
            &mut base,
            ConditionCause {
                source: cause.source,
                rule: cause.rule,
                value: cause.value,
            },
            false,
            sources,
        );
    }
    let mut result = expand_and_suppress(base.active.clone(), base.uncertain.clone(), sources);
    let mut prior = vec![];
    let mut history: Vec<Vec<Option<bool>>> = vec![];
    let mut cyclic = vec![false; sources.entries.len()];
    for _ in 0..sources.entries.len().saturating_mul(3).saturating_add(4) {
        let known = result
            .active
            .iter()
            .map(|cause| cause.rule)
            .collect::<Vec<_>>();
        let possible = result
            .uncertain
            .iter()
            .map(|cause| cause.rule)
            .collect::<Vec<_>>();
        let mut states = sources
            .entries
            .iter()
            .enumerate()
            .map(|(i, source)| {
                if cyclic[i] {
                    None
                } else {
                    source.active(&known, &possible, sources, participant)
                }
            })
            .collect::<Vec<_>>();
        if states == prior {
            return result;
        }
        if let Some(start) = history.iter().position(|seen| seen == &states) {
            for i in 0..states.len() {
                if history[start..].iter().any(|seen| seen[i] != states[i]) {
                    cyclic[i] = true;
                    states[i] = None;
                }
            }
            history.clear();
        }
        let mut active = base.active.clone();
        let mut uncertain = base.uncertain.clone();
        for (source, state) in sources.entries.iter().zip(&states) {
            let cause = ConditionCause {
                source: ConditionSource::Authored(
                    source,
                    sources.source_fingerprint.as_deref(),
                    &participant.participant_key,
                ),
                rule: source.rule,
                value: source.value,
            };
            match state {
                Some(true) => {
                    active.push(cause);
                    if source.rule.definition().has_value && source.value.is_none() {
                        uncertain.push(cause);
                    }
                }
                None => uncertain.push(cause),
                Some(false) => {}
            }
        }
        result = expand_and_suppress(active, uncertain, sources);
        history.push(states.clone());
        prior = states;
    }
    result
}

fn admit_derived<'a>(
    result: &mut ConditionResolution<'a>,
    cause: ConditionCause<'a>,
    uncertain: bool,
    sources: &SourceConditions,
) {
    use super::applicability::{ConditionImmunityState, condition_immunity};
    match condition_immunity(&sources.immunities, cause.rule.definition().slug) {
        ConditionImmunityState::Immune => {}
        ConditionImmunityState::Available if !uncertain => push_unique(&mut result.active, cause),
        _ => push_unique(&mut result.uncertain, cause),
    }
}
fn push_unique<'a>(causes: &mut Vec<ConditionCause<'a>>, cause: ConditionCause<'a>) {
    if !causes
        .iter()
        .any(|prior| prior.source.key() == cause.source.key() && prior.rule == cause.rule)
    {
        causes.push(cause);
    }
}
fn expand_and_suppress<'a>(
    active: Vec<ConditionCause<'a>>,
    uncertain: Vec<ConditionCause<'a>>,
    sources: &SourceConditions,
) -> ConditionResolution<'a> {
    let reachable = expand(active.clone(), uncertain.clone(), sources);
    let active = active
        .into_iter()
        .filter(|cause| {
            !reachable
                .active
                .iter()
                .any(|other| suppresses(other.rule, cause.rule))
        })
        .collect();
    let mut result = expand(active, uncertain, sources);
    let suppressors = result
        .active
        .iter()
        .map(|cause| cause.rule)
        .collect::<Vec<_>>();
    result
        .active
        .retain(|cause| !suppressors.iter().any(|rule| suppresses(*rule, cause.rule)));
    result
        .uncertain
        .retain(|cause| !suppressors.iter().any(|rule| suppresses(*rule, cause.rule)));
    result.active.sort_by_key(|cause| {
        (
            cause.source.condition_id().is_none(),
            cause.source.condition_id().unwrap_or(0),
            cause.rule,
        )
    });
    result
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

fn expand<'a>(
    active: Vec<ConditionCause<'a>>,
    uncertain: Vec<ConditionCause<'a>>,
    sources: &SourceConditions,
) -> ConditionResolution<'a> {
    let mut result = ConditionResolution {
        active,
        uncertain,
        source_unavailable: sources.unavailable_reason.is_some(),
    };
    let mut active_cursor = 0;
    let mut uncertain_cursor = 0;
    while active_cursor < result.active.len() || uncertain_cursor < result.uncertain.len() {
        let (cause, uncertain) = if active_cursor < result.active.len() {
            let cause = result.active[active_cursor];
            active_cursor += 1;
            (cause, false)
        } else {
            let cause = result.uncertain[uncertain_cursor];
            uncertain_cursor += 1;
            (cause, true)
        };
        for grant in cause
            .rule
            .definition()
            .grants
            .iter()
            .filter(|grant| maintained(cause.rule, grant))
        {
            admit_derived(
                &mut result,
                ConditionCause {
                    source: cause.source,
                    rule: grant.target,
                    value: grant.target.definition().default_value,
                },
                uncertain,
                sources,
            );
        }
    }
    result
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
