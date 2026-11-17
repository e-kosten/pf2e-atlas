use super::service::{PartiesRead, PartiesWrite};
use super::storage::new_key;
use crate::{LocalStateError, LocalStateResult, PcReference};
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Debug, PartialEq, Eq)]
pub struct PcPartyMember {
    pub party_key: String,
    pub member_key: String,
}

pub(crate) fn encode(reference: &PcReference) -> LocalStateResult<String> {
    serde_json::to_string(reference)
        .map_err(|error| LocalStateError::InvalidWorkspaceRecovery(error.to_string()))
}

pub(crate) fn decode(payload: &str) -> LocalStateResult<PcReference> {
    let reference: PcReference = serde_json::from_str(payload)
        .map_err(|error| LocalStateError::InvalidWorkspaceRecovery(error.to_string()))?;
    reference.validate()?;
    Ok(reference)
}

impl PartiesRead<'_> {
    pub fn pc_reference(&self, party: &str, member: &str) -> LocalStateResult<Option<PcReference>> {
        let payload: Option<String> = self
            .connection
            .query_row(
                "SELECT r.payload FROM party_members m JOIN parties p USING(party_key)
             LEFT JOIN party_member_references r USING(member_key)
             WHERE p.owner_id=?1 AND p.party_key=?2 AND m.member_key=?3",
                params![self.owner_id, party, member],
                |row| row.get(0),
            )
            .optional()?
            .ok_or_else(member_not_found)?;
        payload.as_deref().map(decode).transpose()
    }

    pub fn encounter_pc_member(
        &self,
        encounter: &str,
        participant: &str,
    ) -> LocalStateResult<Option<PcPartyMember>> {
        let (id, _) = encounter_pc(self.connection, self.owner_id, encounter, participant)?;
        Ok(self
            .connection
            .query_row(
                "SELECT m.party_key,m.member_key FROM encounter_pc_party_members a
             JOIN party_members m USING(member_key) JOIN parties p USING(party_key)
             WHERE a.participant_id=?1 AND p.owner_id=?2",
                params![id, self.owner_id],
                |row| {
                    Ok(PcPartyMember {
                        party_key: row.get(0)?,
                        member_key: row.get(1)?,
                    })
                },
            )
            .optional()?)
    }
}

impl PartiesWrite<'_> {
    pub fn save_imported_pc(
        &self,
        party_key: &str,
        revision: i64,
        existing_member: Option<&str>,
        name: &str,
        level: i64,
        reference: &PcReference,
    ) -> LocalStateResult<String> {
        crate::atomic_action::run(self.connection, || {
            reference.validate()?;
            let party = self.get_at_revision(party_key, revision)?;
            let member_key = if let Some(key) = existing_member {
                self.pc_reference(party_key, key)?;
                key.to_owned()
            } else {
                new_key("member")
            };
            super::service::validate_member_fields(name, Some(level))?;
            let payload = encode(reference)?;
            if existing_member.is_some() {
                self.connection.execute(
                    "UPDATE party_members SET name=?1,level=?2 WHERE member_key=?3",
                    params![name.trim(), level, member_key],
                )?;
            } else {
                super::service::validate_member_count(party.members.len() + 1)?;
                self.connection.execute(
                    "INSERT INTO party_members(member_key,party_key,position,name,level)
                     SELECT ?1,?2,COALESCE(MAX(position)+1,0),?3,?4
                     FROM party_members WHERE party_key=?2",
                    params![member_key, party_key, name.trim(), level],
                )?;
            }
            put_reference(self.connection, &member_key, &payload)?;
            self.bump_party_revision(party_key)?;
            Ok(member_key)
        })
    }

    pub fn update_pc_reference(
        &self,
        party_key: &str,
        revision: i64,
        member_key: &str,
        reference: &PcReference,
    ) -> LocalStateResult<()> {
        crate::atomic_action::run(self.connection, || {
            self.get_at_revision(party_key, revision)?;
            if self.pc_reference(party_key, member_key)?.is_none() {
                return Err(LocalStateError::InvalidParty(
                    "This PC has no imported reference.",
                ));
            }
            reference.validate()?;
            put_reference(self.connection, member_key, &encode(reference)?)?;
            self.bump_party_revision(party_key)
        })
    }

    fn bump_party_revision(&self, key: &str) -> LocalStateResult<()> {
        self.connection.execute(
            "UPDATE parties SET revision=revision+1 WHERE party_key=?1 AND owner_id=?2",
            params![key, self.owner_id],
        )?;
        Ok(())
    }

    pub fn set_encounter_pc_member(
        &self,
        encounter: &str,
        participant: &str,
        expected_encounter_revision: i64,
        member: Option<&str>,
    ) -> LocalStateResult<()> {
        crate::atomic_action::run(self.connection, || {
            let (id, revision) =
                encounter_pc(self.connection, self.owner_id, encounter, participant)?;
            if revision != expected_encounter_revision {
                return Err(LocalStateError::StaleEncounterRevision {
                    encounter_key: encounter.into(),
                    expected_revision: expected_encounter_revision,
                    actual_revision: revision,
                });
            }
            put_association(self.connection, self.owner_id, id, member)?;
            self.connection.execute(
                "UPDATE encounters SET revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE encounter_key=?1 AND owner_id=?2",
                params![encounter, self.owner_id],
            )?;
            Ok(())
        })
    }

    /// Bind newly added PCs in the same workspace transaction as the roster change.
    pub fn associate_added_pc(
        &self,
        encounter: &str,
        participant: &str,
        member: &str,
    ) -> LocalStateResult<()> {
        crate::atomic_action::run(self.connection, || {
            let (id, _) = encounter_pc(self.connection, self.owner_id, encounter, participant)?;
            put_association(self.connection, self.owner_id, id, Some(member))
        })
    }
}

fn member_not_found() -> LocalStateError {
    LocalStateError::InvalidParty("The selected PC is no longer in this party.")
}

fn encounter_pc(
    db: &Connection,
    owner: i64,
    encounter: &str,
    participant: &str,
) -> LocalStateResult<(i64, i64)> {
    db.query_row(
        "SELECT p.id,e.revision FROM encounter_participants p JOIN encounters e ON e.id=p.encounter_id
         WHERE e.owner_id=?1 AND e.encounter_key=?2 AND p.participant_key=?3 AND p.participant_kind='pc'",
        params![owner, encounter, participant],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).optional()?.ok_or_else(|| LocalStateError::ParticipantNotFound(participant.into()))
}

fn put_reference(db: &Connection, member: &str, payload: &str) -> LocalStateResult<()> {
    db.execute(
        "INSERT INTO party_member_references(member_key,payload) VALUES(?1,?2)
         ON CONFLICT(member_key) DO UPDATE SET payload=excluded.payload",
        params![member, payload],
    )?;
    Ok(())
}

fn put_association(
    db: &Connection,
    owner: i64,
    participant: i64,
    member: Option<&str>,
) -> LocalStateResult<()> {
    if let Some(member) = member {
        let changed = db.execute(
            "INSERT INTO encounter_pc_party_members(participant_id,member_key)
             SELECT ?1,m.member_key FROM party_members m JOIN parties p USING(party_key)
             WHERE p.owner_id=?2 AND m.member_key=?3
             ON CONFLICT(participant_id) DO UPDATE SET member_key=excluded.member_key",
            params![participant, owner, member],
        )?;
        if changed == 0 {
            return Err(member_not_found());
        }
    } else {
        db.execute(
            "DELETE FROM encounter_pc_party_members WHERE participant_id=?1",
            [participant],
        )?;
    }
    Ok(())
}
