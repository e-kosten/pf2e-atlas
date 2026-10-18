//! Resource replacements are independent of HP/settings and guarded by participant revision.
use super::{EncountersWrite, SpellResource};
use crate::{LocalStateError, LocalStateResult};
use rusqlite::{Connection, OptionalExtension, params};

pub(super) fn read(
    connection: &Connection,
    participant: &str,
) -> LocalStateResult<Vec<SpellResource>> {
    let mut statement = connection.prepare("SELECT resource_key,capacity,remaining,undo_remaining FROM encounter_spell_resources WHERE participant_id=(SELECT id FROM encounter_participants WHERE participant_key=?1) ORDER BY resource_key")?;
    Ok(statement
        .query_map([participant], |row| {
            Ok(SpellResource {
                key: row.get(0)?,
                capacity: row.get(1)?,
                remaining: row.get(2)?,
                undo_remaining: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?)
}

impl EncountersWrite<'_> {
    pub fn replace_spell_resources(
        &self,
        participant: &str,
        expected_revision: i64,
        resources: &[SpellResource],
    ) -> LocalStateResult<()> {
        crate::atomic_action::run(self.connection, || {
            let tx = self.connection;
            let (id, encounter, revision): (i64,i64,i64) = tx.query_row("SELECT id,encounter_id,revision FROM encounter_participants WHERE participant_key=?1", [participant], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))).optional()?.ok_or_else(|| LocalStateError::ParticipantNotFound(participant.into()))?;
            if revision != expected_revision {
                return Err(LocalStateError::StaleParticipantRevision {
                    participant_key: participant.into(),
                    expected_revision,
                    actual_revision: revision,
                });
            }
            tx.execute(
                "DELETE FROM encounter_spell_resources WHERE participant_id=?1",
                [id],
            )?;
            for resource in resources {
                tx.execute("INSERT INTO encounter_spell_resources(participant_id,resource_key,capacity,remaining,undo_remaining) VALUES(?1,?2,?3,?4,?5)", params![id, resource.key, resource.capacity, resource.remaining, resource.undo_remaining])?;
            }
            tx.execute("UPDATE encounter_participants SET revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1", [id])?;
            tx.execute("UPDATE encounters SET revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1", [encounter])?;

            Ok(())
        })
    }
}

pub(super) fn read_expended(
    connection: &Connection,
    participant: &str,
) -> LocalStateResult<Vec<String>> {
    let mut statement = connection.prepare("SELECT spell_id FROM encounter_expended_spells WHERE participant_id=(SELECT id FROM encounter_participants WHERE participant_key=?1) ORDER BY spell_id")?;
    Ok(statement
        .query_map([participant], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?)
}

impl EncountersWrite<'_> {
    pub fn set_spell_expended(
        &self,
        participant: &str,
        expected_revision: i64,
        spell: &str,
        expended: bool,
    ) -> LocalStateResult<()> {
        crate::atomic_action::run(self.connection, || {
            let tx = self.connection;
            let (id,encounter,revision):(i64,i64,i64)=tx.query_row("SELECT id,encounter_id,revision FROM encounter_participants WHERE participant_key=?1",[participant],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?))).optional()?.ok_or_else(||LocalStateError::ParticipantNotFound(participant.into()))?;
            if revision != expected_revision {
                return Err(LocalStateError::StaleParticipantRevision {
                    participant_key: participant.into(),
                    expected_revision,
                    actual_revision: revision,
                });
            }
            if expended {
                tx.execute("INSERT OR IGNORE INTO encounter_expended_spells(participant_id,spell_id) VALUES(?1,?2)",params![id,spell])?;
            } else {
                tx.execute(
                    "DELETE FROM encounter_expended_spells WHERE participant_id=?1 AND spell_id=?2",
                    params![id, spell],
                )?;
            }
            tx.execute("UPDATE encounter_participants SET revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",[id])?;
            tx.execute("UPDATE encounters SET revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",[encounter])?;

            Ok(())
        })
    }
}
