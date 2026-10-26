//! Resource allocation and explicit manual spending share one authoritative turn.
use super::{
    conditions::{ConditionRule, condition_rule_for_key},
    event_execution, rules,
};
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::*;
use atlas_local_state::{
    EncounterBoundaryClaim, EncounterBoundaryKind, EncounterParticipant, ParticipantKind,
    PreparedConditionEvent, TurnResourceEffect, TurnResources,
};

pub(super) fn prepare_start(
    participant: &EncounterParticipant,
    round: i64,
    id: &str,
    dying_completed: bool,
    sources: &rules::source_conditions::SourceConditions,
) -> AppServiceResult<(Option<PreparedConditionEvent>, Vec<EncounterBoundaryClaim>)> {
    if participant.participant_kind == ParticipantKind::Hazard {
        return Ok((None, vec![]));
    }
    let budget = rules::capabilities::action_budget(participant, sources);
    let dying = participant.track_dying
        && participant.current_hp == Some(0)
        && participant.conditions.iter().any(|c| {
            condition_rule_for_key(c.condition_key.as_deref()) == Some(ConditionRule::Dying)
                && c.value.is_some_and(|v| v > 0)
        });
    let pending_reason = if dying && !dying_completed {
        Some("Resolve Dying recovery before regaining actions.".into())
    } else if budget.actions.adjusted_value.is_none() {
        Some(
            budget
                .notes
                .first()
                .map(|note| note.reason.clone())
                .unwrap_or_else(|| {
                    "Set the missing condition values before regaining actions.".into()
                }),
        )
    } else {
        None
    };
    let mut changes = PreparedConditionEvent::default();
    let mut claims = vec![];
    let mut effects = vec![];
    if pending_reason.is_none() {
        let lost =
            rules::capabilities::numeric_stunned_actions_lost(participant, sources).unwrap_or(0);
        if let Some(stunned) = participant.conditions.iter().find(|c| {
            condition_rule_for_key(c.condition_key.as_deref()) == Some(ConditionRule::Stunned)
                && c.value.is_some_and(|v| v > 0)
        }) && lost > 0
        {
            let action = ConditionLifecycleActionView::StunnedActionsLost {
                condition_id: stunned.condition_id,
                actions_lost: lost,
            };
            let mut at_regain = participant.clone();
            at_regain.turn_resources = None;
            changes = rules::lifecycle::prepare_lifecycle(&at_regain, &action, None, id, sources)?
                .resolve(None)?;
            for condition in participant.conditions.iter().filter(|c| {
                condition_rule_for_key(c.condition_key.as_deref()) == Some(ConditionRule::Stunned)
            }) {
                if let Some(before) = condition.value {
                    effects.push(TurnResourceEffect {
                        condition_id: Some(condition.condition_id),
                        name: condition.name.clone(),
                        before: Some(before),
                        after: Some((before - lost).max(0)),
                        actions_lost: lost.min(before),
                    });
                }
            }
            claims.push(EncounterBoundaryClaim {
                participant_key: participant.participant_key.clone(),
                round_number: round,
                kind: EncounterBoundaryKind::TurnStart,
                scope_key: "stunned-actions".into(),
            });
        }
    }
    let total = if pending_reason.is_some() {
        0
    } else {
        budget.actions.adjusted_value.unwrap_or(0)
    };
    let extra = i64::from(
        total > 0
            && !budget.quickened_sources.is_empty()
            && budget.extra_action_unavailable_reason.is_none(),
    );
    // Permission to react is separate from owning an unspent reaction. A
    // condition ending later in the turn must not manufacture or erase stock.
    let (reactions_total, reactions_remaining) = if pending_reason.is_none() {
        (Some(1), Some(1))
    } else {
        participant
            .turn_resources
            .as_ref()
            .map(|r| (r.reactions_total, r.reactions_remaining))
            .unwrap_or((None, None))
    };
    changes.turn_resources.push(TurnResources {
        allocation: pending_reason
            .is_none()
            .then(|| rules::capabilities::action_allocation(participant, sources))
            .flatten(),
        actions_open: pending_reason.is_none(),
        participant_key: participant.participant_key.clone(),
        turn_key: format!("turn:{id}"),
        round_number: round,
        pending_reason,
        regular_total: total - extra,
        regular_remaining: total - extra,
        extra_total: extra,
        extra_remaining: extra,
        extra_uses: participant
            .conditions
            .iter()
            .filter(|c| {
                condition_rule_for_key(c.condition_key.as_deref()) == Some(ConditionRule::Quickened)
            })
            .map(|c| c.extra_action.clone())
            .chain(
                sources
                    .active_entries(participant)
                    .into_iter()
                    .filter(|c| c.rule == ConditionRule::Quickened)
                    .map(|_| atlas_local_state::ConditionExtraAction::Unspecified),
            )
            .collect(),
        reactions_total,
        reactions_remaining,
        effects,
    });
    Ok((Some(changes), claims))
}

