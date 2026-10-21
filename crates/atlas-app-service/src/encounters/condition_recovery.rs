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
                ConditionLifecycleActionView::FrightenedEndTurn { .. }
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
        if let Some(saved) = &saved_boundary {
            retain_outgoing_frightened(&mut participant, saved);
        }
        let subject = if action_needs_source(&request.action, &participant) {
            workflow::condition_subject_context(self, &participant)?
        } else {
            workflow::ConditionSubjectContext::from_saved(&participant)
        };
        if let ConditionLifecycleActionView::ResolveBroken { threshold, .. } = &request.action {
            let source = subject
                .facts
                .broken_threshold
                .as_ref()
                .and_then(serde_json::Number::as_i64);
            if source.is_some_and(|source| source != *threshold) {
                return Err(AppServiceError::invalid_request(
                    "The entered Broken Threshold differs from the source",
                ));
            }
            if subject.facts.broken_threshold_known_absent
                && !participant.conditions.iter().any(|condition| {
                    super::conditions::condition_rule_for_key(condition.condition_key.as_deref())
                        == Some(ConditionRule::Broken)
                        && condition.immunity_override
                })
            {
                return Err(AppServiceError::invalid_request(
                    "This object has no Broken Threshold",
                ));
            }
        }
        let maximum = rules::effective_hp_maximum(&participant, subject.level);
        let selected_pending = pending_for_action(&pending, &request.action);
        let confusion =
            if let ConditionLifecycleActionView::Retch { confusion, .. } = &request.action {
                super::action_projection::checked_confusion(
                    &snapshot.participants,
                    &participant,
                    confusion.as_ref(),
                )?
            } else {
                None
            };
        validate_action(&participant, &request.action, confusion.as_ref())?;
        let offer = rules::lifecycle::available_actions(
            &participant,
            subject.fortitude,
            subject.perception,
            maximum,
            &pending,
            &[],
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
            maximum,
            selected_pending,
            &request.request_id,
        )?;
        let claims = lifecycle_claims(
            &participant,
            &request.action,
            saved_boundary
                .as_ref()
                .map_or(snapshot.encounter.round_number, |saved| {
                    saved.snapshot.encounter.round_number
                }),
        );
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
                    PendingConsequence::BrokenHpThreshold { participant_key } => {
                        Some(PendingConditionRecoveryView::BrokenThreshold {
                            consequence_key: saved.consequence_key,
                            participant_key,
                        })
                    }
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
        let options = |source: &workflow::ConditionSubjectContext| {
            rules::lifecycle::available_actions(
                participant,
                source.fortitude,
                source.perception,
                rules::effective_hp_maximum(participant, source.level),
                &pending,
                &claims,
            )
        };
        let mut actions = options(&workflow::ConditionSubjectContext::from_saved(participant));
        // A full-HP offer may only become available once Drained's level is known.
        let needs_source = actions
            .iter()
            .any(|option| offer_needs_source(&option.action, participant))
            || (rules::hp_events::has(participant, ConditionRule::Wounded)
                && full_hp_rest_needs_source(participant));
        if boundary.is_none() && needs_source {
            match workflow::condition_subject_context(self, participant) {
                Ok(source) => actions = options(&source),
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
                    for option in &mut actions {
                        if option.available && offer_needs_source(&option.action, participant) {
                            option.available = false;
                            option.unavailable_reason = Some("Reference data unavailable".into());
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
                matches!(
                    action.action,
                    ConditionRecoveryOfferView::FrightenedEndTurn { .. }
                )
            });
        }
        for option in &mut view.actions {
            if let ConditionRecoveryOfferView::Retch { issues, .. } = &mut option.action {
                *issues = super::retch::evaluate(participant, &[], None)?
                    .issues
                    .into_iter()
                    .map(|issue| super::action_projection::issue_view(issue, participant))
                    .collect();
            }
        }
        Ok(view)
    }
}
fn full_hp_rest_needs_source(participant: &EncounterParticipant) -> bool {
    participant.level_override.is_none()
        && rules::hp_events::has(participant, ConditionRule::Drained)
}

fn action_needs_source(
    action: &ConditionLifecycleActionView,
    participant: &EncounterParticipant,
) -> bool {
    matches!(
        action,
        ConditionLifecycleActionView::Retch { .. }
            | ConditionLifecycleActionView::WakeFromNoise { .. }
            | ConditionLifecycleActionView::ResolveBroken { .. }
    ) || (matches!(action, ConditionLifecycleActionView::FullHpRest { .. })
        && full_hp_rest_needs_source(participant))
}

