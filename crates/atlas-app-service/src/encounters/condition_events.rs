//! Condition event preparation and durable commit. No source I/O follows a write.
use super::conditions::{ConditionRule, condition_rule_for_key};
use super::rules::events::{ConditionConsequences, plan_condition_change};
use super::{timing, workflow};
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::*;
use atlas_local_state::{
    ConditionEventChange, ConditionOrigin, ConditionParameters, EncounterWithParticipants,
    PendingConsequence,
};

struct Prepared {
    encounter_key: String,
    request_id: String,
    intent: serde_json::Value,
    expected_revision: i64,
    reference_revision: Option<u64>,
    consequences: ConditionConsequences,
    prone_prevented: Option<(String, String)>,
}
impl AtlasAppService {
    pub fn set_encounter_life_settings(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: SetEncounterLifeSettingsRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let intent = serde_json::json!({"action":"life_settings","participant_key":participant_key,"request":request});
        if let Some(receipt) = recover(self, encounter_ref, &request.request_id, &intent)? {
            return Ok(receipt);
        }
        let snapshot = snapshot(self, encounter_ref, request.expected_encounter_revision)?;
        let participant = snapshot
            .participants
            .iter()
            .find(|p| p.participant_key == participant_key)
            .ok_or_else(|| AppServiceError::invalid_request("participant no longer exists"))?;
        if participant.participant_kind == atlas_local_state::ParticipantKind::Hazard
            && request.track_dying
        {
            return Err(AppServiceError::invalid_request(
                "hazards do not track dying",
            ));
        }
        let mut update = atlas_local_state::ParticipantEventChange::preserving(participant);
        update.track_dying = request.track_dying;
        if let Some(mortality) = request.mortality {
            update.mortality = match mortality {
                MortalityStateView::Alive => atlas_local_state::MortalityState::Alive,
                MortalityStateView::Dead => atlas_local_state::MortalityState::Dead,
            };
        }
        if let Some(level) = request.level_override {
            if level.value.is_some_and(|value| {
                !(-9_007_199_254_740_991..=9_007_199_254_740_991).contains(&value)
            }) {
                return Err(AppServiceError::invalid_request(
                    "Level must be a safe integer",
                ));
            }
            update.level_override = level.value;
        }
        if let Some(threshold) = request.base_death_threshold {
            if !(1..=9_007_199_254_740_991).contains(&threshold) {
                return Err(AppServiceError::invalid_request(
                    "Death threshold must be a positive safe integer",
                ));
            }
            update.base_death_threshold = threshold;
        }

        self.with_write(|session| {
            session.local_state().encounters().apply_condition_event(
                &snapshot.encounter.encounter_key,
                &request.request_id,
                &intent,
                request.expected_encounter_revision,
                atlas_local_state::PreparedConditionEvent {
                    participant_changes: vec![update],
                    ..Default::default()
                },
            )?;
            Ok(EncounterMutationView {
                encounter_key: snapshot.encounter.encounter_key,
            })
        })
    }
    pub(super) fn commit_condition_add(
        &self,
        encounter_ref: &str,
        request: AddEncounterParticipantConditionRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let intent = serde_json::json!({"action":"add_condition","request":request});
        if let Some(receipt) = recover(self, encounter_ref, &request.request_id, &intent)? {
            return Ok(receipt);
        }
        let prepared = prepare_add(self, encounter_ref, request, intent)?;
        commit(self, prepared)
    }
    pub fn preview_encounter_condition_add(
        &self,
        encounter_ref: &str,
        request: AddEncounterParticipantConditionRequest,
    ) -> AppServiceResult<ConditionEventPreviewView> {
        let intent = serde_json::json!({"action":"add_condition","request":request});
        Ok(preview(prepare_add(self, encounter_ref, request, intent)?))
    }
    pub(super) fn commit_condition_update(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: UpdateEncounterParticipantConditionRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let intent = serde_json::json!({"action":"update_condition","participant_key":participant_key,"request":request});
        if let Some(receipt) = recover(self, encounter_ref, &request.request_id, &intent)? {
            return Ok(receipt);
        }
        commit(
            self,
            prepare_update(self, encounter_ref, participant_key, request, intent)?,
        )
    }
    pub fn preview_encounter_condition_update(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: UpdateEncounterParticipantConditionRequest,
    ) -> AppServiceResult<ConditionEventPreviewView> {
        let intent = serde_json::json!({"action":"update_condition","participant_key":participant_key,"request":request});
        Ok(preview(prepare_update(
            self,
            encounter_ref,
            participant_key,
            request,
            intent,
        )?))
    }
    pub(super) fn commit_condition_remove(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        condition_id: i64,
        request: RemoveEncounterParticipantConditionRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let intent = serde_json::json!({"action":"remove_condition","participant_key":participant_key,"condition_id":condition_id,"request":request});
        if let Some(receipt) = recover(self, encounter_ref, &request.request_id, &intent)? {
            return Ok(receipt);
        }
        let snapshot = snapshot(self, encounter_ref, request.expected_encounter_revision)?;
        workflow::condition_in_snapshot(&snapshot, participant_key, condition_id)?;
        let participant = snapshot
            .participants
            .iter()
            .find(|p| p.participant_key == participant_key)
            .ok_or_else(|| AppServiceError::invalid_request("participant no longer exists"))?;
        let consequences = plan_condition_change(
            participant,
            ConditionEventChange::Remove {
                participant_key: participant_key.into(),
                condition_id,
            },
            request.intent == ConditionRemovalIntentView::RemoveEffect,
            true,
            None,
            &request.request_id,
        )?;
        commit(
            self,
            Prepared {
                encounter_key: snapshot.encounter.encounter_key,
                request_id: request.request_id,
                intent,
                expected_revision: request.expected_encounter_revision,
                reference_revision: None,
                consequences,
                prone_prevented: None,
            },
        )
    }
    pub fn pending_encounter_condition_consequences(
        &self,
        encounter_ref: &str,
    ) -> AppServiceResult<Vec<PendingConditionConsequenceView>> {
        self.with_state(|session| {
            let encounter = session
                .local_state()
                .encounters()
                .get(encounter_ref)?
                .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
            Ok(session
                .local_state()
                .encounters()
                .pending_condition_consequences(&encounter.encounter_key)?
                .into_iter()
                .filter_map(|pending| match pending.consequence {
                    PendingConsequence::DrainedHpLoss {
                        participant_key,
                        magnitude_increase,
                    } => Some(PendingConditionConsequenceView {
                        consequence_key: pending.consequence_key,
                        participant_key,
                        magnitude_increase,
                    }),
                    _ => None,
                })
                .collect())
        })
    }
    pub fn resolve_encounter_drained_consequence(
        &self,
        encounter_ref: &str,
        request: ResolveDrainedConsequenceRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let intent = serde_json::json!({"action":"resolve_drained","request":request});
        if let Some(receipt) = recover(self, encounter_ref, &request.request_id, &intent)? {
            return Ok(receipt);
        }
        let snapshot = snapshot(self, encounter_ref, request.expected_encounter_revision)?;
        let pending = self
            .with_state(|session| {
                Ok(session
                    .local_state()
                    .encounters()
                    .pending_condition_consequences(&snapshot.encounter.encounter_key)?)
            })?
            .into_iter()
            .find(|p| p.consequence_key == request.consequence_key)
            .ok_or_else(|| {
                AppServiceError::invalid_request("pending consequence no longer exists")
            })?;
        let PendingConsequence::DrainedHpLoss {
            participant_key,
            magnitude_increase,
        } = pending.consequence
        else {
            return Err(AppServiceError::invalid_request(
                "selected consequence is not Drained HP loss",
            ));
        };
        let participant = snapshot
            .participants
            .iter()
            .find(|p| p.participant_key == participant_key)
            .ok_or_else(|| AppServiceError::invalid_request("participant no longer exists"))?;
        let amount = request
            .level
            .max(1)
            .checked_mul(magnitude_increase)
            .ok_or_else(|| {
                AppServiceError::invalid_request("Drained HP loss exceeds the supported range")
            })?;
        let mut changes = super::rules::hp_events::plan_hp_event(
            participant,
            &HpEventInputView::DirectLoss { amount },
            &Default::default(),
            None,
            &request.request_id,
        )?;
        if let Some(update) = changes.participant_changes.first_mut() {
            update.level_override = Some(request.level);
        }
        changes.resolved_pending_keys.push(request.consequence_key);
        self.with_write(|session| {
            session.local_state().encounters().apply_condition_event(
                &snapshot.encounter.encounter_key,
                &request.request_id,
                &intent,
                request.expected_encounter_revision,
                changes,
            )?;
            Ok(EncounterMutationView {
                encounter_key: snapshot.encounter.encounter_key,
            })
        })
    }
}
fn recover(
    service: &AtlasAppService,
    encounter_ref: &str,
    request_id: &str,
    intent: &serde_json::Value,
) -> AppServiceResult<Option<EncounterMutationView>> {
    service.with_state(|session| {
        let encounter = session
            .local_state()
            .encounters()
            .get(encounter_ref)?
            .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
        Ok(session
            .local_state()
            .encounters()
            .condition_event_receipt(&encounter.encounter_key, request_id, intent)?
            .map(|r| EncounterMutationView {
                encounter_key: r.encounter_key,
            }))
    })
}
fn snapshot(
    service: &AtlasAppService,
    encounter_ref: &str,
    expected: i64,
) -> AppServiceResult<EncounterWithParticipants> {
    service.with_state(|session| {
        let snapshot = session
            .local_state()
            .encounters()
            .get_with_participants(encounter_ref)?
            .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
        if snapshot.encounter.revision != expected {
            return Err(atlas_local_state::LocalStateError::StaleEncounterRevision {
                encounter_key: snapshot.encounter.encounter_key,
                expected_revision: expected,
                actual_revision: snapshot.encounter.revision,
            }
            .into());
        }
        Ok(snapshot)
    })
}
fn commit(
    service: &AtlasAppService,
    prepared: Prepared,
) -> AppServiceResult<EncounterMutationView> {
    service.with_write(move |session| {
        // Recovery precedes reference checks, including a second lookup under the write lock.
        if let Some(receipt) = session.local_state().encounters().condition_event_receipt(
            &prepared.encounter_key,
            &prepared.request_id,
            &prepared.intent,
        )? {
            return Ok(EncounterMutationView {
                encounter_key: receipt.encounter_key,
            });
        }
        if let Some(revision) = prepared.reference_revision {
            session.require_reference_revision(revision)?;
        }
        let receipt = session.local_state().encounters().apply_condition_event(
            &prepared.encounter_key,
            &prepared.request_id,
            &prepared.intent,
            prepared.expected_revision,
            prepared.consequences.changes,
        )?;
        Ok(EncounterMutationView {
            encounter_key: receipt.encounter_key,
        })
    })
}
fn preview(prepared: Prepared) -> ConditionEventPreviewView {
    let mut consequences = vec![];
    if let Some((participant_key, reason)) = prepared.prone_prevented {
        consequences.push(ConditionEventConsequenceView::PronePrevented {
            participant_key,
            reason,
        });
    }
    if prepared.consequences.drop_held_items
        && let Some(change) = prepared.consequences.changes.condition_changes.first()
    {
        let participant_key = match change {
            ConditionEventChange::Add(input) => input.participant_key.clone(),
            ConditionEventChange::Replace(input) => input.participant_key.clone(),
            ConditionEventChange::Remove {
                participant_key, ..
            } => participant_key.clone(),
        };
        consequences.push(ConditionEventConsequenceView::DropHeldItems { participant_key });
    }

    if let Some(amount) = prepared.consequences.hp_loss
        && let Some(participant) = prepared.consequences.changes.participant_changes.first()
    {
        consequences.push(ConditionEventConsequenceView::DirectHpLoss {
            participant_key: participant.participant_key.clone(),
            amount,
        });
    }
    if prepared.consequences.becomes_prone
        && let Some(ConditionEventChange::Add(input)) =
            prepared.consequences.changes.condition_changes.last()
    {
        consequences.push(ConditionEventConsequenceView::BecomeProne {
            participant_key: input.participant_key.clone(),
        });
    }
    for pending in &prepared.consequences.changes.pending {
        if let PendingConsequence::DrainedHpLoss {
            participant_key,
            magnitude_increase,
        } = &pending.consequence
        {
            consequences.push(ConditionEventConsequenceView::DrainedLevelRequired {
                participant_key: participant_key.clone(),
                magnitude_increase: *magnitude_increase,
            })
        }
    }
    ConditionEventPreviewView {
        encounter_key: prepared.encounter_key,
        encounter_revision: prepared.expected_revision,
        consequences,
    }
}
fn prepare_add(
    service: &AtlasAppService,
    encounter_ref: &str,
    request: AddEncounterParticipantConditionRequest,
    intent: serde_json::Value,
) -> AppServiceResult<Prepared> {
    let snapshot = snapshot(service, encounter_ref, request.expected_encounter_revision)?;
    let subject = if request.condition_ref.is_none() {
        custom_condition_subject(&snapshot, &request.participant_key)?
    } else {
        workflow::condition_subject_context(service, encounter_ref, &request.participant_key)?
    };
    let (condition, reference_revision) =
        workflow::prepare_condition_input(service, request.condition_ref, request.name)?;
    workflow::validate_condition_subject(
        &subject,
        &snapshot,
        condition.key.as_deref(),
        request.immunity_override,
    )?;
    workflow::validate_condition_parameters(
        condition.key.as_deref(),
        &request.scope,
        &request.extra_action,
    )?;
    validate_value(
        condition.key.as_deref(),
        request.value,
        request.duration.as_ref(),
    )?;
    let mut parameters = parameters_input(
        request.parameters,
        condition.key.as_deref(),
        &snapshot,
        &request.participant_key,
        None,
        &request.request_id,
    )?;
    hydrate_broken_threshold(&mut parameters, &subject.facts, request.immunity_override)?;
    let origin = origin_input(
        request.origin,
        &request.request_id,
        request.source_participant_key.as_deref(),
        &snapshot,
    )?;
    let input = atlas_local_state::AddEncounterParticipantCondition {
        participant_key: request.participant_key.clone(),
        condition_key: condition.key,
        name: condition.name,
        value: request.value,
        source_participant_key: request.source_participant_key,
        scope: workflow::condition_scope_input(request.scope, &snapshot)?,
        extra_action: workflow::condition_extra_action_input(request.extra_action)?,
        immunity_override: request.immunity_override,
        parameters,
        origin: Some(origin),
        duration: timing::duration_input(request.duration, None, &snapshot)?,
        note: request.note,
    };
    let prone_prevented = prone_exception(&subject, request.decisions.keep_upright);
    let consequences = plan_condition_change(
        &subject.participant,
        ConditionEventChange::Add(input),
        request.event_intent == ConditionEventIntentView::Apply,
        prone_prevented.is_some(),
        subject.level,
        &request.request_id,
    )?;
    Ok(Prepared {
        encounter_key: snapshot.encounter.encounter_key,
        request_id: request.request_id,
        intent,
        expected_revision: request.expected_encounter_revision,
        reference_revision: subject.reference_revision.or(reference_revision),
        consequences,
        prone_prevented,
    })
}
fn prepare_update(
    service: &AtlasAppService,
    encounter_ref: &str,
    participant_key: &str,
    request: UpdateEncounterParticipantConditionRequest,
    intent: serde_json::Value,
) -> AppServiceResult<Prepared> {
    let snapshot = snapshot(service, encounter_ref, request.expected_encounter_revision)?;
    let existing =
        workflow::condition_in_snapshot(&snapshot, participant_key, request.condition_id)?;
    let subject = if request.condition_ref.is_none() && existing.condition_key.is_none() {
        custom_condition_subject(&snapshot, participant_key)?
    } else {
        workflow::condition_subject_context(service, encounter_ref, participant_key)?
    };
    let (condition, reference_revision) = if let Some(reference) = request.condition_ref {
        workflow::prepare_condition_input(service, Some(reference), None)?
    } else {
        (
            workflow::ConditionInput {
                key: existing.condition_key.clone(),
                name: request.name.trim().into(),
            },
            None,
        )
    };
    if condition.name.is_empty() {
        return Err(AppServiceError::invalid_request(
            "condition name must not be empty",
        ));
    }
    workflow::validate_condition_subject(
        &subject,
        &snapshot,
        condition.key.as_deref(),
        request.immunity_override,
    )?;
    workflow::validate_condition_parameters(
        condition.key.as_deref(),
        &request.scope,
        &request.extra_action,
    )?;
    if !(request.value.is_none()
        && existing.value.is_none()
        && existing.condition_key == condition.key)
    {
        validate_value(
            condition.key.as_deref(),
            request.value,
            request.duration.as_ref(),
        )?;
    }
    let mut parameters = parameters_input(
        request.parameters,
        condition.key.as_deref(),
        &snapshot,
        participant_key,
        Some(&existing),
        &request.request_id,
    )?;
    hydrate_broken_threshold(&mut parameters, &subject.facts, request.immunity_override)?;
    let mut origin = if let Some(provided) = request.origin {
        let prior = existing.origin.as_ref().ok_or_else(|| {
            AppServiceError::invalid_request("this condition has no recorded application origin")
        })?;
        if prior.application_id != provided.application_id
            || prior.grant_slot != provided.grant_slot
        {
            return Err(AppServiceError::invalid_request(
                "condition application identity cannot be changed",
            ));
        }
        Some(origin_input(
            Some(provided),
            &request.request_id,
            request.source_participant_key.as_deref(),
            &snapshot,
        )?)
    } else {
        existing.origin.clone()
    };
    if let Some(origin) = &mut origin {
        origin.participant_key = request.source_participant_key.clone();
    }
    let input = atlas_local_state::UpdateEncounterParticipantCondition {
        expected_encounter_revision: request.expected_encounter_revision,
        participant_key: participant_key.into(),
        condition_id: request.condition_id,
        condition_key: condition.key,
        name: condition.name,
        value: request.value,
        source_participant_key: request.source_participant_key,
        scope: workflow::condition_scope_input(request.scope, &snapshot)?,
        extra_action: workflow::condition_extra_action_input(request.extra_action)?,
        immunity_override: request.immunity_override,
        parameters,
        origin,
        duration: timing::duration_input(request.duration, existing.duration.as_ref(), &snapshot)?,
        note: request.note,
    };
    let prone_prevented = prone_exception(&subject, request.decisions.keep_upright);
    let consequences = plan_condition_change(
        &subject.participant,
        ConditionEventChange::Replace(input),
        request.event_intent == ConditionEventIntentView::Apply,
        prone_prevented.is_some(),
        subject.level,
        &request.request_id,
    )?;
    Ok(Prepared {
        encounter_key: snapshot.encounter.encounter_key,
        request_id: request.request_id,
        intent,
        expected_revision: request.expected_encounter_revision,
        reference_revision: subject.reference_revision.or(reference_revision),
        consequences,
        prone_prevented,
    })
}
fn validate_value(
    key: Option<&str>,
    value: Option<i64>,
    duration: Option<&ConditionDurationView>,
) -> AppServiceResult<()> {
    let rule = condition_rule_for_key(key);
    if key
        .and_then(super::conditions::modeled_condition_by_ref)
        .is_some_and(|condition| condition.has_value)
        && rule != Some(ConditionRule::Stunned)
        && value.is_none()
    {
        return Err(AppServiceError::invalid_request(
            "condition requires a positive value",
        ));
    }
    if value.is_some_and(|v| v < 1) {
        return Err(AppServiceError::invalid_request(
            "condition value must be positive",
        ));
    }
    if value.is_some_and(|v| v > 9_007_199_254_740_991) {
        return Err(AppServiceError::invalid_request(
            "condition value must be a safe integer",
        ));
    }
    if rule == Some(ConditionRule::Stunned) && value.is_some() && duration.is_some() {
        return Err(AppServiceError::invalid_request(
            "Stunned uses an action value or a duration",
        ));
    }
    Ok(())
}
fn origin_input(
    input: Option<ConditionOriginView>,
    request_id: &str,
    source: Option<&str>,
    snapshot: &EncounterWithParticipants,
) -> AppServiceResult<ConditionOrigin> {
    let origin = match input {
        Some(input) => ConditionOrigin {
            application_id: input.application_id,
            grant_slot: input.grant_slot,
            participant_key: input.participant_key,
            record_key: input.record_key,
            activity_id: input.activity_id,
            artifact_generation: input.artifact_generation,
        },
        None => ConditionOrigin {
            application_id: request_id.into(),
            grant_slot: "condition".into(),
            participant_key: source.map(str::to_owned),
            record_key: None,
            activity_id: None,
            artifact_generation: None,
        },
    };
    if origin.application_id.trim().is_empty() || origin.grant_slot.trim().is_empty() {
        return Err(AppServiceError::invalid_request(
            "condition origin requires an application and grant identity",
        ));
    }
    if origin.participant_key.as_ref().is_some_and(|key| {
        !snapshot
            .participants
            .iter()
            .any(|p| &p.participant_key == key)
    }) {
        return Err(AppServiceError::invalid_request(
            "condition origin participant must belong to this encounter",
        ));
    }
    Ok(origin)
}
fn parameters_input(
    input: ConditionParametersView,
    key: Option<&str>,
    snapshot: &EncounterWithParticipants,
    participant_key: &str,
    existing: Option<&atlas_local_state::EncounterParticipantCondition>,
    request_id: &str,
) -> AppServiceResult<ConditionParameters> {
    let rule = condition_rule_for_key(key);
    if matches!(
        rule,
        Some(
            ConditionRule::Helpful
                | ConditionRule::Friendly
                | ConditionRule::Indifferent
                | ConditionRule::Unfriendly
                | ConditionRule::Hostile
        )
    ) {
        let supernatural = match input {
            ConditionParametersView::Attitude { supernatural } => supernatural,
            ConditionParametersView::None => false,
            _ => {
                return Err(AppServiceError::invalid_request(
                    "These parameters do not apply to an attitude condition",
                ));
            }
        };
        let participant = snapshot
            .participants
            .iter()
            .find(|participant| participant.participant_key == participant_key)
            .ok_or_else(|| AppServiceError::invalid_request("participant no longer exists"))?;
        if participant.participant_kind == atlas_local_state::ParticipantKind::Pc && !supernatural {
            return Err(AppServiceError::invalid_request(
                "Confirm that a supernatural effect imposes this attitude on the PC",
            ));
        }
        return Ok(ConditionParameters::Attitude { supernatural });
    }
    match (rule, input) {
        (Some(ConditionRule::Fleeing), ConditionParametersView::None) => {
            Ok(ConditionParameters::Fleeing { subject: None })
        }
        (Some(ConditionRule::Fleeing), ConditionParametersView::Fleeing { subject }) => {
            let subject = subject
                .map(|subject| match subject {
                    ConditionSubjectView::Participant { participant_key } => {
                        if !snapshot
                            .participants
                            .iter()
                            .any(|participant| participant.participant_key == participant_key)
                        {
                            return Err(AppServiceError::invalid_request(
                                "Choose a source of flight in this encounter",
                            ));
                        }
                        Ok(atlas_local_state::ConditionSubject::Participant { participant_key })
                    }
                    ConditionSubjectView::Other { description } => {
                        let description = description.trim();
                        if description.is_empty() || description.chars().count() > 512 {
                            return Err(AppServiceError::invalid_request(
                                "Name the source of flight in 1 to 512 characters",
                            ));
                        }
                        Ok(atlas_local_state::ConditionSubject::Other {
                            description: description.into(),
                        })
                    }
                })
                .transpose()?;
            Ok(ConditionParameters::Fleeing { subject })
        }
        (Some(ConditionRule::Fascinated), ConditionParametersView::None) => {
            Ok(ConditionParameters::Fascinated { subject: None })
        }
        (Some(ConditionRule::Fascinated), ConditionParametersView::Fascinated { subject }) => {
            let subject = subject
                .map(|subject| match subject {
                    ConditionSubjectView::Participant { participant_key } => {
                        if !snapshot
                            .participants
                            .iter()
                            .any(|participant| participant.participant_key == participant_key)
                        {
                            return Err(AppServiceError::invalid_request(
                                "Choose a fascination subject in this encounter",
                            ));
                        }
                        Ok(atlas_local_state::ConditionSubject::Participant { participant_key })
                    }
                    ConditionSubjectView::Other { description } => {
                        let description = description.trim();
                        if description.is_empty() || description.chars().count() > 512 {
                            return Err(AppServiceError::invalid_request(
                                "Name the fascination subject in 1 to 512 characters",
                            ));
                        }
                        Ok(atlas_local_state::ConditionSubject::Other {
                            description: description.into(),
                        })
                    }
                })
                .transpose()?;
            Ok(ConditionParameters::Fascinated { subject })
        }
        (Some(ConditionRule::Frightened), ConditionParametersView::None) => {
            Ok(ConditionParameters::Frightened {
                recovery: atlas_local_state::FrightenedRecovery::EndTurn { minimum: 0 },
            })
        }
        (Some(ConditionRule::Frightened), ConditionParametersView::Frightened { recovery }) => {
            Ok(ConditionParameters::Frightened {
                recovery: match recovery {
                    FrightenedRecoveryView::EndTurn { minimum } if minimum >= 0 => {
                        atlas_local_state::FrightenedRecovery::EndTurn { minimum }
                    }
                    FrightenedRecoveryView::EndTurn { .. } => {
                        return Err(AppServiceError::invalid_request(
                            "Frightened minimum must be nonnegative",
                        ));
                    }
                    FrightenedRecoveryView::Held => atlas_local_state::FrightenedRecovery::Held,
                    FrightenedRecoveryView::SourceDependent => {
                        atlas_local_state::FrightenedRecovery::SourceDependent
                    }
                },
            })
        }
        (Some(ConditionRule::Broken), ConditionParametersView::None) => {
            Ok(ConditionParameters::Broken { threshold: None })
        }
        (Some(ConditionRule::Broken), ConditionParametersView::Broken { threshold }) => {
            if threshold.is_some_and(|value| value < 0) {
                return Err(AppServiceError::invalid_request(
                    "Broken threshold must be nonnegative",
                ));
            }
            Ok(ConditionParameters::Broken { threshold })
        }
        (
            Some(ConditionRule::PersistentDamage),
            ConditionParametersView::PersistentDamage {
                formula,
                damage_type,
                multiplier,
                recovery,
                episode_id,
            },
        ) => {
            if formula.len() > 512 || damage_type.trim().is_empty() {
                return Err(AppServiceError::invalid_request(
                    "persistent damage requires a bounded formula and damage type",
                ));
            }
            if !super::conditions::condition_catalog()
                .damage_types
                .iter()
                .any(|kind| kind.value == damage_type)
            {
                return Err(AppServiceError::invalid_request(
                    "select a supported damage type",
                ));
            }
            atlas_dice::prepare(&formula)
                .map_err(|error| AppServiceError::invalid_request(error.to_string()))?;
            let canonical_episode = snapshot
                .participants
                .iter()
                .find(|p| p.participant_key == participant_key)
                .and_then(|p| {
                    p.conditions
                        .iter()
                        .filter(|c| existing.is_none_or(|old| old.condition_id != c.condition_id))
                        .find_map(|c| match &c.parameters {
                            ConditionParameters::PersistentDamage {
                                damage_type: kind,
                                episode_id,
                                ..
                            } if kind == &damage_type => Some(episode_id.clone()),
                            _ => None,
                        })
                })
                .or_else(|| {
                    existing.and_then(|c| match &c.parameters {
                        ConditionParameters::PersistentDamage {
                            damage_type: kind,
                            episode_id,
                            ..
                        } if kind == &damage_type => Some(episode_id.clone()),
                        _ => None,
                    })
                })
                .unwrap_or_else(|| format!("{request_id}:{damage_type}"));
            if episode_id
                .as_ref()
                .is_some_and(|episode| episode != &canonical_episode)
            {
                return Err(AppServiceError::invalid_request(
                    "persistent damage episode does not match current sources",
                ));
            }
            Ok(ConditionParameters::PersistentDamage {
                formula,
                damage_type,
                multiplier: match multiplier {
                    DamageMultiplierView::Normal => atlas_local_state::DamageMultiplier::Normal,
                    DamageMultiplierView::Double => atlas_local_state::DamageMultiplier::Double,
                    DamageMultiplierView::Half => atlas_local_state::DamageMultiplier::Half,
                },
                recovery: match recovery {
                    PersistentRecoveryView::Standard => {
                        atlas_local_state::PersistentRecovery::Standard
                    }
                    PersistentRecoveryView::Specified { dc } if (1..=100).contains(&dc) => {
                        atlas_local_state::PersistentRecovery::Specified { dc }
                    }
                    PersistentRecoveryView::Specified { .. } => {
                        return Err(AppServiceError::invalid_request(
                            "recovery DC must be between 1 and 100",
                        ));
                    }
                    PersistentRecoveryView::SourceDependent => {
                        atlas_local_state::PersistentRecovery::SourceDependent
                    }
                },
                episode_id: canonical_episode,
            })
        }
        (
            Some(ConditionRule::PersistentDamage),
            ConditionParametersView::PersistentDamageReminder {
                formula,
                damage_type,
                multiplier,
                recovery,
            },
        ) => {
            let formula = formula
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty());
            let damage_type = damage_type.filter(|s| !s.is_empty());
            if formula.as_ref().is_some_and(|s| s.len() > 512) {
                return Err(AppServiceError::invalid_request(
                    "Damage formula is too long",
                ));
            }
            if damage_type.as_ref().is_some_and(|value| {
                !super::conditions::condition_catalog()
                    .damage_types
                    .iter()
                    .any(|kind| &kind.value == value)
            }) {
                return Err(AppServiceError::invalid_request(
                    "select a supported damage type",
                ));
            }
            if formula.is_some() && damage_type.is_some() {
                return Err(AppServiceError::invalid_request(
                    "Complete damage details require a persistent damage source",
                ));
            }
            Ok(ConditionParameters::PersistentDamageReminder {
                formula,
                damage_type,
                multiplier: match multiplier {
                    DamageMultiplierView::Normal => atlas_local_state::DamageMultiplier::Normal,
                    DamageMultiplierView::Double => atlas_local_state::DamageMultiplier::Double,
                    DamageMultiplierView::Half => atlas_local_state::DamageMultiplier::Half,
                },
                recovery: match recovery {
                    PersistentRecoveryView::Standard => {
                        atlas_local_state::PersistentRecovery::Standard
                    }
                    PersistentRecoveryView::Specified { dc } if (1..=100).contains(&dc) => {
                        atlas_local_state::PersistentRecovery::Specified { dc }
                    }
                    PersistentRecoveryView::Specified { .. } => {
                        return Err(AppServiceError::invalid_request(
                            "recovery DC must be between 1 and 100",
                        ));
                    }
                    PersistentRecoveryView::SourceDependent => {
                        atlas_local_state::PersistentRecovery::SourceDependent
                    }
                },
            })
        }
        (Some(ConditionRule::PersistentDamage), _) => Err(AppServiceError::invalid_request(
            "Persistent Damage requires a damage formula and type",
        )),
        (Some(ConditionRule::Sickened), ConditionParametersView::Sickened { recovery_dc }) => {
            if recovery_dc.is_some_and(|dc| dc < 1) {
                return Err(AppServiceError::invalid_request(
                    "recovery DC must be positive",
                ));
            }
            Ok(ConditionParameters::Sickened { recovery_dc })
        }
        (Some(ConditionRule::Sickened), ConditionParametersView::None) => {
            Ok(ConditionParameters::Sickened { recovery_dc: None })
        }
        (Some(ConditionRule::Unconscious), ConditionParametersView::Unconscious { waking }) => {
            Ok(ConditionParameters::Unconscious {
                waking: match waking {
                    WakingRuleView::Normal => atlas_local_state::WakingRule::Normal,
                    WakingRuleView::SourceControlled => {
                        atlas_local_state::WakingRule::SourceControlled
                    }
                },
            })
        }
        (Some(ConditionRule::Unconscious), ConditionParametersView::None) => {
            Ok(ConditionParameters::Unconscious {
                waking: atlas_local_state::WakingRule::Normal,
            })
        }
        (_, ConditionParametersView::None) => Ok(ConditionParameters::None),
        _ => Err(AppServiceError::invalid_request(
            "condition parameters do not match the selected condition",
        )),
    }
}