pub(super) fn view(
    participant: &EncounterParticipant,
    current: Option<&str>,
    sources: &rules::source_conditions::SourceConditions,
) -> Option<TurnResourcesView> {
    if participant.participant_kind == ParticipantKind::Hazard {
        return None;
    }
    let budget = rules::capabilities::action_budget(participant, sources);
    let saved = participant.turn_resources.as_ref();
    let ready = saved.filter(|r| r.pending_reason.is_none());
    let mut action_permission = budget.can_act.clone();
    if current != Some(participant.participant_key.as_str()) {
        action_permission.available = false;
        action_permission.reason = Some("This creature's turn has not started.".into());
    }
    if ready.is_some_and(|r| !r.actions_open) {
        action_permission.available = false;
        action_permission.reason = Some("This turn has ended.".into());
    }
    if ready.is_none() {
        action_permission.available = false;
        action_permission.reason = Some("Regain actions to start tracking this turn.".into());
    }
    Some(TurnResourcesView {
        action_calculation: if current == Some(participant.participant_key.as_str())
            && ready.is_some_and(|r| r.actions_open)
        {
            TurnActionCalculationView::ThisTurn {
                count: ready
                    .and_then(|r| r.allocation.as_ref())
                    .map(rules::capabilities::allocation_count),
                changes: ready
                    .and_then(|r| r.allocation.as_ref())
                    .map(rules::capabilities::allocation_changes)
                    .unwrap_or_default(),
            }
        } else {
            TurnActionCalculationView::NextTurn {
                count: budget.actions.clone(),
            }
        },
        inherent_conditions: participant.record_key.as_ref().map(|key| {
            InherentActionConditionsView {
                unavailable_reason: sources.unavailable_reason.clone(),
                entries: sources
                    .active_entries(participant)
                    .into_iter()
                    .chain(sources.unresolved_entries(participant))
                    .map(|condition| InherentActionConditionView {
                        unavailable_reason: condition.activation_unavailable.clone().or_else(
                            || {
                                if sources.unresolved_entries(participant).contains(&condition) {
                                    Some("Activation needs context; check the source rule.".into())
                                } else if condition.rule.definition().has_value
                                    && condition.value.is_none()
                                {
                                    Some("Source condition value is unavailable.".into())
                                } else {
                                    None
                                }
                            },
                        ),
                        name: condition.rule.definition().name.into(),
                        has_value: condition.rule.definition().has_value,
                        value: condition.value,
                        condition_ref: condition.rule.definition().record_key.into(),
                        source_name: condition.source_label.clone(),
                        source: RecordNavigationView {
                            record_key: key.clone(),
                            owners: vec![condition.owner.clone()],
                            field: None,
                            passage: None,
                            spell_rank: None,
                            source_fingerprint: sources.source_fingerprint.clone(),
                        },
                    })
                    .collect(),
            }
        }),
        turn_key: saved.map(|r| r.turn_key.clone()),
        round_number: saved.map(|r| r.round_number),
        regular_total: ready.map(|r| r.regular_total),
        regular_remaining: ready.map(|r| r.regular_remaining),
        extra_total: ready.map(|r| r.extra_total),
        extra_remaining: ready.map(|r| r.extra_remaining),
        extra_uses: saved
            .map(|r| {
                r.extra_uses
                    .iter()
                    .cloned()
                    .map(super::projection::condition_extra_action_view)
                    .collect()
            })
            .unwrap_or_default(),
        reactions_total: saved.and_then(|r| r.reactions_total),
        reactions_remaining: saved.and_then(|r| r.reactions_remaining),
        start_results: saved
            .map(|r| {
                r.effects
                    .iter()
                    .map(|e| {
                        // Stunned can overlap Slowed at regain. Describe the
                        // condition transition, not an extra deduction from the
                        // available actions already displayed by the chips.
                        let outcome = if e.after == Some(0) {
                            format!("{} cleared", e.before.unwrap_or(0))
                        } else {
                            format!("{} → {}", e.before.unwrap_or(0), e.after.unwrap_or(0),)
                        };
                        // Both effect producers are Stunned reducers. Preserve
                        // that rule identity after the saved condition is gone;
                        // never recover identity by parsing its display name.
                        TurnResourceResultView {
                            condition_name: e.name.clone(),
                            condition_ref: ConditionRule::Stunned.definition().record_key.into(),
                            outcome,
                        }
                    })
                    .collect()
            })
            .unwrap_or_default(),
        pending_reason: saved.and_then(|r| r.pending_reason.clone()),
        forecast: budget.actions,
        can_edit_actions: current == Some(participant.participant_key.as_str())
            && ready.is_some_and(|r| r.actions_open),
        can_spend_actions: action_permission,
        can_spend_reactions: budget.can_react,
    })
}

