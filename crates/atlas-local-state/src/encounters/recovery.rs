//! Versioned creation facts and atomic recovery replacements; no game-rule evaluation.
use super::*;
use super::{receipt_codec, storage};
use crate::{LocalStateError, LocalStateResult};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParticipantGameplay {
    #[serde(default)]
    pub manual_trackers: Vec<ManualTracker>,
    pub participant_variant: ParticipantVariant,
    pub variant_origin: ParticipantVariantOrigin,
    pub level_override: Option<i64>,
    pub hp_origin: ParticipantHpOrigin,
    pub max_hp: Option<i64>,
    pub current_hp: Option<i64>,
    pub temporary_hp: i64,
    pub initiative: Option<i64>,
    pub defeated: bool,
    pub hazard_disabled: bool,
    pub mortality: MortalityState,
    pub track_dying: bool,
    pub base_death_threshold: i64,
    pub conditions: Vec<EncounterParticipantCondition>,
    pub spell_resources: Vec<SpellResource>,
    pub expended_spells: Vec<String>,
    pub ability_states: Vec<ParticipantAbilityState>,
    pub persistent_precedence: Vec<PersistentSourcePrecedence>,
}
impl From<&EncounterParticipant> for ParticipantGameplay {
    fn from(p: &EncounterParticipant) -> Self {
        Self {
            participant_variant: p.participant_variant,
            variant_origin: p.variant_origin,
            level_override: p.level_override,
            hp_origin: p.hp_origin,
            max_hp: p.max_hp,
            current_hp: p.current_hp,
            temporary_hp: p.temporary_hp,
            initiative: p.initiative,
            defeated: p.defeated,
            hazard_disabled: p.hazard_disabled,
            mortality: p.mortality,
            track_dying: p.track_dying,
            base_death_threshold: p.base_death_threshold,
            conditions: p.conditions.clone(),
            spell_resources: p.spell_resources.clone(),
            expended_spells: p.expended_spells.clone(),
            ability_states: p.ability_states.clone(),
            manual_trackers: p.manual_trackers.clone(),
            persistent_precedence: p.persistent_precedence.clone(),
        }
    }
}

pub struct ApplyEncounterRecovery<'a> {
    pub encounter_key: &'a str,
    pub request_id: &'a str,
    pub intent: &'a serde_json::Value,
    pub expected_revision: i64,
    pub reset_encounter: bool,
    pub end_turn: bool,
    pub replacements: &'a [(String, ParticipantGameplay)],
}