pub(super) fn prone_exception(
    subject: &workflow::ConditionSubjectContext,
    keep_upright: bool,
) -> Option<(String, String)> {
    if keep_upright {
        return Some((
            subject.participant.participant_key.clone(),
            "Kept upright by explicit table decision.".into(),
        ));
    }
    let prone = super::conditions::modeled_condition_by_rule(ConditionRule::Prone)?;
    match super::rules::condition_applicability(prone, &subject.participant, &subject.facts) {
        Some(EncounterConditionEligibilityView::Immune { reason }) => {
            Some((subject.participant.participant_key.clone(), reason))
        }
        _ => None,
    }
}

fn hydrate_broken_threshold(
    parameters: &mut ConditionParameters,
    facts: &super::rules::ConditionApplicabilityFacts,
    immunity_override: bool,
) -> AppServiceResult<()> {
    if let ConditionParameters::Broken { threshold } = parameters {
        if facts.broken_threshold_known_absent && !immunity_override {
            return Err(AppServiceError::invalid_request(
                "This object has no Broken Threshold",
            ));
        }
        let source = facts
            .broken_threshold
            .as_ref()
            .and_then(serde_json::Number::as_i64);
        if source.is_some() && threshold.is_some() && source != *threshold {
            return Err(AppServiceError::invalid_request(
                "The entered Broken Threshold differs from the source",
            ));
        }
        *threshold = source.or(*threshold);
    }
    Ok(())
}

