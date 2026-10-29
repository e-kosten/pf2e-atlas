//! Own-turn Stunned application uses the same reducer as explicit lifecycle recovery.
use super::super::conditions::ConditionRule;
use super::events::{ConditionConsequence, ConditionConsequences, subject_after_event};
use crate::{AppServiceError, AppServiceResult};
use atlas_local_state::{ConditionEventChange, EncounterParticipant};

pub(in crate::encounters) fn apply(
    participant: &EncounterParticipant,
    sources: &super::source_conditions::SourceConditions,
    planned: &mut ConditionConsequences,
) -> AppServiceResult<()> {
    let Some(resources) = participant
        .turn_resources
        .as_ref()
        .filter(|r| r.actions_open && r.pending_reason.is_none())
    else {
        return Ok(());
    };
    let after = subject_after_event(participant, &planned.changes)?;
    let before_causes = super::causes::condition_causes(participant, None, sources);
    let after_causes = super::causes::condition_causes(&after, None, sources);
    let same_cause = |before: &super::causes::ConditionCause<'_>,
                      after: &super::causes::ConditionCause<'_>| {
        before.rule == after.rule
            && before.source.key() == after.source.key()
            && before.value == after.value
    };
    let mut new_stunned = false;
    for cause in after_causes
        .active
        .iter()
        .filter(|cause| cause.rule == ConditionRule::Stunned)
    {
        if before_causes
            .active
            .iter()
            .any(|before| same_cause(before, cause))
        {
            continue;
        }
        if before_causes
            .uncertain
            .iter()
            .any(|before| same_cause(before, cause))
        {
            let label = match cause.source {
                super::causes::ConditionSource::Authored(source, ..) => {
                    source.source_label.as_str()
                }
                super::causes::ConditionSource::Saved(source) => source.name.as_str(),
            };
            return Err(AppServiceError::invalid_request(format!(
                "Check whether Stunned from {label} already applied before resolving action loss."
            )));
        }
        new_stunned = true;
    }
    if after_causes.uncertain.iter().any(|cause| {
        cause.rule == ConditionRule::Stunned
            && !before_causes
                .active
                .iter()
                .chain(&before_causes.uncertain)
                .any(|before| same_cause(before, cause))
    }) {
        return Err(after_causes
            .known_value(ConditionRule::Stunned)
            .err()
            .unwrap_or_else(|| {
                AppServiceError::invalid_request(
                    "Check whether the new source Stunned applies before resolving action loss.",
                )
            }));
    }
    if !new_stunned {
        return Ok(());
    }
    let lost = super::capabilities::stunned_resolution_loss(&after, sources).ok_or_else(|| {
        after_causes
            .known_value(ConditionRule::Stunned)
            .err()
            .unwrap_or_else(|| {
                AppServiceError::invalid_request(
                    "Check source conditions before resolving Stunned action loss.",
                )
            })
    })?;
    if lost <= 0 {
        return Ok(());
    }
    let mut resolved = super::lifecycle::consume_numeric_stunned(&after, lost, sources)?.changes;
    let reduced_subject =
        super::events::subject_after_changes(&after, &resolved.condition_changes)?;
    let stunned_after = super::causes::condition_causes(&reduced_subject, None, sources)
        .known_value(ConditionRule::Stunned)
        .ok();
    let mut reductions = std::collections::BTreeMap::new();
    for reduction in resolved.condition_changes {
        let id = match &reduction {
            ConditionEventChange::Replace(row) => row.condition_id,
            ConditionEventChange::Remove { condition_id, .. } => *condition_id,
            ConditionEventChange::Add(_) => continue,
        };
        reductions.insert(id, reduction);
    }
    let mut next_id = participant
        .conditions
        .iter()
        .map(|row| row.condition_id)
        .min()
        .unwrap_or(0)
        .min(0);
    planned.changes.condition_changes = std::mem::take(&mut planned.changes.condition_changes)
        .into_iter()
        .filter_map(|mut change| {
            let id = match &change {
                ConditionEventChange::Add(_) => {
                    next_id -= 1;
                    next_id
                }
                ConditionEventChange::Replace(row) => row.condition_id,
                ConditionEventChange::Remove { condition_id, .. } => *condition_id,
            };
            let Some(reduction) = reductions.remove(&id) else {
                return Some(change);
            };
            match (&mut change, reduction) {
                (ConditionEventChange::Add(row), ConditionEventChange::Replace(reduced)) => {
                    row.value = reduced.value;
                    Some(change)
                }
                (ConditionEventChange::Add(row), ConditionEventChange::Remove { .. }) => {
                    if let Some(origin) = row.origin.take() {
                        planned.changes.consumed_applications.push(
                            atlas_local_state::ConsumedConditionApplication {
                                participant_key: row.participant_key.clone(),
                                origin,
                            },
                        );
                    }
                    None
                }
                (_, reduction) => Some(reduction),
            }
        })
        .collect();
    planned
        .changes
        .condition_changes
        .extend(reductions.into_values());
    for resource in &mut resolved.turn_resources {
        for effect in &mut resource.effects {
            if effect.condition_id.is_some_and(|id| id < 0) {
                effect.condition_id = None;
            }
        }
    }
    planned
        .changes
        .turn_resources
        .extend(resolved.turn_resources);
    planned
        .effects
        .push(ConditionConsequence::StunnedActionsLost {
            before: resources.regular_remaining + resources.extra_remaining,
            after: resources.regular_remaining + resources.extra_remaining - lost,
            stunned_after,
        });
    Ok(())
}