fn offer_needs_source(
    offer: &ConditionRecoveryOfferView,
    participant: &EncounterParticipant,
) -> bool {
    matches!(
        offer,
        ConditionRecoveryOfferView::Retch { .. }
            | ConditionRecoveryOfferView::WakeFromNoise { .. }
            | ConditionRecoveryOfferView::ResolveBroken { .. }
    ) || (matches!(offer, ConditionRecoveryOfferView::FullHpRest)
        && full_hp_rest_needs_source(participant))
}

fn pending_for_action<'a>(
    pending: &'a [SavedPendingConsequence],
    action: &ConditionLifecycleActionView,
) -> Option<&'a SavedPendingConsequence> {
    let key = match action {
        ConditionLifecycleActionView::ConfusedDamageRecovery {
            consequence_key, ..
        }
        | ConditionLifecycleActionView::ResolveBroken {
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
        ConditionLifecycleActionView::FrightenedEndTurn { .. } => {
            (EncounterBoundaryKind::TurnEnd, "frightened")
        }
        ConditionLifecycleActionView::WakeFromNoise { .. } => {
            (EncounterBoundaryKind::TurnStart, "waking-noise")
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
        (
            A::Retch {
                dc,
                fortitude_modifier,
                check,
                ..
            },
            O::Retch { check: facts, .. },
        ) => {
            if facts.dc.is_some() && dc.is_some() && facts.dc != *dc {
                return Err(AppServiceError::invalid_request(
                    "Update this source's recovery DC before using a different DC",
                ));
            }
            (
                check,
                facts,
                *dc,
                *fortitude_modifier,
                Purpose::Retch,
                false,
                1,
            )
        }
        (
            A::WakeFromNoise {
                dc,
                perception_modifier,
                check,
            },
            O::WakeFromNoise { check: facts },
        ) => (
            check,
            facts,
            Some(*dc),
            *perception_modifier,
            Purpose::WakeFromNoise,
            false,
            0,
        ),
        (
            A::ConfusedDamageRecovery {
                qualifying_damage: true,
                check: Some(check),
                ..
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
    let modifier = choose_modifier(facts.modifier, modifier)?;
    if dc.is_some_and(|dc| dc < 1) {
        return Err(AppServiceError::invalid_request(
            "Check DC must be positive",
        ));
    }
    if !matches!(input, LifecycleCheckInputView::TableDegree { .. })
        && (dc.is_none() || modifier.is_none())
    {
        return Err(AppServiceError::invalid_request(
            "Supply the source DC and current modifier, or record the table's degree of success",
        ));
    }
    if let LifecycleCheckInputView::TableResult { natural, .. } = &input {
        if purpose != Purpose::ConfusedDamageRecovery && natural.is_none() {
            return Err(AppServiceError::invalid_request(
                "Record the natural d20 result or the table's degree of success",
            ));
        }
        if natural.is_some_and(|value| !(1..=20).contains(&value)) {
            return Err(AppServiceError::invalid_request(
                "Natural d20 result must be between 1 and 20",
            ));
        }
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
fn choose_modifier(source: Option<i64>, supplied: Option<i64>) -> AppServiceResult<Option<i64>> {
    if source.is_some() && supplied.is_some() && source != supplied {
        return Err(AppServiceError::invalid_request(
            "The entered modifier differs from the current encounter statistic",
        ));
    }
    Ok(source.or(supplied))
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
    let mut table_result = None;
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
        LifecycleCheckInputView::TableResult { total, natural } => {
            table_result = Some(total);
            calculate_degree(
                total,
                check.dc.ok_or_else(|| {
                    atlas_dice::DiceError::Invalid("Check DC is unavailable".into())
                })?,
                natural,
                check.purpose != atlas_local_state::LifecycleCheckPurpose::ConfusedDamageRecovery,
            )
        }
    };
    Ok((
        check.purpose,
        check.dc,
        check.modifier,
        roll,
        table_result,
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
    confusion: Option<&super::rules::action_context::ConfusionChoice>,
) -> AppServiceResult<()> {
    let ConditionLifecycleActionView::Retch { judgments, .. } = action else {
        return Ok(());
    };
    let evaluation = super::retch::evaluate(participant, judgments, confusion)?;
    if !evaluation.issues.is_empty() {
        return Err(AppServiceError::invalid_request(
            "Retch is prevented or needs a decision about an active condition",
        ));
    }
    if evaluation.disruption.is_some() {
        return Err(AppServiceError::invalid_request(
            "Retch unexpectedly requires an additional disruption check",
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