pub(super) fn capture_baseline(connection: &Connection, key: &str) -> LocalStateResult<()> {
    let p = storage::participant(connection, key)?
        .ok_or_else(|| LocalStateError::ParticipantNotFound(key.into()))?;
    connection.execute("INSERT INTO encounter_participant_baselines(participant_key,payload_json) VALUES(?1,?2) ON CONFLICT(participant_key) DO UPDATE SET payload_json=excluded.payload_json",
        params![key, receipt_codec::encode(&ParticipantGameplay::from(&p))?])?;
    Ok(())
}
impl EncountersRead<'_> {
    pub fn participant_baseline(&self, key: &str) -> LocalStateResult<Option<ParticipantGameplay>> {
        let payload: Option<String> = self
            .connection
            .query_row(
                "SELECT payload_json FROM encounter_participant_baselines WHERE participant_key=?1",
                [key],
                |r| r.get(0),
            )
            .optional()?;
        payload.map(|s| receipt_codec::decode(&s)).transpose()
    }
    pub fn recovery_applied(
        &self,
        key: &str,
        id: &str,
        intent: &serde_json::Value,
    ) -> LocalStateResult<bool> {
        super::initiative_rolls::validate_request(id, intent)?;
        let saved: Option<String> = self.connection.query_row("SELECT intent_json FROM encounter_recovery_receipts WHERE encounter_id=(SELECT id FROM encounters WHERE encounter_key=?1) AND request_id=?2", params![key,id], |r|r.get(0)).optional()?;
        match saved {
            Some(saved) if serde_json::from_str::<serde_json::Value>(&saved)? != *intent => {
                Err(LocalStateError::InvalidRollRequest(
                    "recovery request ID was reused with different intent",
                ))
            }
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }
}
impl EncountersWrite<'_> {
    /// Replace only gameplay. The encounter guard covers roster and every participant.
    pub fn apply_recovery(&self, request: ApplyEncounterRecovery<'_>) -> LocalStateResult<()> {
        let ApplyEncounterRecovery {
            encounter_key: key,
            request_id: id,
            intent,
            expected_revision,
            reset_encounter,
            end_turn,
            replacements,
        } = request;
        crate::atomic_action::run(self.connection, || {
            if self.recovery_applied(key, id, intent)? {
                return Ok(());
            }
            let snapshot = self
                .get_with_participants(key)?
                .ok_or_else(|| LocalStateError::EncounterNotFound(key.into()))?;
            if snapshot.encounter.revision != expected_revision {
                return Err(LocalStateError::StaleEncounterRevision {
                    encounter_key: key.into(),
                    expected_revision,
                    actual_revision: snapshot.encounter.revision,
                });
            }
            let mut seen = std::collections::BTreeSet::new();
            for (participant, _) in replacements {
                if !seen.insert(participant)
                    || !snapshot
                        .participants
                        .iter()
                        .any(|p| &p.participant_key == participant)
                {
                    return Err(LocalStateError::InvalidRollRequest(
                        "recovery participants changed",
                    ));
                }
            }
            for (participant, state) in replacements {
                if snapshot
                    .participants
                    .iter()
                    .any(|p| &p.participant_key == participant && p.initiative != state.initiative)
                {
                    self.connection.execute("UPDATE encounter_participants SET initiative=NULL,initiative_order=-id WHERE participant_key=?1", [participant])?;
                }
            }
            for (participant, state) in replacements {
                let order = snapshot
                    .participants
                    .iter()
                    .find(|p| &p.participant_key == participant && p.initiative == state.initiative)
                    .map(|p| p.initiative_order);
                replace(self.connection, participant, state, order)?;
                for table in [
                    "encounter_pending_consequences",
                    "encounter_boundary_claims",
                    "encounter_condition_applications",
                ] {
                    self.connection.execute(
                        &format!("DELETE FROM {table} WHERE participant_key=?1"),
                        [participant],
                    )?;
                }
            }
            // Outgoing snapshots include the whole roster and cannot survive a reset/rest.
            self.connection.execute("DELETE FROM encounter_outgoing_turns WHERE encounter_id=(SELECT id FROM encounters WHERE encounter_key=?1)", [key])?;
            if end_turn || reset_encounter {
                self.connection.execute("UPDATE encounter_participants SET turn_resources_json=NULL,revision=revision+1 WHERE turn_resources_json IS NOT NULL AND encounter_id=(SELECT id FROM encounters WHERE encounter_key=?1)", [key])?;
            }
            if end_turn || reset_encounter {
                self.connection.execute("DELETE FROM encounter_boundary_claims WHERE encounter_id=(SELECT id FROM encounters WHERE encounter_key=?1)", [key])?;
            }
            self.connection.execute("UPDATE encounters SET revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),status=CASE WHEN ?2 THEN 'draft' ELSE status END,round_number=CASE WHEN ?2 THEN 1 ELSE round_number END,current_turn_participant_key=CASE WHEN ?3 OR ?2 THEN NULL ELSE current_turn_participant_key END WHERE encounter_key=?1", params![key,reset_encounter,end_turn])?;
            self.connection.execute("INSERT INTO encounter_recovery_receipts(encounter_id,request_id,intent_json) VALUES((SELECT id FROM encounters WHERE encounter_key=?1),?2,?3)",params![key,id,serde_json::to_string(intent)?])?;
            Ok(())
        })
    }
}
fn replace(
    c: &Connection,
    key: &str,
    s: &ParticipantGameplay,
    preserved_order: Option<i64>,
) -> LocalStateResult<()> {
    let order: i64 = c.query_row("SELECT COALESCE(MAX(initiative_order),-1)+1 FROM encounter_participants WHERE encounter_id=(SELECT encounter_id FROM encounter_participants WHERE participant_key=?1) AND initiative IS ?2", params![key,s.initiative], |r|r.get(0))?;
    let order = preserved_order.unwrap_or(order);
    c.execute("UPDATE encounter_participants SET participant_variant=?2,variant_origin=?3,level_override=?4,hp_origin=?5,max_hp=?6,current_hp=?7,temporary_hp=?8,initiative=?9,initiative_order=?10,defeated=?11,hazard_disabled=?12,mortality=?13,track_dying=?14,base_death_threshold=?15,turn_resources_json=NULL,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE participant_key=?1",
        params![key,s.participant_variant.as_str(),s.variant_origin.as_str(),s.level_override,s.hp_origin.as_str(),s.max_hp,s.current_hp,s.temporary_hp,s.initiative,order,s.defeated,s.hazard_disabled, if s.mortality==MortalityState::Dead {"dead"} else {"alive"},s.track_dying,s.base_death_threshold])?;
    // Keep surviving condition identities so unaffected workflow references stay valid.
    let p = storage::participant(c, key)?
        .ok_or_else(|| LocalStateError::ParticipantNotFound(key.into()))?;
    for old in &p.conditions {
        if let Some(new) = s
            .conditions
            .iter()
            .find(|v| v.condition_id == old.condition_id)
        {
            c.execute("UPDATE encounter_participant_conditions SET value=?2,duration_amount=?3,duration_unit=?4,duration_remaining_seconds=?5 WHERE id=?1",params![old.condition_id,new.value,new.duration.as_ref().map(|d|d.amount),new.duration.as_ref().map(|d|d.unit.as_str()),new.duration.as_ref().and_then(|d|d.remaining_seconds)])?;
        } else {
            c.execute(
                "DELETE FROM encounter_participant_conditions WHERE id=?1",
                [old.condition_id],
            )?;
        }
    }
    for table in [
        "encounter_spell_resources",
        "encounter_expended_spells",
        "encounter_persistent_precedence",
    ] {
        c.execute(&format!("DELETE FROM {table} WHERE participant_id=(SELECT id FROM encounter_participants WHERE participant_key=?1)"), [key])?;
    }
    for r in &s.spell_resources {
        c.execute("INSERT INTO encounter_spell_resources VALUES((SELECT id FROM encounter_participants WHERE participant_key=?1),?2,?3,?4,?5)",params![key,r.key,r.capacity,r.remaining,r.undo_remaining])?;
    }
    for spell in &s.expended_spells {
        c.execute("INSERT INTO encounter_expended_spells VALUES((SELECT id FROM encounter_participants WHERE participant_key=?1),?2)",params![key,spell])?;
    }
    for p in &s.persistent_precedence {
        c.execute("INSERT INTO encounter_persistent_precedence VALUES((SELECT id FROM encounter_participants WHERE participant_key=?1),?2,?3)",params![key,p.episode_id,serde_json::to_string(&p.source_order)?])?;
    }
    c.execute(
        "DELETE FROM encounter_ability_states WHERE participant_key=?1",
        [key],
    )?;
    for a in &s.ability_states {
        c.execute(
            "INSERT INTO encounter_ability_states VALUES(?1,?2,?3,?4)",
            params![
                key,
                serde_json::to_string(&a.owner)?,
                a.rule_ordinal.map(|v| v as i64).unwrap_or(-1),
                receipt_codec::encode(a)?
            ],
        )?;
    }
    c.execute(
        "DELETE FROM encounter_manual_trackers WHERE participant_key=?1",
        [key],
    )?;
    for tracker in &s.manual_trackers {
        c.execute(
            "INSERT INTO encounter_manual_trackers VALUES(?1,?2,?3)",
            params![key, tracker.tracker_key, receipt_codec::encode(tracker)?],
        )?;
    }
    Ok(())
}
