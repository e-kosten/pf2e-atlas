//! Own-turn Stunned application uses the same reducer as explicit lifecycle recovery.
use super::super::conditions::{ConditionRule, condition_rule_for_key};
use super::events::{ConditionConsequence, ConditionConsequences, subject_after_change};
use crate::AppServiceResult;
use atlas_app_model::ConditionLifecycleActionView;
use atlas_local_state::{ConditionEventChange, EncounterParticipant};

/// The caller establishes actual current-turn identity; this function performs no I/O.
pub(in crate::encounters) fn apply(
    participant: &EncounterParticipant,
    planned: &mut ConditionConsequences,
    request_id: &str,
) -> AppServiceResult<()> {
    let Some(resources) = participant
        .turn_resources
        .as_ref()
        .filter(|r| r.actions_open && r.pending_reason.is_none())
    else {
        return Ok(());
    };
    let Some(change) = planned.changes.condition_changes.first() else {
        return Ok(());
    };
    let (condition_id, key, value) = match change {
        ConditionEventChange::Add(input) => (-1, input.condition_key.as_deref(), input.value),
        ConditionEventChange::Replace(input) => {
            let prior = participant
                .conditions
                .iter()
                .find(|c| c.condition_id == input.condition_id);
            // Notes, duration and source edits do not replay an already applied loss.
            if prior
                .is_some_and(|p| p.condition_key == input.condition_key && p.value == input.value)
            {
                return Ok(());
            }
            (
                input.condition_id,
                input.condition_key.as_deref(),
                input.value,
            )
        }
        ConditionEventChange::Remove { .. } => return Ok(()),
    };
    if condition_rule_for_key(key) != Some(ConditionRule::Stunned) || !value.is_some_and(|v| v > 0)
    {
        return Ok(());
    }
    let after = subject_after_change(participant, change)?;
    let Some(lost) = super::capabilities::stunned_resolution_loss(&after).filter(|n| *n > 0) else {
        return Ok(());
    };
    let mut resolved = super::lifecycle::prepare_lifecycle(
        &after,
        &ConditionLifecycleActionView::StunnedActionsLost {
            condition_id,
            actions_lost: lost,
        },
        None,
        request_id,
    )?
    .resolve(None)?;
    for reduction in resolved.condition_changes {
        let reduced_id = match &reduction {
            ConditionEventChange::Replace(input) => input.condition_id,
            ConditionEventChange::Remove { condition_id, .. } => *condition_id,
            ConditionEventChange::Add(_) => continue,
        };
        if reduced_id != condition_id {
            planned.changes.condition_changes.push(reduction);
        } else {
            match reduction {
                ConditionEventChange::Replace(reduced) => {
                    match &mut planned.changes.condition_changes[0] {
                        ConditionEventChange::Add(input) => input.value = reduced.value,
                        ConditionEventChange::Replace(input) => input.value = reduced.value,
                        ConditionEventChange::Remove { .. } => {}
                    }
                }
                ConditionEventChange::Remove { .. } if condition_id == -1 => {
                    planned.changes.condition_changes.remove(0);
                }
                ConditionEventChange::Remove { .. } => {
                    planned.changes.condition_changes[0] = reduction
                }
                ConditionEventChange::Add(_) => {}
            }
        }
    }
    // -1 belongs only to the borrowed preview subject. A source consumed before
    // insertion has no saved row ID; its request and retained result identify it.
    for resource in &mut resolved.turn_resources {
        for effect in &mut resource.effects {
            if effect.condition_id == Some(-1) {
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
            stunned_after: after
                .conditions
                .iter()
                .filter(|c| {
                    condition_rule_for_key(c.condition_key.as_deref())
                        == Some(ConditionRule::Stunned)
                })
                .filter_map(|c| c.value)
                .max()
                .unwrap_or(0)
                .saturating_sub(lost),
        });
    Ok(())
}