impl AtlasAppService {
    pub fn update_turn_resources(
        &self,
        encounter_ref: &str,
        request: UpdateTurnResourcesRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let intent = serde_json::json!({"action":"turn_resources","request":request});
        if let Some(receipt) =
            event_execution::recover(self, encounter_ref, &request.request_id, &intent)?
        {
            return Ok(EncounterMutationView {
                encounter_key: receipt.encounter_key,
            });
        }
        let snapshot =
            event_execution::snapshot(self, encounter_ref, request.expected_encounter_revision)?;
        let participant = snapshot
            .participants
            .iter()
            .find(|p| p.participant_key == request.participant_key)
            .ok_or_else(|| AppServiceError::invalid_request("participant no longer exists"))?;
        if participant.participant_kind == ParticipantKind::Hazard {
            return Err(AppServiceError::invalid_request(
                "Hazards use their authored routine.",
            ));
        }
        if participant
            .turn_resources
            .as_ref()
            .map(|r| r.turn_key.as_str())
            != request.turn_key.as_deref()
        {
            return Err(AppServiceError::invalid_request(
                "The turn changed. Refresh before editing its resources.",
            ));
        }
        let mut claims = vec![];
        let (source_conditions, reference_revision) =
            if matches!(request.action, TurnResourceAction::Regain) {
                source_action_context(self, participant)?
            } else {
                (Default::default(), None)
            };
        let changes = match request.action {
            TurnResourceAction::Regain => {
                if snapshot.encounter.current_turn_participant_key.as_deref()
                    != Some(&participant.participant_key)
                {
                    return Err(AppServiceError::invalid_request(
                        "Start this creature's turn before regaining actions.",
                    ));
                }
                if participant
                    .turn_resources
                    .as_ref()
                    .is_some_and(|r| r.pending_reason.is_none())
                {
                    return Err(AppServiceError::invalid_request(
                        "This turn's resources were already regained.",
                    ));
                }
                let dying_completed = self.with_state(|session| {
                    Ok(session.local_state().encounters().boundary_claimed(
                        &snapshot.encounter.encounter_key,
                        &EncounterBoundaryClaim {
                            participant_key: participant.participant_key.clone(),
                            round_number: snapshot.encounter.round_number,
                            kind: EncounterBoundaryKind::TurnStart,
                            scope_key: "dying".into(),
                        },
                    )?)
                })?;
                let (changes, start_claims) = prepare_start(
                    participant,
                    snapshot.encounter.round_number,
                    &request.request_id,
                    dying_completed,
                    &source_conditions,
                )?;
                claims = start_claims;
                if let Some(reason) = changes
                    .as_ref()
                    .and_then(|event| event.turn_resources.first())
                    .and_then(|r| r.pending_reason.clone())
                {
                    return Err(AppServiceError::invalid_request(reason));
                }
                changes.ok_or_else(|| {
                    AppServiceError::invalid_request("No creature allocation is available.")
                })?
            }
            TurnResourceAction::Spend { resource } | TurnResourceAction::Restore { resource } => {
                let restore = matches!(request.action, TurnResourceAction::Restore { .. });
                let mut saved = editable_resources(
                    participant,
                    snapshot.encounter.current_turn_participant_key.as_deref(),
                    &resource,
                )?;
                let projection = view(
                    participant,
                    snapshot.encounter.current_turn_participant_key.as_deref(),
                    &source_conditions,
                )
                .ok_or_else(|| AppServiceError::invalid_request("Resources unavailable."))?;
                let permission = if resource == TurnResourceKind::Reaction {
                    &projection.can_spend_reactions
                } else {
                    &projection.can_spend_actions
                };
                if !restore && !permission.available {
                    return Err(AppServiceError::invalid_request(
                        permission
                            .reason
                            .clone()
                            .unwrap_or_else(|| "This resource cannot be spent.".into()),
                    ));
                }
                let (remaining, total) = resource_slot(&mut saved, &resource)?;
                let next = *remaining + if restore { 1 } else { -1 };
                if !(0..=total).contains(&next) {
                    return Err(AppServiceError::invalid_request(
                        "The resource is already at its limit.",
                    ));
                }
                *remaining = next;
                PreparedConditionEvent {
                    turn_resources: vec![saved],
                    ..Default::default()
                }
            }
            TurnResourceAction::Correct {
                resource,
                remaining,
            } => {
                let mut saved = editable_resources(
                    participant,
                    snapshot.encounter.current_turn_participant_key.as_deref(),
                    &resource,
                )?;
                let (target, total) = resource_slot(&mut saved, &resource)?;
                if !(0..=total).contains(&remaining) {
                    return Err(AppServiceError::invalid_request(
                        "Remaining resources must fit this turn's allowance.",
                    ));
                }
                *target = remaining;
                PreparedConditionEvent {
                    turn_resources: vec![saved],
                    ..Default::default()
                }
            }
        };
        let receipt = event_execution::EncounterEvent {
            encounter_key: &snapshot.encounter.encounter_key,
            request_id: &request.request_id,
            intent: &intent,
            expected_revision: request.expected_encounter_revision,
            reference_revision,
            participant_revision: Some((&participant.participant_key, participant.revision)),
        }
        .evaluate(
            self,
            atlas_local_state::ConditionEventBoundary {
                claims: &claims,
                outgoing_boundary_key: None,
            },
            || Ok(changes),
        )?;
        Ok(EncounterMutationView {
            encounter_key: receipt.encounter_key,
        })
    }
}

