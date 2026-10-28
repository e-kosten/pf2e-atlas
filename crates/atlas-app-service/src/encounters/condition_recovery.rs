//! Lifecycle commands prepare source facts once and save check outcomes with consequences.
use super::conditions::ConditionRule;
use super::event_execution::EncounterEvent;
use super::{rules, workflow};
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::*;
use atlas_local_state::{
    CheckDegree, ConditionEventOutcome, EncounterBoundaryClaim, EncounterBoundaryKind,
    EncounterEventReceipt, EncounterParticipant, PendingConsequence, SavedPendingConsequence,
};

struct PreparedCheck {
    input: LifecycleCheckInputView,
    dc: Option<i64>,
    modifier: Option<i64>,
    purpose: atlas_local_state::LifecycleCheckPurpose,
    flat: bool,
    table_actions: u8,
}
impl AtlasAppService {
    pub fn apply_condition_lifecycle(
        &self,
        encounter_ref: &str,
        request: ApplyConditionLifecycleRequest,
    ) -> AppServiceResult<ConditionLifecycleResultView> {
        let intent = serde_json::json!({"action":"condition_lifecycle","request":request});
        let (snapshot, pending) = self.with_state(|session| {
            let store = session.local_state();
            let encounter = store
                .encounters()
                .get_with_participants(encounter_ref)?
                .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
            if let Some(receipt) = store.encounters().condition_event_receipt(
                &encounter.encounter.encounter_key,
                &request.request_id,
                &intent,
            )? {
                return Ok((Err(receipt), vec![]));
            }
            let pending = store
                .encounters()
                .pending_condition_consequences(&encounter.encounter.encounter_key)?;
            Ok((Ok(encounter), pending))
        })?;
        let snapshot = match snapshot {
            Err(receipt) => return Ok(receipt_view(receipt)),
            Ok(snapshot) => snapshot,
        };
        if snapshot.encounter.revision != request.expected_encounter_revision {
            return Err(atlas_local_state::LocalStateError::StaleEncounterRevision {
                encounter_key: snapshot.encounter.encounter_key,
                expected_revision: request.expected_encounter_revision,
                actual_revision: snapshot.encounter.revision,
            }
            .into());
        }
        let saved_boundary = self.with_state(|session| {
            super::outgoing_turns::boundary(
                &session.local_state().encounters(),
                &snapshot.encounter.encounter_key,
                request.boundary_key.as_deref(),
                &request.participant_key,
            )
        })?;
        if saved_boundary.is_some()
            && !matches!(
                request.action,
                ConditionLifecycleActionView::FrightenedEndTurn
            )
        {
            return Err(AppServiceError::invalid_request(
                "Only outgoing end-turn recovery belongs to this reminder",
            ));
        }
        let mut participant = snapshot
            .participants
            .iter()
            .find(|p| p.participant_key == request.participant_key)
            .ok_or_else(|| AppServiceError::invalid_request("participant no longer exists"))?
            .clone();
        if matches!(
            request.action,
            ConditionLifecycleActionView::StunnedActionsLost { .. }
        ) && participant.turn_resources.as_ref().is_some_and(|r| {
            r.pending_reason.is_some()
                || !r.actions_open
                || snapshot.encounter.current_turn_participant_key.as_deref()
                    != Some(&participant.participant_key)
        }) {
            return Err(AppServiceError::invalid_request(
                "Resolve Stunned during this creature's action regain or active turn.",
            ));
        }
        if let Some(saved) = &saved_boundary {
            retain_outgoing_frightened(&mut participant, saved);
        }
        let subject = if action_needs_source(&request.action)
            || (matches!(
                request.action,
                ConditionLifecycleActionView::StunnedActionsLost { .. }
            ) && participant
                .turn_resources
                .as_ref()
                .is_none_or(|r| r.pending_reason.is_some() || !r.actions_open))
        {
            workflow::condition_subject_context(self, &participant)?
        } else {
            workflow::ConditionSubjectContext::from_saved(&participant)
        };
        let selected_pending = pending_for_action(&pending, &request.action);
        validate_action(&participant, &request.action, &subject.source_conditions)?;
        let offer = rules::lifecycle::available_actions(
            &participant,
            subject.fortitude,
            &pending,
            &[],
            &subject.source_conditions,
        )
        .into_iter()
        .find(|offer| rules::lifecycle::matches_action(&offer.action, &request.action))
        .ok_or_else(|| {
            AppServiceError::invalid_request("Recovery action is no longer available")
        })?;
        let check = prepare_check(&request.action, &offer.action)?;
        // Validate source decisions and prepare consequences once, before any RNG.
        let planned = rules::lifecycle::prepare_lifecycle(
            &participant,
            &request.action,
            selected_pending,
            &request.request_id,
            &subject.source_conditions,
        )?;
        let claims = if matches!(
            request.action,
            ConditionLifecycleActionView::StunnedActionsLost { .. }
        ) && participant
            .turn_resources
            .as_ref()
            .is_some_and(|r| r.pending_reason.is_none())
        {
            vec![]
        } else {
            lifecycle_claims(
                &participant,
                &request.action,
                saved_boundary
                    .as_ref()
                    .map_or(snapshot.encounter.round_number, |saved| {
                        saved.snapshot.encounter.round_number
                    }),
            )
        };
        let receipt = EncounterEvent {
            encounter_key: &snapshot.encounter.encounter_key,
            request_id: &request.request_id,
            intent: &intent,
            expected_revision: request.expected_encounter_revision,
            reference_revision: subject.reference_revision,
            participant_revision: None,
        }
        .evaluate(
            self,
            atlas_local_state::ConditionEventBoundary {
                claims: &claims,
                outgoing_boundary_key: request.boundary_key.as_deref(),
            },
            || {
                let outcome = check.map(resolve_check).transpose()?;
                let degree = outcome.as_ref().map(|outcome| degree_view(outcome.5));
                let mut changes = planned.resolve(degree).map_err(|error| {
                    atlas_local_state::LocalStateError::InvalidConditionEvent(
                        error.into_app_error().message,
                    )
                })?;
                if let Some((purpose, dc, modifier, roll, table_result, degree, table_actions)) =
                    outcome
                {
                    changes
                        .outcomes
                        .push(ConditionEventOutcome::LifecycleCheck {
                            participant_key: participant.participant_key.clone(),
                            purpose,
                            dc,
                            modifier,
                            roll,
                            table_result,
                            degree,
                            table_actions,
                        });
                }
                Ok(changes)
            },
        )?;
        Ok(receipt_view(receipt))
    }
    pub fn pending_condition_recoveries(
        &self,
        encounter_ref: &str,
    ) -> AppServiceResult<Vec<PendingConditionRecoveryView>> {
        self.with_state(|session| {
            let store = session.local_state();
            let encounter = store
                .encounters()
                .get(encounter_ref)?
                .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
            Ok(store
                .encounters()
                .pending_condition_consequences(&encounter.encounter_key)?
                .into_iter()
                .filter_map(|saved| match saved.consequence {
                    PendingConsequence::ConfusedDamageRecovery {
                        participant_key,
                        condition_ids,
                        qualifying_damage,
                    } => Some(PendingConditionRecoveryView::ConfusedDamage {
                        consequence_key: saved.consequence_key,
                        participant_key,
                        condition_ids,
                        qualifying_damage,
                    }),
                    _ => None,
                })
                .collect())
        })
    }
    pub fn condition_lifecycle_actions(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        boundary_key: Option<&str>,
    ) -> AppServiceResult<ConditionLifecycleActionsView> {
        let (snapshot, pending, claims, boundary) = self.with_state(|session| {
            let store = session.local_state();
            let snapshot = store
                .encounters()
                .get_with_participants(encounter_ref)?
                .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
            let pending = store
                .encounters()
                .pending_condition_consequences(&snapshot.encounter.encounter_key)?;
            let boundary = super::outgoing_turns::boundary(
                &store.encounters(),
                &snapshot.encounter.encounter_key,
                boundary_key,
                participant_key,
            )?;
            let claims = store.encounters().boundary_claims(
                &snapshot.encounter.encounter_key,
                participant_key,
                boundary
                    .as_ref()
                    .map_or(snapshot.encounter.round_number, |saved| {
                        saved.snapshot.encounter.round_number
                    }),
            )?;
            Ok((snapshot, pending, claims, boundary))
        })?;
        let participant = snapshot
            .participants
            .iter()
            .find(|p| p.participant_key == participant_key)
            .ok_or_else(|| AppServiceError::invalid_request("participant no longer exists"))?;
        let mut participant = participant.clone();
        if let Some(saved) = &boundary {
            retain_outgoing_frightened(&mut participant, saved);
        }
        let participant = &participant;
        let effective_claims: Vec<_> = claims
            .iter()
            .filter(|claim| {
                !(claim.scope_key == "stunned-actions"
                    && participant
                        .turn_resources
                        .as_ref()
                        .is_some_and(|r| r.pending_reason.is_none() && r.actions_open))
            })
            .cloned()
            .collect();
        let options = |source: &workflow::ConditionSubjectContext| {
            let mut options = rules::lifecycle::available_actions(
                participant,
                source.fortitude,
                &pending,
                &effective_claims,
                &source.source_conditions,
            );
            for option in &mut options {
                if matches!(
                    option.action,
                    ConditionRecoveryOfferView::StunnedActionsLost { .. }
                ) && participant.turn_resources.as_ref().is_some_and(|r| {
                    r.pending_reason.is_some()
                        || !r.actions_open
                        || snapshot.encounter.current_turn_participant_key.as_deref()
                            != Some(&participant.participant_key)
                }) {
                    option.available = false;
                    option.unavailable_reason = Some(
                        "Resolve Stunned during this creature's action regain or active turn."
                            .into(),
                    );
                }
            }
            options
        };
        let mut action_sources = rules::source_conditions::SourceConditions::default();
        let mut actions = options(&workflow::ConditionSubjectContext::from_saved(participant));
        let needs_source = actions.iter().any(|option| {
            offer_needs_source(&option.action)
                || (matches!(
                    option.action,
                    ConditionRecoveryOfferView::StunnedActionsLost { .. }
                ) && participant
                    .turn_resources
                    .as_ref()
                    .is_none_or(|r| r.pending_reason.is_some() || !r.actions_open))
        });
        if boundary.is_none() && needs_source {
            match workflow::condition_subject_context(self, participant) {
                Ok(source) => {
                    actions = options(&source);
                    action_sources = source.source_conditions;
                }
                Err(error) => {
                    let error = error.into_app_error();
                    if !matches!(
                        error.code,
                        AppErrorCode::SetupRequired
                            | AppErrorCode::ArtifactIncompatible
                            | AppErrorCode::ArtifactGenerationChanged
                            | AppErrorCode::IndexUnavailable
                            | AppErrorCode::ArtifactNotReady
                    ) {
                        return Err(error.into());
                    }
                    // Other hints and table outcomes remain usable. An unrecorded
                    // action allowance cannot be guessed for Stunned resolution.
                    for option in &mut actions {
                        if let ConditionRecoveryOfferView::StunnedActionsLost {
                            actions_lost, ..
                        } = &mut option.action
                            && participant
                                .turn_resources
                                .as_ref()
                                .is_none_or(|r| r.pending_reason.is_some() || !r.actions_open)
                        {
                            *actions_lost = None;
                            option.available = false;
                            option.unavailable_reason = Some("This creature's action allotment is unavailable. Refresh reference data before resolving Stunned.".into());
                        }
                    }
                }
            }
        }
        let mut view = ConditionLifecycleActionsView {
            encounter_key: snapshot.encounter.encounter_key,
            encounter_revision: snapshot.encounter.revision,
            participant_key: participant_key.into(),
            actions,
        };
        if boundary.is_some() {
            view.actions.retain(|action| {
                matches!(action.action, ConditionRecoveryOfferView::FrightenedEndTurn)
            });
        }
        for option in &mut view.actions {
            if let ConditionRecoveryOfferView::Retch { issues, .. } = &mut option.action {
                *issues = super::retch::evaluate(participant, &action_sources)
                    .issues
                    .into_iter()
                    .map(|issue| {
                        super::action_projection::issue_view(issue, participant, &action_sources)
                    })
                    .collect();
            }
        }
        Ok(view)
    }
}
fn action_needs_source(action: &ConditionLifecycleActionView) -> bool {
    matches!(
        action,
        ConditionLifecycleActionView::Retch {
            check: LifecycleCheckInputView::Roll,
            ..
        }
    )
}
fn offer_needs_source(offer: &ConditionRecoveryOfferView) -> bool {
    matches!(offer, ConditionRecoveryOfferView::Retch { .. })
}

