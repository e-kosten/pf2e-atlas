//! Caller-owned atomic attempts: recovery/guards precede debits and random outcomes.
use super::attempt_model::*;
use super::{EncountersRead, EncountersWrite, ParticipantRollGuard, receipt_codec, storage};
use crate::{LocalStateError, LocalStateResult};
use rusqlite::{Connection, OptionalExtension, params};

impl EncountersRead<'_> {
    pub fn action_attempt_receipt(
        &self,
        encounter: &str,
        request: &str,
        intent: &serde_json::Value,
    ) -> LocalStateResult<Option<ActionAttemptReceipt>> {
        super::initiative_rolls::validate_request(request, intent)?;
        recover(self.connection, encounter, request, intent)
    }
    pub fn action_attempts(&self, encounter: &str) -> LocalStateResult<Vec<SavedActionAttempt>> {
        // Pending work must remain discoverable even after other participants fill
        // the recent history window; only terminal history is bounded.
        let mut statement = self.connection.prepare(
            "WITH recent AS (
                SELECT rowid FROM encounter_action_attempts
                WHERE encounter_id=(SELECT id FROM encounters WHERE encounter_key=?1)
                  AND status <> 'awaiting_check'
                ORDER BY rowid DESC LIMIT 200
             )
             SELECT a.attempt_key,a.revision,a.status,a.payload_json
             FROM encounter_action_attempts a JOIN encounters e ON e.id=a.encounter_id
             WHERE e.encounter_key=?1
               AND (a.status='awaiting_check' OR a.rowid IN (SELECT rowid FROM recent))
             ORDER BY a.rowid DESC",
        )?;
        let rows = statement
            .query_map([encounter], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        rows.into_iter()
            .map(|(key, revision, status, payload)| decode(&key, revision, &status, &payload))
            .collect()
    }
    pub fn action_attempt(
        &self,
        encounter: &str,
        attempt: &str,
    ) -> LocalStateResult<Option<SavedActionAttempt>> {
        read(self.connection, encounter, attempt)
    }
}
impl EncountersWrite<'_> {
    pub fn begin_action_attempt(
        &self,
        encounter: &str,
        request: &str,
        intent: &serde_json::Value,
        guard: ParticipantRollGuard<'_>,
        prepared: &PreparedActionAttempt,
        complete: impl FnOnce(&FrozenAction) -> LocalStateResult<ActionCompletion>,
    ) -> LocalStateResult<ActionAttemptReceipt> {
        crate::atomic_action::run(self.connection, || {
            super::initiative_rolls::validate_request(request, intent)?;
            if let Some(receipt) = recover(self.connection, encounter, request, intent)? {
                return Ok(receipt);
            }
            let snapshot = storage::get_with_participants(self.connection, encounter)?
                .ok_or_else(|| LocalStateError::EncounterNotFound(encounter.into()))?;
            if snapshot.encounter.encounter_key != encounter {
                return Err(LocalStateError::InvalidRollRequest(
                    "use the canonical encounter key",
                ));
            }
            if snapshot.encounter.revision != guard.expected_revision {
                return Err(LocalStateError::StaleEncounterRevision {
                    encounter_key: encounter.into(),
                    expected_revision: guard.expected_revision,
                    actual_revision: snapshot.encounter.revision,
                });
            }
            let participant = snapshot
                .participants
                .iter()
                .find(|participant| participant.participant_key == guard.participant_key)
                .ok_or_else(|| {
                    LocalStateError::ParticipantNotFound(guard.participant_key.into())
                })?;
            if participant.revision != guard.expected_participant_revision {
                return Err(LocalStateError::StaleParticipantRevision {
                    participant_key: guard.participant_key.into(),
                    expected_revision: guard.expected_participant_revision,
                    actual_revision: participant.revision,
                });
            }
            if prepared.context.participant_key != guard.participant_key {
                return Err(LocalStateError::InvalidRollRequest(
                    "attempt participant does not match guard",
                ));
            }
            validate_context(&prepared.context)?;
            if participant.record_key.as_deref()
                != Some(prepared.context.source_record_key.as_str())
            {
                return Err(LocalStateError::InvalidRollRequest(
                    "action source does not belong to this participant",
                ));
            }
            if let FrozenAction::Strike {
                retained_roll: Some(retained),
                ..
            } = &prepared.context.action
            {
                let duplicate: bool = self.connection.query_row(
                    "SELECT EXISTS(SELECT 1 FROM encounter_action_attempts a JOIN encounters e ON e.id=a.encounter_id WHERE e.encounter_key=?1 AND json_extract(a.payload_json,'$.payload.context.action.retained_roll.request_id')=?2)",
                    params![encounter, retained.request_id], |row| row.get(0))?;
                if duplicate {
                    return Err(LocalStateError::InvalidRollRequest(
                        "this attack already has saved context",
                    ));
                }
            }
            let pending: bool = self.connection.query_row("SELECT EXISTS(SELECT 1 FROM encounter_action_attempts a JOIN encounter_participants p ON p.id=a.participant_id WHERE p.participant_key=?1 AND a.status='awaiting_check')", [guard.participant_key], |row|row.get(0))?;
            if pending {
                return Err(LocalStateError::InvalidRollRequest(
                    "finish or abandon the pending action check",
                ));
            }
            if let Some(resources) = &prepared.resources {
                validate_resource_change(participant, prepared, resources)?;
                self.replace_spell_resources(
                    guard.participant_key,
                    guard.expected_participant_revision,
                    resources,
                )?;
            } else if prepared.context.cost.is_some() {
                return Err(LocalStateError::InvalidRollRequest(
                    "casting cost requires an atomic resource replacement",
                ));
            }
            let attempt_key = format!("action:{encounter}:{request}");
            let mut attempt = SavedActionAttempt {
                attempt_key: attempt_key.clone(),
                revision: 0,
                status: if !prepared.context.checks.is_empty() {
                    ActionAttemptStatus::AwaitingCheck
                } else {
                    ActionAttemptStatus::Ready
                },
                context: prepared.context.clone(),
                outcomes: vec![],
                completion: None,
            };
            if attempt.status == ActionAttemptStatus::Ready {
                let completion = complete(&attempt.context.action)?;
                validate_completion(&attempt.context.action, &completion)?;
                attempt.completion = Some(completion);
            }
            self.connection.execute("INSERT INTO encounter_action_attempts(attempt_key,encounter_id,participant_id,revision,status,payload_json) SELECT ?1,e.id,p.id,0,?4,?5 FROM encounters e JOIN encounter_participants p ON p.encounter_id=e.id WHERE e.encounter_key=?2 AND p.participant_key=?3",params![attempt_key,encounter,guard.participant_key,attempt.status.as_str(),receipt_codec::encode(&attempt)?])?;
            if prepared.resources.is_none() {
                advance_encounter(self.connection, encounter)?;
            }
            save_receipt(self.connection, encounter, request, intent, &attempt_key)
        })
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "Receipt identity, revision guard, and the two ordered evaluation callbacks are independent inputs"
    )]
    pub fn resolve_action_check(
        &self,
        encounter: &str,
        request: &str,
        intent: &serde_json::Value,
        attempt_key: &str,
        expected_revision: i64,
        evaluate: impl FnOnce(i64) -> LocalStateResult<ActionCheckResult>,
        complete: impl FnOnce(&FrozenAction) -> LocalStateResult<ActionCompletion>,
    ) -> LocalStateResult<ActionAttemptReceipt> {
        self.update_action_attempt(
            encounter,
            request,
            intent,
            attempt_key,
            expected_revision,
            |attempt| {
                let check = attempt.context.checks.get(attempt.outcomes.len()).ok_or(
                    LocalStateError::InvalidRollRequest("attempt has no pending check"),
                )?;
                let mut result = evaluate(check.dc)?;
                if check.stage == ActionCheckStage::Targeting
                    && attempt
                        .context
                        .target
                        .as_ref()
                        .is_some_and(|target| target.guessed_location_correct == Some(false))
                {
                    result.success = false;
                }
                if result.roll.is_some() == result.table_result.is_some()
                    || result
                        .table_result
                        .is_some_and(|value| !(1..=20).contains(&value))
                {
                    return Err(LocalStateError::InvalidRollRequest(
                        "record exactly one d20 check outcome",
                    ));
                }
                attempt.status = if result.success
                    && attempt.outcomes.len() + 1 < attempt.context.checks.len()
                {
                    ActionAttemptStatus::AwaitingCheck
                } else if result.success {
                    ActionAttemptStatus::Ready
                } else {
                    ActionAttemptStatus::Disrupted
                };
                attempt.outcomes.push(result);
                let secret_targeting = check.stage == ActionCheckStage::Targeting
                    && attempt.context.target.as_ref().is_some_and(|target| {
                        matches!(
                            &target.detection,
                            super::RelationshipKnowledge::Known {
                                value: super::RelationshipValue::Detection(
                                    super::DetectionState::Undetected
                                ),
                                ..
                            }
                        )
                    });
                if attempt.status == ActionAttemptStatus::Ready || secret_targeting {
                    let completion = complete(&attempt.context.action)?;
                    validate_completion(&attempt.context.action, &completion)?;
                    attempt.completion = Some(completion);
                }
                Ok(())
            },
        )
    }
    pub fn abandon_action_attempt(
        &self,
        encounter: &str,
        request: &str,
        intent: &serde_json::Value,
        attempt_key: &str,
        expected_revision: i64,
    ) -> LocalStateResult<ActionAttemptReceipt> {
        self.update_action_attempt(
            encounter,
            request,
            intent,
            attempt_key,
            expected_revision,
            |attempt| {
                attempt.status = ActionAttemptStatus::Abandoned;
                Ok(())
            },
        )
    }
    fn update_action_attempt(
        &self,
        encounter: &str,
        request: &str,
        intent: &serde_json::Value,
        attempt_key: &str,
        expected_revision: i64,
        update: impl FnOnce(&mut SavedActionAttempt) -> LocalStateResult<()>,
    ) -> LocalStateResult<ActionAttemptReceipt> {
        crate::atomic_action::run(self.connection, || {
            super::initiative_rolls::validate_request(request, intent)?;
            if let Some(receipt) = recover(self.connection, encounter, request, intent)? {
                return Ok(receipt);
            }
            let mut attempt = read(self.connection, encounter, attempt_key)?.ok_or(
                LocalStateError::InvalidRollRequest("action attempt not found"),
            )?;
            if attempt.revision != expected_revision {
                return Err(LocalStateError::StaleActionAttemptRevision {
                    attempt_key: attempt_key.into(),
                    expected_revision,
                    actual_revision: attempt.revision,
                });
            }
            if attempt.status != ActionAttemptStatus::AwaitingCheck {
                return Err(LocalStateError::InvalidRollRequest(
                    "action attempt is already resolved",
                ));
            }
            update(&mut attempt)?;
            attempt.revision =
                attempt
                    .revision
                    .checked_add(1)
                    .ok_or(LocalStateError::InvalidRollRequest(
                        "action attempt revision exhausted",
                    ))?;
            self.connection.execute("UPDATE encounter_action_attempts SET revision=?2,status=?3,payload_json=?4 WHERE attempt_key=?1",params![attempt_key,attempt.revision,attempt.status.as_str(),receipt_codec::encode(&attempt)?])?;
            advance_encounter(self.connection, encounter)?;
            save_receipt(self.connection, encounter, request, intent, attempt_key)
        })
    }
}
fn validate_completion(
    action: &FrozenAction,
    completion: &ActionCompletion,
) -> LocalStateResult<()> {
    if let FrozenAction::Strike {
        retained_roll: Some(retained),
        ..
    } = action
    {
        return if matches!(completion, ActionCompletion::StrikeRoll { roll } if *roll == retained.outcome)
        {
            Ok(())
        } else {
            Err(LocalStateError::InvalidRollRequest(
                "action changed the original attack roll",
            ))
        };
    }
    if matches!(
        (action, completion),
        (FrozenAction::Cast { .. }, ActionCompletion::Cast)
            | (
                FrozenAction::Strike {
                    automatic_self_hit: false,
                    ..
                },
                ActionCompletion::StrikeRoll { .. }
            )
            | (
                FrozenAction::Strike {
                    automatic_self_hit: true,
                    ..
                },
                ActionCompletion::AutomaticSelfHit
            )
    ) {
        Ok(())
    } else {
        Err(LocalStateError::InvalidRollRequest(
            "action result does not match frozen action",
        ))
    }
}
fn validate_context(context: &FrozenActionContext) -> LocalStateResult<()> {
    if context.participant_key.is_empty()
        || context.source_fingerprint.is_empty()
        || context.source_fingerprint.len() > 256
        || context.artifact_generation.is_empty()
        || match &context.action {
            FrozenAction::Cast { spell_id, rank } => {
                spell_id.is_empty() || spell_id.len() > 128 || !(1..=10).contains(rank)
            }
            FrozenAction::Strike {
                activity_id,
                attack_number,
                modifier,
                automatic_self_hit,
                ..
            } => {
                activity_id.is_empty()
                    || activity_id.len() > 128
                    || !(1..=3).contains(attack_number)
                    || (!automatic_self_hit && modifier.is_none())
                    || modifier.is_some_and(|modifier| !(-10000..=10000).contains(&modifier))
            }
        }
        || context.judgments.len() > 200
        || context.checks.iter().any(|check| {
            check.dc < 1
                || (check.stage == ActionCheckStage::Disruption && check.sources.is_empty())
                || check.sources.len() > 200
        })
    {
        return Err(LocalStateError::InvalidRollRequest(
            "invalid frozen action context",
        ));
    }
    if let Some(target) = &context.target {
        let undetected = matches!(
            &target.detection,
            super::RelationshipKnowledge::Known {
                value: super::RelationshipValue::Detection(super::DetectionState::Undetected),
                ..
            }
        );
        if undetected
            && target.mode != ActionTargetMode::Area
            && (target.mode != ActionTargetMode::GuessedLocation
                || target.guessed_location_correct.is_none())
        {
            return Err(LocalStateError::InvalidRollRequest(
                "confirm the guessed location before committing the action",
            ));
        }
    }
    atlas_domain::RecordKey::parse(&context.source_record_key).map_err(|error| {
        LocalStateError::InvalidRecordKey {
            record_key: context.source_record_key.clone(),
            reason: error.to_string(),
        }
    })?;
    if let Some(cost) = &context.cost
        && (!matches!(context.action, FrozenAction::Cast { .. })
            || cost.resource_key.is_empty()
            || cost.before < 1
            || cost.after != cost.before - 1)
    {
        return Err(LocalStateError::InvalidRollRequest(
            "invalid casting resource cost",
        ));
    }
    Ok(())
}
fn validate_resource_change(
    participant: &super::EncounterParticipant,
    prepared: &PreparedActionAttempt,
    resources: &[super::SpellResource],
) -> LocalStateResult<()> {
    let cost = prepared
        .context
        .cost
        .as_ref()
        .ok_or(LocalStateError::InvalidRollRequest(
            "resource replacement requires casting cost",
        ))?;
    let changed = resources
        .iter()
        .find(|resource| resource.key == cost.resource_key)
        .ok_or(LocalStateError::InvalidRollRequest(
            "casting resource is missing",
        ))?;
    if changed.remaining != cost.after
        || changed.capacity < cost.before
        || resources.iter().any(|resource| {
            resource.remaining < 0
                || resource.remaining > resource.capacity
                || resource.capacity < 0
        })
    {
        return Err(LocalStateError::InvalidRollRequest(
            "casting resource replacement does not match frozen cost",
        ));
    }
    if let Some(saved) = participant
        .spell_resources
        .iter()
        .find(|resource| resource.key == cost.resource_key)
        && (saved.remaining != cost.before || saved.capacity != changed.capacity)
    {
        return Err(LocalStateError::InvalidRollRequest(
            "casting resource changed",
        ));
    }
    for saved in &participant.spell_resources {
        if saved.key != cost.resource_key
            && !resources.iter().any(|resource| {
                resource.key == saved.key
                    && resource.remaining == saved.remaining
                    && resource.capacity == saved.capacity
            })
        {
            return Err(LocalStateError::InvalidRollRequest(
                "casting changed an unrelated resource",
            ));
        }
    }
    let mut keys = std::collections::BTreeSet::new();
    if resources.iter().any(|resource| !keys.insert(&resource.key)) {
        return Err(LocalStateError::InvalidRollRequest(
            "duplicate casting resource",
        ));
    }
    Ok(())
}
fn read(
    connection: &Connection,
    encounter: &str,
    attempt: &str,
) -> LocalStateResult<Option<SavedActionAttempt>> {
    let row:Option<(i64,String,String)>=connection.query_row("SELECT a.revision,a.status,a.payload_json FROM encounter_action_attempts a JOIN encounters e ON e.id=a.encounter_id WHERE e.encounter_key=?1 AND a.attempt_key=?2",params![encounter,attempt],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?))).optional()?;
    row.map(|(revision, status, payload)| decode(attempt, revision, &status, &payload))
        .transpose()
}
fn decode(
    key: &str,
    revision: i64,
    status: &str,
    payload: &str,
) -> LocalStateResult<SavedActionAttempt> {
    let attempt: SavedActionAttempt = receipt_codec::decode(payload)?;
    validate_context(&attempt.context)?;
    if attempt.attempt_key != key
        || attempt.revision != revision
        || attempt.status.as_str() != status
    {
        return Err(LocalStateError::InvalidRollRequest(
            "saved casting attempt identity mismatch",
        ));
    }
    Ok(attempt)
}
fn recover(
    connection: &Connection,
    encounter: &str,
    request: &str,
    intent: &serde_json::Value,
) -> LocalStateResult<Option<ActionAttemptReceipt>> {
    let row:Option<(String,String)>=connection.query_row("SELECT r.intent_json,r.receipt_json FROM encounter_action_receipts r JOIN encounters e ON e.id=r.encounter_id WHERE e.encounter_key=?1 AND r.request_id=?2",params![encounter,request],|row|Ok((row.get(0)?,row.get(1)?))).optional()?;
    let Some((saved, payload)) = row else {
        return Ok(None);
    };
    if serde_json::from_str::<serde_json::Value>(&saved)? != *intent {
        return Err(LocalStateError::RollRequestConflict);
    }
    let receipt: ActionAttemptReceipt = receipt_codec::decode(&payload)?;
    if receipt.encounter_key != encounter || receipt.request_id != request {
        return Err(LocalStateError::InvalidRollRequest(
            "saved casting receipt identity mismatch",
        ));
    }
    Ok(Some(receipt))
}
fn save_receipt(
    connection: &Connection,
    encounter: &str,
    request: &str,
    intent: &serde_json::Value,
    attempt: &str,
) -> LocalStateResult<ActionAttemptReceipt> {
    let receipt = ActionAttemptReceipt {
        encounter_key: encounter.into(),
        request_id: request.into(),
        attempt_key: attempt.into(),
    };
    connection.execute("INSERT INTO encounter_action_receipts(encounter_id,request_id,intent_json,receipt_json) SELECT id,?2,?3,?4 FROM encounters WHERE encounter_key=?1",params![encounter,request,serde_json::to_string(intent)?,receipt_codec::encode(&receipt)?])?;
    Ok(receipt)
}
fn advance_encounter(connection: &Connection, encounter: &str) -> LocalStateResult<()> {
    connection.execute("UPDATE encounters SET revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE encounter_key=?1",[encounter])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    fn complete(action: &FrozenAction) -> LocalStateResult<ActionCompletion> {
        match action {
            FrozenAction::Cast { .. } => Ok(ActionCompletion::Cast),
            FrozenAction::Strike { .. } => Ok(ActionCompletion::StrikeRoll {
                roll: atlas_dice::prepare("1d20")?.roll()?,
            }),
        }
    }
    fn fixture() -> (Connection, String, String, PreparedActionAttempt) {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch("PRAGMA foreign_keys=ON; CREATE TABLE record_identity(id INTEGER PRIMARY KEY,key TEXT UNIQUE,last_known_name TEXT,availability TEXT NOT NULL);").unwrap();
        for sql in [
            LOCAL_STATE_SCHEMA_SQL,
            ENCOUNTER_STATUS_SCHEMA_SQL,
            HAZARD_PARTICIPATION_SCHEMA_SQL,
            CONDITION_CONTEXT_SCHEMA_SQL,
            CONDITION_IMMUNITY_OVERRIDE_SCHEMA_SQL,
            CONDITION_EVENTS_SCHEMA_SQL,
            ACTION_ATTEMPTS_SCHEMA_SQL,
            PERSISTENT_SELECTIONS_SCHEMA_SQL,
            ENCOUNTER_BOUNDARIES_SCHEMA_SQL,
            PARTICIPANT_FACTS_SCHEMA_SQL,
        ] {
            connection.execute_batch(sql).unwrap();
        }
        connection.execute_batch("BEGIN IMMEDIATE").unwrap();
        let store = EncountersWrite::from_connection(&connection);
        let encounter = store
            .create(NewEncounter {
                slug: "attempt".into(),
                name: "Attempt".into(),
                description: None,
                note: None,
            })
            .unwrap();
        let participant = store
            .add_participant(
                &encounter.encounter_key,
                AddEncounterParticipant {
                    spell_resources: vec![SpellResource {
                        key: "slot".into(),
                        capacity: 2,
                        remaining: 2,
                        undo_remaining: None,
                    }],
                    participant_variant: ParticipantVariant::Normal,
                    hp_origin: ParticipantHpOrigin::Explicit,
                    variant_origin: ParticipantVariantOrigin::Explicit,
                    record_key: Some(
                        atlas_domain::RecordKey::parse("actors:aaaaaaaaaaaaaaaa").unwrap(),
                    ),
                    participant_kind: ParticipantKind::Creature,
                    display_name: "Caster".into(),
                    record_title_snapshot: None,
                    record_kind_snapshot: None,
                    side: ParticipantSide::Enemy,
                    initiative: None,
                    max_hp: Some(20),
                    current_hp: Some(20),
                    temporary_hp: 0,
                    note: None,
                },
            )
            .unwrap();
        let prepared = PreparedActionAttempt {
            context: FrozenActionContext {
                participant_key: participant.participant_key.clone(),
                source_record_key: "actors:aaaaaaaaaaaaaaaa".into(),
                title: "Spell".into(),
                target: None,
                confusion: None,
                action: FrozenAction::Cast {
                    spell_id: "bbbbbbbbbbbbbbbb".into(),
                    rank: 1,
                },
                source_fingerprint: "source".into(),
                artifact_generation: "generation".into(),
                reference_revision: 1,
                judgments: vec![],
                cost: Some(SpellResourceCost {
                    resource_key: "slot".into(),
                    label: "Rank 1".into(),
                    before: 2,
                    after: 1,
                }),
                checks: vec![FrozenActionCheck {
                    stage: ActionCheckStage::Disruption,
                    dc: 7,
                    sources: vec![ActionCheckSource {
                        condition_id: 1,
                        name: "Stupefied".into(),
                    }],
                }],
            },
            resources: Some(vec![SpellResource {
                key: "slot".into(),
                capacity: 2,
                remaining: 1,
                undo_remaining: None,
            }]),
        };
        (
            connection,
            encounter.encounter_key,
            participant.participant_key,
            prepared,
        )
    }
    #[test]
    fn pending_attempt_remains_discoverable_beyond_recent_history_limit() {
        let (connection, key, participant, mut prepared) = fixture();
        let store = EncountersWrite::from_connection(&connection);
        let snapshot = store.get_with_participants(&key).unwrap().unwrap();
        let pending = store
            .begin_action_attempt(
                &key,
                "pending",
                &serde_json::json!({"action":"pending"}),
                ParticipantRollGuard {
                    expected_revision: snapshot.encounter.revision,
                    participant_key: &participant,
                    expected_participant_revision: snapshot.participants[0].revision,
                },
                &prepared,
                complete,
            )
            .unwrap();
        let other = store
            .add_participant(
                &key,
                AddEncounterParticipant {
                    record_key: Some(
                        atlas_domain::RecordKey::parse("actors:aaaaaaaaaaaaaaaa").unwrap(),
                    ),
                    participant_kind: ParticipantKind::Creature,
                    participant_variant: ParticipantVariant::Normal,
                    hp_origin: ParticipantHpOrigin::Explicit,
                    variant_origin: ParticipantVariantOrigin::Explicit,
                    display_name: "Other caster".into(),
                    record_title_snapshot: None,
                    record_kind_snapshot: None,
                    side: ParticipantSide::Enemy,
                    initiative: None,
                    max_hp: Some(20),
                    current_hp: Some(20),
                    temporary_hp: 0,
                    note: None,
                    spell_resources: vec![],
                },
            )
            .unwrap();
        prepared.context.participant_key = other.participant_key.clone();
        prepared.context.cost = None;
        prepared.context.checks.clear();
        prepared.resources = None;
        for index in 0..201 {
            let snapshot = store.get_with_participants(&key).unwrap().unwrap();
            let actor = snapshot
                .participants
                .iter()
                .find(|p| p.participant_key == other.participant_key)
                .unwrap();
            store
                .begin_action_attempt(
                    &key,
                    &format!("ready-{index}"),
                    &serde_json::json!({"action":"ready","index":index}),
                    ParticipantRollGuard {
                        expected_revision: snapshot.encounter.revision,
                        participant_key: &other.participant_key,
                        expected_participant_revision: actor.revision,
                    },
                    &prepared,
                    complete,
                )
                .unwrap();
        }
        let attempts = store.action_attempts(&key).unwrap();
        assert!(
            attempts.iter().any(|a| a.attempt_key == pending.attempt_key
                && a.status == ActionAttemptStatus::AwaitingCheck),
            "pending checks must not disappear behind completed history"
        );
        assert_eq!(
            attempts
                .iter()
                .filter(|a| a.status != ActionAttemptStatus::AwaitingCheck)
                .count(),
            200
        );
        store
            .abandon_action_attempt(
                &key,
                "abandon",
                &serde_json::json!({"action":"abandon"}),
                &pending.attempt_key,
                0,
            )
            .unwrap();
        let attempts = store.action_attempts(&key).unwrap();
        assert_eq!(attempts.len(), 200);
        assert!(
            attempts
                .iter()
                .all(|a| a.status != ActionAttemptStatus::AwaitingCheck)
        );
    }
    #[test]
    fn successful_check_is_retained_before_later_targeting_check() {
        let (connection, key, participant, mut prepared) = fixture();
        prepared.context.action = FrozenAction::Strike {
            activity_id: "strike".into(),
            attack_number: 1,
            modifier: Some(12),
            automatic_self_hit: false,
            mode: ActionRollMode::Normal,
            retained_roll: None,
        };
        prepared.context.cost = None;
        prepared.resources = None;
        prepared.context.checks.push(FrozenActionCheck {
            stage: ActionCheckStage::Targeting,
            dc: 11,
            sources: vec![ActionCheckSource {
                condition_id: 2,
                name: "Hidden".into(),
            }],
        });
        let store = EncountersWrite::from_connection(&connection);
        let before = store.get_with_participants(&key).unwrap().unwrap();
        let receipt = store
            .begin_action_attempt(
                &key,
                "strike",
                &serde_json::json!({"begin":"strike"}),
                ParticipantRollGuard {
                    expected_revision: before.encounter.revision,
                    participant_key: &participant,
                    expected_participant_revision: before.participants[0].revision,
                },
                &prepared,
                complete,
            )
            .unwrap();
        store
            .resolve_action_check(
                &key,
                "disruption",
                &serde_json::json!({"stage":1}),
                &receipt.attempt_key,
                0,
                |dc| {
                    assert_eq!(dc, 7);
                    Ok(ActionCheckResult {
                        roll: None,
                        table_result: Some(20),
                        success: true,
                    })
                },
                complete,
            )
            .unwrap();
        let saved = store
            .action_attempt(&key, &receipt.attempt_key)
            .unwrap()
            .unwrap();
        assert_eq!(saved.status, ActionAttemptStatus::AwaitingCheck);
        assert_eq!(saved.outcomes.len(), 1);
        store
            .resolve_action_check(
                &key,
                "target",
                &serde_json::json!({"stage":2}),
                &receipt.attempt_key,
                1,
                |dc| {
                    assert_eq!(dc, 11);
                    Ok(ActionCheckResult {
                        roll: None,
                        table_result: Some(20),
                        success: true,
                    })
                },
                complete,
            )
            .unwrap();
        let saved = store
            .action_attempt(&key, &receipt.attempt_key)
            .unwrap()
            .unwrap();
        assert_eq!(saved.status, ActionAttemptStatus::Ready);
        assert_eq!(saved.outcomes.len(), 2);
    }
    #[test]
    fn casting_cost_check_and_retries_commit_once_and_disruption_keeps_cost() {
        let (connection, key, participant, prepared) = fixture();
        let store = EncountersWrite::from_connection(&connection);
        let before = store.get_with_participants(&key).unwrap().unwrap();
        let intent = serde_json::json!({"begin":"spell"});
        let begin = || {
            store.begin_action_attempt(
                &key,
                "begin",
                &intent,
                ParticipantRollGuard {
                    expected_revision: before.encounter.revision,
                    participant_key: &participant,
                    expected_participant_revision: before.participants[0].revision,
                },
                &prepared,
                complete,
            )
        };
        let receipt = begin().unwrap();
        assert_eq!(begin().unwrap(), receipt);
        let charged = store.get_with_participants(&key).unwrap().unwrap();
        assert_eq!(charged.participants[0].spell_resources[0].remaining, 1);
        assert_eq!(charged.participants[0].current_hp, Some(20));
        assert_eq!(charged.encounter.revision, before.encounter.revision + 1);
        let calls = std::cell::Cell::new(0);
        let check_intent = serde_json::json!({"check":"table-1"});
        let check = || {
            store.resolve_action_check(
                &key,
                "check",
                &check_intent,
                &receipt.attempt_key,
                0,
                |dc| {
                    calls.set(calls.get() + 1);
                    assert_eq!(dc, 7);
                    Ok(ActionCheckResult {
                        roll: None,
                        table_result: Some(1),
                        success: false,
                    })
                },
                complete,
            )
        };
        let result = check().unwrap();
        assert_eq!(check().unwrap(), result);
        assert_eq!(calls.get(), 1);
        let attempt = store
            .action_attempt(&key, &receipt.attempt_key)
            .unwrap()
            .unwrap();
        assert_eq!(attempt.status, ActionAttemptStatus::Disrupted);
        assert_eq!(
            store
                .get_with_participants(&key)
                .unwrap()
                .unwrap()
                .participants[0]
                .spell_resources[0]
                .remaining,
            1
        );
        assert!(matches!(
            store.resolve_action_check(
                &key,
                "new-check",
                &check_intent,
                &receipt.attempt_key,
                0,
                |_| panic!("stale checks must not roll"),
                complete
            ),
            Err(LocalStateError::StaleActionAttemptRevision { .. })
        ));
    }
    #[test]
    fn failed_attempt_insert_rolls_back_resource_debit_and_revision() {
        let (connection, key, participant, prepared) = fixture();
        connection.execute_batch("CREATE TRIGGER reject_cast BEFORE INSERT ON encounter_action_attempts BEGIN SELECT RAISE(ABORT,'fixture rejects insert'); END;").unwrap();
        let store = EncountersWrite::from_connection(&connection);
        let before = store.get_with_participants(&key).unwrap().unwrap();
        assert!(
            store
                .begin_action_attempt(
                    &key,
                    "begin",
                    &serde_json::json!({"begin":true}),
                    ParticipantRollGuard {
                        expected_revision: before.encounter.revision,
                        participant_key: &participant,
                        expected_participant_revision: before.participants[0].revision
                    },
                    &prepared,
                    complete
                )
                .is_err()
        );
        assert_eq!(store.get_with_participants(&key).unwrap().unwrap(), before);
        assert!(store.action_attempts(&key).unwrap().is_empty());
    }
    #[test]
    fn abandon_keeps_cost_and_blocks_later_outcome_evaluation() {
        let (connection, key, participant, prepared) = fixture();
        let store = EncountersWrite::from_connection(&connection);
        let before = store.get_with_participants(&key).unwrap().unwrap();
        let receipt = store
            .begin_action_attempt(
                &key,
                "begin",
                &serde_json::json!({"begin":true}),
                ParticipantRollGuard {
                    expected_revision: before.encounter.revision,
                    participant_key: &participant,
                    expected_participant_revision: before.participants[0].revision,
                },
                &prepared,
                complete,
            )
            .unwrap();
        store
            .abandon_action_attempt(
                &key,
                "abandon",
                &serde_json::json!({"abandon":true}),
                &receipt.attempt_key,
                0,
            )
            .unwrap();
        assert_eq!(
            store
                .action_attempt(&key, &receipt.attempt_key)
                .unwrap()
                .unwrap()
                .status,
            ActionAttemptStatus::Abandoned
        );
        assert_eq!(
            store
                .get_with_participants(&key)
                .unwrap()
                .unwrap()
                .participants[0]
                .spell_resources[0]
                .remaining,
            1
        );
        assert!(
            store
                .resolve_action_check(
                    &key,
                    "check",
                    &serde_json::json!({"check":true}),
                    &receipt.attempt_key,
                    1,
                    |_| panic!("abandoned casts must not roll"),
                    complete
                )
                .is_err()
        );
    }
    #[test]
    fn wrong_guessed_location_retains_secret_roll_and_completion_but_prevents_effect() {
        let (connection, key, participant, mut prepared) = fixture();
        prepared.context.checks[0].stage = ActionCheckStage::Targeting;
        prepared.context.checks[0].dc = 11;
        prepared.context.target = Some(FrozenActionTarget {
            participant_key: "target".into(),
            mode: ActionTargetMode::GuessedLocation,
            detection: RelationshipKnowledge::Known {
                value: RelationshipValue::Detection(DetectionState::Undetected),
                provenance: RelationshipProvenance::Table,
            },
            vision_only_precise: None,
            distance_feet: None,
            guessed_location_correct: Some(false),
            visual_effect: Some(false),
            auditory_effect: Some(false),
        });
        let store = EncountersWrite::from_connection(&connection);
        let before = store.get_with_participants(&key).unwrap().unwrap();
        let receipt = store
            .begin_action_attempt(
                &key,
                "secret",
                &serde_json::json!({"secret":true}),
                ParticipantRollGuard {
                    expected_revision: before.encounter.revision,
                    participant_key: &participant,
                    expected_participant_revision: before.participants[0].revision,
                },
                &prepared,
                complete,
            )
            .unwrap();
        store
            .resolve_action_check(
                &key,
                "secret-check",
                &serde_json::json!({"check":true}),
                &receipt.attempt_key,
                0,
                |_| {
                    Ok(ActionCheckResult {
                        roll: None,
                        table_result: Some(20),
                        success: true,
                    })
                },
                complete,
            )
            .unwrap();
        let saved = store
            .action_attempt(&key, &receipt.attempt_key)
            .unwrap()
            .unwrap();
        assert_eq!(saved.status, ActionAttemptStatus::Disrupted);
        assert_eq!(saved.outcomes[0].table_result, Some(20));
        assert!(!saved.outcomes[0].success);
        assert!(matches!(saved.completion, Some(ActionCompletion::Cast)));
        assert_eq!(
            store
                .get_with_participants(&key)
                .unwrap()
                .unwrap()
                .participants[0]
                .spell_resources[0]
                .remaining,
            1
        );
    }
}