fn custom_condition_subject(
    snapshot: &EncounterWithParticipants,
    participant_key: &str,
) -> AppServiceResult<workflow::ConditionSubjectContext> {
    let participant = snapshot
        .participants
        .iter()
        .find(|participant| participant.participant_key == participant_key)
        .cloned()
        .ok_or_else(|| {
            AppServiceError::new(
                AppErrorCode::EncounterParticipantNotFound,
                "Encounter participant was not found",
            )
        })?;
    Ok(workflow::ConditionSubjectContext {
        level: participant.level_override,
        participant,
        facts: Default::default(),
        reference_revision: None,
        fortitude: None,
        perception: None,
    })
}

#[cfg(test)]
mod lifecycle_event_tests {
    use super::*;
    fn fixture() -> (AtlasAppService, String, String) {
        let worker = crate::test_support::encounter_fixture_worker().worker;
        let encounter = worker
            .create_encounter(CreateEncounterRequest {
                name: "Lifecycle".into(),
                description: None,
                note: None,
            })
            .unwrap()
            .encounter;
        worker
            .add_encounter_manual_participant(AddEncounterManualParticipantRequest {
                encounter_ref: encounter.encounter_key.clone(),
                display_name: "PC".into(),
                max_hp: Some(10),
                current_hp: Some(10),
                initiative: None,
            })
            .unwrap();
        let detail = worker.encounter(&encounter.encounter_key).unwrap();
        let participant = detail.participants[0].participant_key.clone();
        (worker, encounter.encounter_key, participant)
    }
    fn add(
        worker: &AtlasAppService,
        key: &str,
        participant: &str,
        rule: ConditionRule,
        id: &str,
        value: Option<i64>,
        intent: ConditionEventIntentView,
    ) {
        let definition = super::super::conditions::modeled_condition_by_rule(rule).unwrap();
        worker
            .add_encounter_participant_condition(
                key,
                AddEncounterParticipantConditionRequest {
                    request_id: id.into(),
                    expected_encounter_revision: worker.encounter(key).unwrap().revision,
                    event_intent: intent,
                    decisions: Default::default(),
                    participant_key: participant.into(),
                    condition_ref: Some(definition.condition_ref.into()),
                    name: None,
                    value,
                    source_participant_key: None,
                    scope: Default::default(),
                    extra_action: Default::default(),
                    immunity_override: false,
                    parameters: Default::default(),
                    origin: None,
                    duration: None,
                    note: None,
                },
            )
            .unwrap();
    }
    #[test]
    fn unsafe_condition_values_are_rejected_before_persistence() {
        let (worker, key, participant) = fixture();
        let before = worker.encounter(&key).unwrap();
        let definition =
            super::super::conditions::modeled_condition_by_rule(ConditionRule::Frightened).unwrap();
        for value in [9_007_199_254_740_992, i64::MAX] {
            let result = worker.add_encounter_participant_condition(
                &key,
                AddEncounterParticipantConditionRequest {
                    request_id: format!("unsafe-value-{value}"),
                    expected_encounter_revision: before.revision,
                    participant_key: participant.clone(),
                    condition_ref: Some(definition.condition_ref.into()),
                    name: None,
                    value: Some(value),
                    source_participant_key: None,
                    scope: Default::default(),
                    extra_action: Default::default(),
                    immunity_override: false,
                    parameters: Default::default(),
                    origin: None,
                    duration: None,
                    note: None,
                    event_intent: ConditionEventIntentView::Apply,
                    decisions: Default::default(),
                },
            );
            assert!(
                result.is_err(),
                "unsafe condition value {value} must not enter saved state"
            );
            let after = worker.encounter(&key).unwrap();
            assert_eq!(after.revision, before.revision);
            assert!(after.participants[0].conditions.is_empty());
        }
    }
    #[test]
    fn missing_level_defers_drained_loss_and_resolution_retry_never_charges_twice() {
        let (worker, key, participant) = fixture();
        add(
            &worker,
            &key,
            &participant,
            ConditionRule::Drained,
            "drained",
            Some(2),
            ConditionEventIntentView::Apply,
        );
        assert_eq!(
            worker.encounter(&key).unwrap().participants[0].current_hp,
            Some(10)
        );
        let pending = worker
            .pending_encounter_condition_consequences(&key)
            .unwrap();
        assert_eq!(pending.len(), 1);
        let request = ResolveDrainedConsequenceRequest {
            request_id: "resolve-drained".into(),
            expected_encounter_revision: worker.encounter(&key).unwrap().revision,
            consequence_key: pending[0].consequence_key.clone(),
            level: 3,
        };
        worker
            .resolve_encounter_drained_consequence(&key, request.clone())
            .unwrap();
        worker
            .resolve_encounter_drained_consequence(&key, request)
            .unwrap();
        assert_eq!(
            worker.encounter(&key).unwrap().participants[0].current_hp,
            Some(4)
        );
        assert!(
            worker
                .pending_encounter_condition_consequences(&key)
                .unwrap()
                .is_empty()
        );
    }
    #[test]
    fn independent_unconscious_sources_create_one_lasting_prone_and_removal_does_not_undo_it() {
        let (worker, key, participant) = fixture();
        add(
            &worker,
            &key,
            &participant,
            ConditionRule::Unconscious,
            "sleep-one",
            None,
            ConditionEventIntentView::Apply,
        );
        add(
            &worker,
            &key,
            &participant,
            ConditionRule::Unconscious,
            "sleep-two",
            None,
            ConditionEventIntentView::Apply,
        );
        let detail = worker.encounter(&key).unwrap();
        let rows = &detail.participants[0].conditions;
        let unconscious =
            super::super::conditions::modeled_condition_by_rule(ConditionRule::Unconscious)
                .unwrap()
                .condition_ref;
        let prone = super::super::conditions::modeled_condition_by_rule(ConditionRule::Prone)
            .unwrap()
            .condition_ref;
        assert_eq!(
            rows.iter()
                .filter(|c| c.condition_key.as_deref() == Some(prone))
                .count(),
            1
        );
        let id = rows
            .iter()
            .find(|c| c.condition_key.as_deref() == Some(unconscious))
            .unwrap()
            .condition_id;
        worker
            .remove_encounter_participant_condition(
                &key,
                &participant,
                id,
                RemoveEncounterParticipantConditionRequest {
                    request_id: "remove-sleep".into(),
                    expected_encounter_revision: detail.revision,
                    intent: ConditionRemovalIntentView::RemoveEffect,
                },
            )
            .unwrap();
        let detail = worker.encounter(&key).unwrap();
        let rows = &detail.participants[0].conditions;
        assert_eq!(
            rows.iter()
                .filter(|c| c.condition_key.as_deref() == Some(unconscious))
                .count(),
            1
        );
        assert_eq!(
            rows.iter()
                .filter(|c| c.condition_key.as_deref() == Some(prone))
                .count(),
            1
        );
    }
    #[test]
    fn correcting_unconscious_does_not_replay_initial_prone() {
        let (worker, key, participant) = fixture();
        add(
            &worker,
            &key,
            &participant,
            ConditionRule::Unconscious,
            "correct-sleep",
            None,
            ConditionEventIntentView::Correction,
        );
        assert_eq!(
            worker.encounter(&key).unwrap().participants[0]
                .conditions
                .len(),
            1
        );
    }
    #[test]
    fn applying_doomed_at_death_threshold_dies_atomically_without_an_invalid_temporary_row() {
        let (worker, key, participant) = fixture();
        add(
            &worker,
            &key,
            &participant,
            ConditionRule::Doomed,
            "doomed-four",
            Some(4),
            ConditionEventIntentView::Apply,
        );
        let detail = worker.encounter(&key).unwrap();
        assert_eq!(detail.participants[0].mortality, MortalityStateView::Dead);
        assert!(detail.participants[0].conditions.is_empty());
    }
    #[test]
    fn drained_loss_to_zero_keeps_pending_knockout_without_repeating_the_charge() {
        let (worker, key, participant) = fixture();
        worker
            .set_encounter_life_settings(
                &key,
                &participant,
                SetEncounterLifeSettingsRequest {
                    mortality: None,
                    request_id: "level".into(),
                    expected_encounter_revision: worker.encounter(&key).unwrap().revision,
                    track_dying: true,
                    level_override: Some(LevelOverrideInputView { value: Some(10) }),
                    base_death_threshold: None,
                },
            )
            .unwrap();
        add(
            &worker,
            &key,
            &participant,
            ConditionRule::Drained,
            "drain-to-zero",
            Some(1),
            ConditionEventIntentView::Apply,
        );
        assert_eq!(
            worker.encounter(&key).unwrap().participants[0].current_hp,
            Some(0)
        );
        assert_eq!(
            worker.pending_encounter_hp_lifecycle(&key).unwrap().len(),
            1
        );
    }
    #[test]
    fn actual_dying_removal_increases_wounded_but_correction_does_not() {
        for (suffix, intent, expected) in [
            ("effect", ConditionRemovalIntentView::RemoveEffect, true),
            ("correction", ConditionRemovalIntentView::Correction, false),
        ] {
            let (worker, key, participant) = fixture();
            add(
                &worker,
                &key,
                &participant,
                ConditionRule::Dying,
                "dying",
                Some(1),
                ConditionEventIntentView::Correction,
            );
            let detail = worker.encounter(&key).unwrap();
            let id = detail.participants[0].conditions[0].condition_id;
            worker
                .remove_encounter_participant_condition(
                    &key,
                    &participant,
                    id,
                    RemoveEncounterParticipantConditionRequest {
                        request_id: format!("remove-{suffix}"),
                        expected_encounter_revision: detail.revision,
                        intent,
                    },
                )
                .unwrap();
            let wounded =
                super::super::conditions::modeled_condition_by_rule(ConditionRule::Wounded)
                    .unwrap()
                    .condition_ref;
            assert_eq!(
                worker.encounter(&key).unwrap().participants[0]
                    .conditions
                    .iter()
                    .any(|condition| condition.condition_key.as_deref() == Some(wounded)),
                expected
            );
        }
    }
    #[test]
    fn pc_attitude_requires_explicit_supernatural_fact_and_does_not_reuse_immunity_override() {
        let (worker, key, participant) = fixture();
        let definition =
            super::super::conditions::modeled_condition_by_rule(ConditionRule::Helpful).unwrap();
        let request =
            |id: &str, parameters, immunity_override| AddEncounterParticipantConditionRequest {
                request_id: id.into(),
                expected_encounter_revision: worker.encounter(&key).unwrap().revision,
                event_intent: ConditionEventIntentView::Apply,
                decisions: Default::default(),
                participant_key: participant.clone(),
                condition_ref: Some(definition.condition_ref.into()),
                name: None,
                value: None,
                source_participant_key: None,
                scope: Default::default(),
                extra_action: Default::default(),
                immunity_override,
                parameters,
                origin: None,
                duration: None,
                note: None,
            };
        assert!(
            worker
                .add_encounter_participant_condition(
                    &key,
                    request("ordinary", Default::default(), false)
                )
                .is_err()
        );
        assert!(
            worker
                .add_encounter_participant_condition(
                    &key,
                    request("wrong-override", Default::default(), true)
                )
                .is_err()
        );
        worker
            .add_encounter_participant_condition(
                &key,
                request(
                    "supernatural",
                    ConditionParametersView::Attitude { supernatural: true },
                    false,
                ),
            )
            .unwrap();
        assert_eq!(
            worker.encounter(&key).unwrap().participants[0].conditions[0].parameters,
            ConditionParametersView::Attitude { supernatural: true }
        );
    }
    #[test]
    fn unknown_numeric_sources_do_not_block_unrelated_application_or_removal() {
        for unknown_rule in [
            ConditionRule::Drained,
            ConditionRule::Dying,
            ConditionRule::Doomed,
        ] {
            let (worker, key, participant) = fixture();
            let definition =
                super::super::conditions::modeled_condition_by_rule(unknown_rule).unwrap();
            let condition = worker
                .with_write(|session| {
                    Ok(session.local_state().encounters().add_condition(
                        atlas_local_state::AddEncounterParticipantCondition {
                            participant_key: participant.clone(),
                            condition_key: Some(definition.condition_ref.into()),
                            name: definition.name.into(),
                            value: None,
                            source_participant_key: None,
                            scope: Default::default(),
                            extra_action: Default::default(),
                            immunity_override: false,
                            parameters: Default::default(),
                            origin: None,
                            duration: None,
                            note: None,
                        },
                    )?)
                })
                .unwrap();
            add(
                &worker,
                &key,
                &participant,
                ConditionRule::Unconscious,
                "unrelated-unconscious",
                None,
                ConditionEventIntentView::Apply,
            );
            add(
                &worker,
                &key,
                &participant,
                ConditionRule::Frightened,
                "unrelated-frightened",
                Some(2),
                ConditionEventIntentView::Apply,
            );
            let detail = worker.encounter(&key).unwrap();
            assert_eq!(detail.participants[0].current_hp, Some(10));
            assert_eq!(detail.participants[0].mortality, MortalityStateView::Alive);
            assert!(
                detail.participants[0]
                    .conditions
                    .iter()
                    .any(|row| row.condition_id == condition.condition_id && row.value.is_none())
            );
            worker
                .update_encounter_participant_condition(
                    &key,
                    &participant,
                    UpdateEncounterParticipantConditionRequest {
                        request_id: "note-only-unknown".into(),
                        event_intent: ConditionEventIntentView::Apply,
                        decisions: Default::default(),
                        expected_encounter_revision: detail.revision,
                        condition_id: condition.condition_id,
                        condition_ref: None,
                        name: definition.name.into(),
                        value: None,
                        source_participant_key: None,
                        scope: Default::default(),
                        extra_action: Default::default(),
                        immunity_override: false,
                        parameters: Default::default(),
                        origin: None,
                        duration: None,
                        note: Some("Retained unknown magnitude".into()),
                    },
                )
                .unwrap();
            let detail = worker.encounter(&key).unwrap();
            if unknown_rule == ConditionRule::Drained {
                worker
                    .remove_encounter_participant_condition(
                        &key,
                        &participant,
                        condition.condition_id,
                        RemoveEncounterParticipantConditionRequest {
                            request_id: "remove-unknown-drained".into(),
                            expected_encounter_revision: detail.revision,
                            intent: ConditionRemovalIntentView::RemoveEffect,
                        },
                    )
                    .unwrap();
                assert_eq!(
                    worker.encounter(&key).unwrap().participants[0].current_hp,
                    Some(10)
                );
            }
        }
    }
    #[test]
    fn fascinated_subject_is_typed_scoped_and_independent_of_causal_origin() {
        let (worker, key, participant) = fixture();
        worker
            .add_encounter_manual_participant(AddEncounterManualParticipantRequest {
                encounter_ref: key.clone(),
                display_name: "Originator".into(),
                max_hp: None,
                current_hp: None,
                initiative: None,
            })
            .unwrap();
        let origin = worker.encounter(&key).unwrap().participants[1]
            .participant_key
            .clone();
        let definition =
            super::super::conditions::modeled_condition_by_rule(ConditionRule::Fascinated).unwrap();
        let request = |id: &str, parameters| AddEncounterParticipantConditionRequest {
            request_id: id.into(),
            expected_encounter_revision: worker.encounter(&key).unwrap().revision,
            event_intent: ConditionEventIntentView::Apply,
            decisions: Default::default(),
            participant_key: participant.clone(),
            condition_ref: Some(definition.condition_ref.into()),
            name: None,
            value: None,
            source_participant_key: Some(origin.clone()),
            scope: Default::default(),
            extra_action: Default::default(),
            immunity_override: false,
            parameters,
            origin: None,
            duration: None,
            note: None,
        };
        assert!(
            worker
                .add_encounter_participant_condition(
                    &key,
                    request(
                        "outside-subject",
                        ConditionParametersView::Fascinated {
                            subject: Some(ConditionSubjectView::Participant {
                                participant_key: "other-encounter".into()
                            })
                        }
                    )
                )
                .is_err()
        );
        assert!(
            worker
                .add_encounter_participant_condition(
                    &key,
                    request(
                        "empty-subject",
                        ConditionParametersView::Fascinated {
                            subject: Some(ConditionSubjectView::Other {
                                description: "  ".into()
                            })
                        }
                    )
                )
                .is_err()
        );
        worker
            .add_encounter_participant_condition(
                &key,
                request("unknown-subject", Default::default()),
            )
            .unwrap();
        assert_eq!(
            worker.encounter(&key).unwrap().participants[0].conditions[0].parameters,
            ConditionParametersView::Fascinated { subject: None }
        );
        worker
            .add_encounter_participant_condition(
                &key,
                request(
                    "object-subject",
                    ConditionParametersView::Fascinated {
                        subject: Some(ConditionSubjectView::Other {
                            description: "  the lantern  ".into(),
                        }),
                    },
                ),
            )
            .unwrap();
        let detail = worker.encounter(&key).unwrap();
        let source = &detail.participants[0].conditions[1];
        assert_eq!(
            source.parameters,
            ConditionParametersView::Fascinated {
                subject: Some(ConditionSubjectView::Other {
                    description: "the lantern".into()
                })
            }
        );
        assert_eq!(source.source_participant_key, Some(origin));
    }
}
