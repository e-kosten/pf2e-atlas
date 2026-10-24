//! Atomic prepared condition events with recovery before revision checks.
use super::event_model::*;
use super::{EncountersRead, EncountersWrite, receipt_codec, storage};
use crate::{LocalStateError, LocalStateResult};
use rusqlite::{OptionalExtension, params};

impl EncountersRead<'_> {
    pub fn condition_event_receipt(
        &self,
        encounter_key: &str,
        request_id: &str,
        intent: &serde_json::Value,
    ) -> LocalStateResult<Option<EncounterEventReceipt>> {
        super::initiative_rolls::validate_request(request_id, intent)?;
        let row: Option<(String,String)> = self.connection.query_row("SELECT intent_json,receipt_json FROM encounter_event_receipts WHERE encounter_id=(SELECT id FROM encounters WHERE encounter_key=?1) AND request_id=?2", params![encounter_key,request_id], |row| Ok((row.get(0)?,row.get(1)?))).optional()?;
        row.map(|(saved, receipt)| {
            if serde_json::from_str::<serde_json::Value>(&saved)? != *intent {
                return Err(LocalStateError::InvalidRollRequest(
                    "event request ID was reused with different intent",
                ));
            }
            receipt_codec::decode(&receipt)
        })
        .transpose()
    }
    pub fn boundary_receipt(
        &self,
        key: &str,
        claim: &EncounterBoundaryClaim,
    ) -> LocalStateResult<Option<EncounterEventReceipt>> {
        let payload:Option<String>=self.connection.query_row("SELECT r.receipt_json FROM encounter_boundary_claims c JOIN encounter_event_receipts r ON r.encounter_id=c.encounter_id AND r.request_id=c.request_id WHERE c.encounter_id=(SELECT id FROM encounters WHERE encounter_key=?1) AND c.participant_key=?2 AND c.round_number=?3 AND c.kind=?4 AND c.scope_key=?5",params![key,claim.participant_key,claim.round_number,claim.kind.as_str(),claim.scope_key],|row|row.get(0)).optional()?;
        payload.map(|text| receipt_codec::decode(&text)).transpose()
    }
    pub fn boundary_claims(
        &self,
        key: &str,
        participant_key: &str,
        round_number: i64,
    ) -> LocalStateResult<Vec<EncounterBoundaryClaim>> {
        let mut statement=self.connection.prepare("SELECT kind,scope_key FROM encounter_boundary_claims WHERE encounter_id=(SELECT id FROM encounters WHERE encounter_key=?1) AND participant_key=?2 AND round_number=?3 ORDER BY kind,scope_key")?;
        let rows = statement.query_map(params![key, participant_key, round_number], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        rows.map(|row| {
            let (kind, scope_key) = row?;
            let kind = match kind.as_str() {
                "prepared_persistent_damage" => EncounterBoundaryKind::PreparedPersistentDamage,
                "prepared_persistent_recovery" => EncounterBoundaryKind::PreparedPersistentRecovery,
                "persistent_damage" => EncounterBoundaryKind::PersistentDamage,
                "persistent_recovery" => EncounterBoundaryKind::PersistentRecovery,
                "turn_start" => EncounterBoundaryKind::TurnStart,
                "turn_end" => EncounterBoundaryKind::TurnEnd,
                _ => {
                    return Err(LocalStateError::IncompatibleSchema(
                        "unknown encounter boundary kind".into(),
                    ));
                }
            };
            Ok(EncounterBoundaryClaim {
                participant_key: participant_key.into(),
                round_number,
                kind,
                scope_key,
            })
        })
        .collect()
    }
    pub fn boundary_claimed(
        &self,
        key: &str,
        claim: &EncounterBoundaryClaim,
    ) -> LocalStateResult<bool> {
        Ok(self.connection.query_row("SELECT EXISTS(SELECT 1 FROM encounter_boundary_claims WHERE encounter_id=(SELECT id FROM encounters WHERE encounter_key=?1) AND participant_key=?2 AND round_number=?3 AND kind=?4 AND scope_key=?5)",params![key,claim.participant_key,claim.round_number,claim.kind.as_str(),claim.scope_key],|row|row.get(0))?)
    }
    pub fn pending_condition_consequences(
        &self,
        encounter_key: &str,
    ) -> LocalStateResult<Vec<SavedPendingConsequence>> {
        let mut statement = self.connection.prepare("SELECT consequence_key,payload_json FROM encounter_pending_consequences WHERE encounter_id=(SELECT id FROM encounters WHERE encounter_key=?1) ORDER BY consequence_key")?;
        let rows = statement.query_map([encounter_key], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?;
        rows.map(|row| {
            let (consequence_key, payload) = row?;
            Ok(SavedPendingConsequence {
                consequence_key,
                consequence: receipt_codec::decode(&payload)?,
            })
        })
        .collect()
    }
}
impl EncountersWrite<'_> {
    /// Remove workflow-selected obsolete decisions as part of an already guarded write.
    pub fn remove_pending_condition_consequences(
        &self,
        encounter_key: &str,
        keys: &[&str],
    ) -> LocalStateResult<()> {
        for key in keys {
            self.connection.execute(
                "DELETE FROM encounter_pending_consequences WHERE encounter_id=(SELECT id FROM encounters WHERE encounter_key=?1) AND consequence_key=?2",
                params![encounter_key, key],
            )?;
        }
        Ok(())
    }

    pub fn apply_condition_event(
        &self,
        encounter_key: &str,
        request_id: &str,
        intent: &serde_json::Value,
        expected_revision: i64,
        changes: PreparedConditionEvent,
    ) -> LocalStateResult<EncounterEventReceipt> {
        self.apply_condition_event_with(
            encounter_key,
            request_id,
            intent,
            expected_revision,
            || Ok(changes),
        )
    }
    /// Bounded pure evaluation after recovery and encounter guards, before writes.
    pub fn apply_condition_event_with(
        &self,
        encounter_key: &str,
        request_id: &str,
        intent: &serde_json::Value,
        expected_revision: i64,
        evaluate: impl FnOnce() -> LocalStateResult<PreparedConditionEvent>,
    ) -> LocalStateResult<EncounterEventReceipt> {
        self.apply_condition_event_with_claims(
            encounter_key,
            request_id,
            intent,
            expected_revision,
            ConditionEventBoundary {
                claims: &[],
                outgoing_boundary_key: None,
            },
            evaluate,
        )
    }
    pub fn apply_claimed_condition_event(
        &self,
        key: &str,
        id: &str,
        intent: &serde_json::Value,
        revision: i64,
        boundary: ConditionEventBoundary<'_>,
        changes: PreparedConditionEvent,
    ) -> LocalStateResult<EncounterEventReceipt> {
        self.apply_condition_event_with_claims(key, id, intent, revision, boundary, || Ok(changes))
    }
    pub fn apply_condition_event_with_claims(
        &self,
        encounter_key: &str,
        request_id: &str,
        intent: &serde_json::Value,
        expected_revision: i64,
        boundary: ConditionEventBoundary<'_>,
        evaluate: impl FnOnce() -> LocalStateResult<PreparedConditionEvent>,
    ) -> LocalStateResult<EncounterEventReceipt> {
        self.write_condition_event(
            encounter_key,
            request_id,
            intent,
            expected_revision,
            boundary,
            true,
            evaluate,
        )
    }

    /// Store actual preparation outcomes and claims without changing gameplay or its revision.
    pub fn prepare_condition_event_with_claims(
        &self,
        encounter_key: &str,
        request_id: &str,
        intent: &serde_json::Value,
        expected_revision: i64,
        boundary: ConditionEventBoundary<'_>,
        evaluate: impl FnOnce() -> LocalStateResult<Vec<ConditionEventOutcome>>,
    ) -> LocalStateResult<EncounterEventReceipt> {
        self.write_condition_event(
            encounter_key,
            request_id,
            intent,
            expected_revision,
            boundary,
            false,
            || {
                Ok(PreparedConditionEvent {
                    outcomes: evaluate()?,
                    ..Default::default()
                })
            },
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "shared atomic event guards for writes and receipt-only preparation"
    )]
    fn write_condition_event(
        &self,
        encounter_key: &str,
        request_id: &str,
        intent: &serde_json::Value,
        expected_revision: i64,
        boundary: ConditionEventBoundary<'_>,
        advances_revision: bool,
        evaluate: impl FnOnce() -> LocalStateResult<PreparedConditionEvent>,
    ) -> LocalStateResult<EncounterEventReceipt> {
        let ConditionEventBoundary {
            claims,
            outgoing_boundary_key,
        } = boundary;
        crate::atomic_action::run(self.connection, || {
            if let Some(receipt) =
                self.condition_event_receipt(encounter_key, request_id, intent)?
            {
                return Ok(receipt);
            }
            let snapshot = storage::get_with_participants(self.connection, encounter_key)?
                .ok_or_else(|| LocalStateError::EncounterNotFound(encounter_key.into()))?;
            if snapshot.encounter.encounter_key != encounter_key {
                return Err(LocalStateError::InvalidRollRequest(
                    "use the canonical encounter key",
                ));
            }
            if snapshot.encounter.revision != expected_revision {
                return Err(LocalStateError::StaleEncounterRevision {
                    encounter_key: encounter_key.into(),
                    expected_revision,
                    actual_revision: snapshot.encounter.revision,
                });
            }
            let outgoing = outgoing_boundary_key
                .map(|key| self.outgoing_turn(encounter_key, key))
                .transpose()?
                .flatten();
            if outgoing_boundary_key.is_some() && outgoing.is_none() {
                return Err(LocalStateError::InvalidRollRequest(
                    "outgoing boundary is no longer current",
                ));
            }
            for claim in claims {
                let archived = outgoing.as_ref().is_some_and(|saved| {
                    saved
                        .snapshot
                        .encounter
                        .current_turn_participant_key
                        .as_deref()
                        == Some(claim.participant_key.as_str())
                        && saved.snapshot.encounter.round_number == claim.round_number
                        && saved.next_participant_key.is_some()
                        && claim.kind != EncounterBoundaryKind::TurnStart
                });
                if (outgoing_boundary_key.is_some() && !archived)
                    || (outgoing_boundary_key.is_none()
                        && claim.round_number != snapshot.encounter.round_number)
                    || !snapshot
                        .participants
                        .iter()
                        .any(|p| p.participant_key == claim.participant_key)
                {
                    return Err(LocalStateError::InvalidRollRequest(
                        "boundary claim must identify the current turn or its saved outgoing boundary",
                    ));
                }
                if self.boundary_claimed(encounter_key, claim)? {
                    return Err(LocalStateError::EncounterBoundaryConflict);
                }
                self.connection.execute("INSERT INTO encounter_boundary_claims(encounter_id,participant_key,round_number,kind,scope_key,request_id) SELECT id,?2,?3,?4,?5,?6 FROM encounters WHERE encounter_key=?1",params![encounter_key,claim.participant_key,claim.round_number,claim.kind.as_str(),claim.scope_key,request_id])?;
            }
            let changes = evaluate()?;
            for change in &changes.participant_changes {
                let participant = snapshot
                    .participants
                    .iter()
                    .find(|p| p.participant_key == change.participant_key)
                    .ok_or_else(|| {
                        LocalStateError::ParticipantNotFound(change.participant_key.clone())
                    })?;
                if participant.revision != change.expected_revision {
                    return Err(LocalStateError::StaleParticipantRevision {
                        participant_key: change.participant_key.clone(),
                        expected_revision: change.expected_revision,
                        actual_revision: participant.revision,
                    });
                }
            }
            let mut added_condition_ids = Vec::new();
            for change in changes.condition_changes {
                match change {
                    ConditionEventChange::Add(condition) => {
                        if !snapshot
                            .participants
                            .iter()
                            .any(|p| p.participant_key == condition.participant_key)
                        {
                            return Err(LocalStateError::ParticipantNotFound(
                                condition.participant_key,
                            ));
                        }
                        let origin = condition.origin.clone();
                        let participant = condition.participant_key.clone();
                        if let Some(origin) = &origin {
                            let prior:Option<i64>=self.connection.query_row("SELECT condition_id FROM encounter_condition_applications WHERE encounter_id=(SELECT id FROM encounters WHERE encounter_key=?1) AND participant_key=?2 AND application_id=?3 AND grant_slot=?4",params![encounter_key,participant,origin.application_id,origin.grant_slot],|r|r.get(0)).optional()?;
                            if prior.is_some() {
                                return Err(LocalStateError::InvalidRollRequest(
                                    "condition application was already committed",
                                ));
                            }
                        }
                        let id = storage::add_condition(self.connection, condition)?;
                        if let Some(origin) = origin {
                            self.connection.execute("INSERT INTO encounter_condition_applications(encounter_id,participant_key,application_id,grant_slot,condition_id) SELECT id,?2,?3,?4,?5 FROM encounters WHERE encounter_key=?1",params![encounter_key,participant,origin.application_id,origin.grant_slot,id])?;
                        }
                        added_condition_ids.push(id);
                    }
                    ConditionEventChange::Replace(mut condition) => {
                        // Earlier changes in this same event may already have touched the encounter.
                        condition.expected_encounter_revision =
                            storage::get(self.connection, encounter_key)?
                                .ok_or_else(|| {
                                    LocalStateError::EncounterNotFound(encounter_key.into())
                                })?
                                .revision;
                        if !snapshot
                            .participants
                            .iter()
                            .any(|p| p.participant_key == condition.participant_key)
                        {
                            return Err(LocalStateError::ParticipantNotFound(
                                condition.participant_key,
                            ));
                        }
                        if !storage::update_condition(self.connection, condition)? {
                            return Err(LocalStateError::InvalidRollRequest(
                                "condition no longer exists",
                            ));
                        }
                    }
                    ConditionEventChange::Remove {
                        participant_key,
                        condition_id,
                    } => {
                        if !snapshot
                            .participants
                            .iter()
                            .any(|p| p.participant_key == participant_key)
                        {
                            return Err(LocalStateError::ParticipantNotFound(participant_key));
                        }
                        if !storage::remove_condition(
                            self.connection,
                            &participant_key,
                            condition_id,
                        )? {
                            return Err(LocalStateError::InvalidRollRequest(
                                "condition no longer exists",
                            ));
                        }
                    }
                }
            }
            for resources in &changes.turn_resources {
                self.save_turn_resources(encounter_key, resources)?;
            }
            for change in changes.participant_changes {
                self.connection.execute("UPDATE encounter_participants SET current_hp=?2,temporary_hp=?3,track_dying=?4,mortality=?5,defeated=?6,hazard_disabled=?7,level_override=?8,base_death_threshold=?9,hp_origin=CASE WHEN current_hp IS NOT ?2 AND hp_origin='derived_pristine' THEN 'derived_edited' ELSE hp_origin END,revision=revision+1 WHERE participant_key=?1",params![change.participant_key,change.current_hp,change.temporary_hp,change.track_dying,match change.mortality {super::MortalityState::Alive=>"alive",super::MortalityState::Dead=>"dead"},change.defeated,change.hazard_disabled,change.level_override,change.base_death_threshold])?;
            }
            for movement in changes.initiative_moves {
                if !snapshot
                    .participants
                    .iter()
                    .any(|p| p.participant_key == movement.participant_key)
                    || !snapshot
                        .participants
                        .iter()
                        .any(|p| p.participant_key == movement.target_participant_key)
                {
                    return Err(LocalStateError::InvalidRollRequest(
                        "initiative movement must stay in the event encounter",
                    ));
                }
                if !storage::reorder_participant(self.connection, movement)? {
                    return Err(LocalStateError::InvalidRollRequest(
                        "initiative participant no longer exists",
                    ));
                }
            }
            for change in changes.precedence_changes {
                if !snapshot
                    .participants
                    .iter()
                    .any(|p| p.participant_key == change.participant_key)
                {
                    return Err(LocalStateError::ParticipantNotFound(change.participant_key));
                }
                let resolve = |source| match source {
                    ConditionEventSource::Existing(id) => Ok(id),
                    ConditionEventSource::Added(index) => added_condition_ids
                        .get(index)
                        .copied()
                        .ok_or(LocalStateError::InvalidRollRequest(
                            "event source addition is unavailable",
                        )),
                };
                let members = change
                    .sources
                    .into_iter()
                    .map(resolve)
                    .collect::<LocalStateResult<Vec<_>>>()?;
                super::persistent_precedence::write(
                    self.connection,
                    &change.participant_key,
                    &super::PersistentSourcePrecedence {
                        episode_id: change.episode_id,
                        source_order: members,
                    },
                )?;
            }
            for pending in &changes.pending {
                let participant = match &pending.consequence {
                    PendingConsequence::BrokenHpThreshold { participant_key }
                    | PendingConsequence::ConfusedDamageRecovery {
                        participant_key, ..
                    }
                    | PendingConsequence::DrainedHpLoss {
                        participant_key, ..
                    }
                    | PendingConsequence::HpLifecycle {
                        participant_key, ..
                    }
                    | PendingConsequence::InitiativePlacement {
                        participant_key, ..
                    } => participant_key,
                };
                if !snapshot
                    .participants
                    .iter()
                    .any(|p| &p.participant_key == participant)
                {
                    return Err(LocalStateError::ParticipantNotFound(participant.clone()));
                }
                self.connection.execute("INSERT INTO encounter_pending_consequences(encounter_id,consequence_key,participant_key,payload_json) SELECT id,?2,?3,?4 FROM encounters WHERE encounter_key=?1",params![encounter_key,pending.consequence_key,participant,receipt_codec::encode(&pending.consequence)?])?;
            }
            for key in changes.resolved_pending_keys {
                self.connection.execute("DELETE FROM encounter_pending_consequences WHERE encounter_id=(SELECT id FROM encounters WHERE encounter_key=?1) AND consequence_key=?2",params![encounter_key,key])?;
            }
            if advances_revision {
                self.connection.execute("UPDATE encounters SET revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE encounter_key=?1",[encounter_key])?;
            }
            let receipt = EncounterEventReceipt {
                encounter_key: encounter_key.into(),
                request_id: request_id.into(),
                added_condition_ids,
                outcomes: changes.outcomes,
                pending: changes.pending,
            };
            self.connection.execute("INSERT INTO encounter_event_receipts(encounter_id,request_id,intent_json,receipt_json) SELECT id,?2,?3,?4 FROM encounters WHERE encounter_key=?1",params![encounter_key,request_id,serde_json::to_string(intent)?,receipt_codec::encode(&receipt)?])?;
            Ok(receipt)
        })
    }
}