pub(super) fn source_action_context(
    service: &AtlasAppService,
    participant: &EncounterParticipant,
) -> AppServiceResult<(rules::source_conditions::SourceConditions, Option<u64>)> {
    if participant.record_key.is_none() || participant.participant_kind == ParticipantKind::Hazard {
        return Ok((Default::default(), None));
    }
    match super::workflow::condition_subject_context(service, participant) {
        Ok(subject) => Ok((subject.source_conditions, subject.reference_revision)),
        Err(error) => {
            let error = error.into_app_error();
            if matches!(
                error.code,
                AppErrorCode::SetupRequired
                    | AppErrorCode::ArtifactIncompatible
                    | AppErrorCode::ArtifactGenerationChanged
                    | AppErrorCode::IndexUnavailable
                    | AppErrorCode::ArtifactNotReady
            ) {
                Ok((
                    rules::source_conditions::SourceConditions::unavailable(),
                    None,
                ))
            } else {
                Err(error.into())
            }
        }
    }
}

fn editable_resources(
    participant: &EncounterParticipant,
    current: Option<&str>,
    resource: &TurnResourceKind,
) -> AppServiceResult<TurnResources> {
    let saved = participant.turn_resources.clone().ok_or_else(|| {
        AppServiceError::invalid_request("Regain actions before tracking resources.")
    })?;
    if *resource != TurnResourceKind::Reaction
        && (saved.pending_reason.is_some()
            || !saved.actions_open
            || current != Some(participant.participant_key.as_str()))
    {
        return Err(AppServiceError::invalid_request(
            "Actions can only be edited during this creature's active turn.",
        ));
    }
    Ok(saved)
}
fn resource_slot<'a>(
    saved: &'a mut TurnResources,
    resource: &TurnResourceKind,
) -> AppServiceResult<(&'a mut i64, i64)> {
    Ok(match resource {
        TurnResourceKind::Action => (&mut saved.regular_remaining, saved.regular_total),
        TurnResourceKind::ExtraAction => (&mut saved.extra_remaining, saved.extra_total),
        TurnResourceKind::Reaction => (
            saved.reactions_remaining.as_mut().ok_or_else(|| {
                AppServiceError::invalid_request("Regain resources before tracking a reaction.")
            })?,
            saved.reactions_total.ok_or_else(|| {
                AppServiceError::invalid_request("Reaction allowance is unknown.")
            })?,
        ),
    })
}