fn pending_for_action<'a>(
    pending: &'a [SavedPendingConsequence],
    action: &ConditionLifecycleActionView,
) -> Option<&'a SavedPendingConsequence> {
    let key = match action {
        ConditionLifecycleActionView::ConfusedDamageRecovery {
            consequence_key, ..
        } => consequence_key,
        _ => return None,
    };
    pending.iter().find(|saved| &saved.consequence_key == key)
}
fn lifecycle_claims(
    participant: &EncounterParticipant,
    action: &ConditionLifecycleActionView,
    round: i64,
) -> Vec<EncounterBoundaryClaim> {
    let (kind, scope) = match action {
        ConditionLifecycleActionView::DyingRecovery { .. } => {
            (EncounterBoundaryKind::TurnStart, "dying")
        }
        ConditionLifecycleActionView::StunnedActionsLost { .. } => {
            (EncounterBoundaryKind::TurnStart, "stunned-actions")
        }
        ConditionLifecycleActionView::FrightenedEndTurn => {
            (EncounterBoundaryKind::TurnEnd, "frightened")
        }
        _ => return vec![],
    };
    vec![EncounterBoundaryClaim {
        participant_key: participant.participant_key.clone(),
        round_number: round,
        kind,
        scope_key: scope.into(),
    }]
}
fn prepare_check(
    action: &ConditionLifecycleActionView,
    offer: &ConditionRecoveryOfferView,
) -> AppServiceResult<Option<PreparedCheck>> {
    use ConditionLifecycleActionView as A;
    use ConditionRecoveryOfferView as O;
    use atlas_local_state::LifecycleCheckPurpose as Purpose;
    let (input, facts, dc, modifier, purpose, flat, table_actions) = match (action, offer) {
        (A::DyingRecovery { check }, O::DyingRecovery { check: facts }) => {
            (check, facts, None, None, Purpose::DyingRecovery, true, 0)
        }
        (A::Retch { check, .. }, O::Retch { check: facts, .. }) => {
            (check, facts, None, None, Purpose::Retch, false, 1)
        }
        (
            A::ConfusedDamageRecovery {
                check: Some(check), ..
            },
            O::ConfusedDamageRecovery { check: facts, .. },
        ) => (
            check,
            facts,
            None,
            None,
            Purpose::ConfusedDamageRecovery,
            true,
            0,
        ),
        _ => return Ok(None),
    };
    let input = input.clone();
    let dc = facts.dc.or(dc);
    let modifier = facts.modifier.or(modifier);
    if dc.is_some_and(|dc| dc < 1) {
        return Err(AppServiceError::invalid_request(
            "Check DC must be positive",
        ));
    }
    if !matches!(input, LifecycleCheckInputView::TableDegree { .. })
        && (dc.is_none() || modifier.is_none())
    {
        return Err(AppServiceError::invalid_request(
            "Roll requires a known recovery DC and current modifier; table outcomes need neither",
        ));
    }
    Ok(Some(PreparedCheck {
        input,
        dc,
        modifier,
        purpose,
        flat,
        table_actions,
    }))
}
type ResolvedCheck = (
    atlas_local_state::LifecycleCheckPurpose,
    Option<i64>,
    Option<i64>,
    Option<atlas_dice::RollOutcome>,
    Option<i64>,
    CheckDegree,
    u8,
);
fn resolve_check(check: PreparedCheck) -> Result<ResolvedCheck, atlas_dice::DiceError> {
    let mut roll = None;
    let degree = match check.input {
        LifecycleCheckInputView::TableDegree { degree } => degree_local(degree),
        LifecycleCheckInputView::Roll => {
            let dc = check
                .dc
                .ok_or_else(|| atlas_dice::DiceError::Invalid("Check DC is unavailable".into()))?;
            let modifier = check.modifier.ok_or_else(|| {
                atlas_dice::DiceError::Invalid("Check modifier is unavailable".into())
            })?;
            if check.flat && dc >= 21 {
                CheckDegree::Failure
            } else if check.flat && dc <= 1 {
                CheckDegree::Success
            } else {
                let outcome = atlas_dice::prepare(&format!("1d20{modifier:+}"))?.roll()?;
                let natural = natural_d20(&outcome.expression);
                let degree = calculate_degree(
                    i64::from(outcome.total),
                    dc,
                    natural,
                    check.purpose
                        != atlas_local_state::LifecycleCheckPurpose::ConfusedDamageRecovery,
                );
                roll = Some(outcome);
                degree
            }
        }
    };
    Ok((
        check.purpose,
        check.dc,
        check.modifier,
        roll,
        None,
        degree,
        check.table_actions,
    ))
}
fn natural_d20(expression: &atlas_dice::RollExpression) -> Option<i64> {
    match expression {
        atlas_dice::RollExpression::Dice { dice, .. } => dice
            .iter()
            .find(|die| die.kept && die.sides == 20)
            .map(|die| i64::from(die.value)),
        atlas_dice::RollExpression::Binary { left, right, .. } => {
            natural_d20(left).or_else(|| natural_d20(right))
        }
        atlas_dice::RollExpression::Negate { inner, .. } => natural_d20(inner),
        _ => None,
    }
}
fn calculate_degree(
    total: i64,
    dc: i64,
    natural: Option<i64>,
    critical_degrees: bool,
) -> CheckDegree {
    if !critical_degrees {
        return if total >= dc {
            CheckDegree::Success
        } else {
            CheckDegree::Failure
        };
    }
    let total = i128::from(total);
    let dc = i128::from(dc);
    let base = if total >= dc + 10 {
        3
    } else if total >= dc {
        2
    } else if total <= dc - 10 {
        0
    } else {
        1
    };
    let adjusted = (base
        + match natural {
            Some(20) => 1,
            Some(1) => -1,
            _ => 0,
        })
    .clamp(0, 3);
    match adjusted {
        0 => CheckDegree::CriticalFailure,
        1 => CheckDegree::Failure,
        2 => CheckDegree::Success,
        _ => CheckDegree::CriticalSuccess,
    }
}
fn degree_local(value: CheckDegreeView) -> CheckDegree {
    match value {
        CheckDegreeView::CriticalFailure => CheckDegree::CriticalFailure,
        CheckDegreeView::Failure => CheckDegree::Failure,
        CheckDegreeView::Success => CheckDegree::Success,
        CheckDegreeView::CriticalSuccess => CheckDegree::CriticalSuccess,
    }
}
fn degree_view(value: CheckDegree) -> CheckDegreeView {
    match value {
        CheckDegree::CriticalFailure => CheckDegreeView::CriticalFailure,
        CheckDegree::Failure => CheckDegreeView::Failure,
        CheckDegree::Success => CheckDegreeView::Success,
        CheckDegree::CriticalSuccess => CheckDegreeView::CriticalSuccess,
    }
}
fn receipt_view(receipt: EncounterEventReceipt) -> ConditionLifecycleResultView {
    let check = receipt
        .outcomes
        .into_iter()
        .find_map(|outcome| match outcome {
            ConditionEventOutcome::LifecycleCheck {
                dc,
                modifier,
                roll,
                table_result,
                degree,
                ..
            } => Some(LifecycleCheckResultView {
                dc,
                modifier,
                roll,
                table_result,
                degree: degree_view(degree),
            }),
            _ => None,
        });
    ConditionLifecycleResultView {
        encounter_key: receipt.encounter_key,
        request_id: receipt.request_id,
        check,
    }
}
fn validate_action(
    participant: &EncounterParticipant,
    action: &ConditionLifecycleActionView,
    sources: &rules::source_conditions::SourceConditions,
) -> AppServiceResult<()> {
    let ConditionLifecycleActionView::Retch {
        check: LifecycleCheckInputView::Roll,
        ..
    } = action
    else {
        // An explicit table outcome records the GM's resolution; it does not ask
        // Atlas to authorize or replay the action that produced it.
        return Ok(());
    };
    let evaluation = super::retch::evaluate(participant, sources);
    if !evaluation.issues.is_empty() || evaluation.disruption.is_some() {
        return Err(AppServiceError::invalid_request(
            "Resolve the Retch restrictions at the table, then record its outcome",
        ));
    }
    Ok(())
}

fn retain_outgoing_frightened(
    participant: &mut EncounterParticipant,
    saved: &atlas_local_state::SavedOutgoingTurn,
) {
    let due = saved
        .snapshot
        .participants
        .iter()
        .find(|p| p.participant_key == participant.participant_key);
    participant.conditions.retain(|condition| {
        super::conditions::condition_rule_for_key(condition.condition_key.as_deref())
            != Some(ConditionRule::Frightened)
            || due.is_some_and(|due| {
                due.conditions
                    .iter()
                    .any(|old| old.condition_id == condition.condition_id)
            })
    });
}
