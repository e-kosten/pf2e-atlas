use rand::random;
use rusqlite::{Connection, OptionalExtension, params};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use super::model::{
    AddEncounterParticipant, AddEncounterParticipantCondition, Encounter, EncounterParticipant,
    EncounterParticipantCondition, EncounterStatus, EncounterWithParticipants, NewEncounter,
    ParticipantKind, ParticipantSide, ReorderEncounterParticipant, ReorderPlacement,
    UpdateEncounter, UpdateEncounterParticipant, UpdateEncounterParticipantCondition,
};
use crate::slug::validate_slug;
use crate::{LocalStateError, LocalStateResult};

pub(crate) fn insert_encounter(
    connection: &Connection,
    encounter: NewEncounter,
) -> LocalStateResult<String> {
    validate_slug(&encounter.slug)?;
    let slug = encounter.slug;
    let now = now_rfc3339()?;
    for _ in 0..5 {
        let encounter_key = new_encounter_key();
        let result = connection.execute(
            "INSERT INTO encounters (
                encounter_key, slug, name, description, note, status,
                round_number, current_turn_participant_key, created_at, updated_at
             )
             VALUES (?1, ?2, ?3, ?4, ?5, 'draft', 1, NULL, ?6, ?6)",
            params![
                encounter_key,
                slug,
                encounter.name,
                encounter.description,
                encounter.note,
                now
            ],
        );
        match result {
            Ok(_) => return Ok(encounter_key),
            Err(rusqlite::Error::SqliteFailure(error, _))
                if error.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                if slug_exists(connection, &slug)? {
                    return Err(LocalStateError::EncounterAlreadyExists(slug));
                }
            }
            Err(error) => return Err(error.into()),
        }
    }
    Err(LocalStateError::EncounterKeyAllocationFailed)
}

pub(crate) fn list(connection: &Connection) -> LocalStateResult<Vec<Encounter>> {
    let mut statement = connection.prepare(
        "SELECT encounter_key, slug, name, description, note, status, round_number,
                current_turn_participant_key, created_at, updated_at
         FROM encounters
         ORDER BY updated_at DESC, slug ASC",
    )?;
    let rows = statement.query_map([], encounter_from_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub(crate) fn get(
    connection: &Connection,
    encounter_ref: &str,
) -> LocalStateResult<Option<Encounter>> {
    connection
        .query_row(
            "SELECT encounter_key, slug, name, description, note, status, round_number,
                    current_turn_participant_key, created_at, updated_at
             FROM encounters
             WHERE encounter_key = ?1 OR slug = ?1",
            params![encounter_ref],
            encounter_from_row,
        )
        .optional()
        .map_err(Into::into)
}

pub(crate) fn get_with_participants(
    connection: &Connection,
    encounter_ref: &str,
) -> LocalStateResult<Option<EncounterWithParticipants>> {
    let Some(encounter) = get(connection, encounter_ref)? else {
        return Ok(None);
    };
    let participants = participants(connection, &encounter.encounter_key)?;
    Ok(Some(EncounterWithParticipants {
        encounter,
        participants,
    }))
}

pub(crate) fn update_encounter(
    connection: &Connection,
    encounter: UpdateEncounter,
) -> LocalStateResult<bool> {
    validate_slug(&encounter.slug)?;
    let now = now_rfc3339()?;
    let result = connection.execute(
        "UPDATE encounters
         SET slug = ?1, name = ?2, description = ?3, note = ?4, status = ?5, updated_at = ?6
         WHERE encounter_key = ?7",
        params![
            encounter.slug,
            encounter.name,
            encounter.description,
            encounter.note,
            encounter.status.as_str(),
            now,
            encounter.encounter_key
        ],
    );
    match result {
        Ok(updated) => Ok(updated > 0),
        Err(rusqlite::Error::SqliteFailure(error, _))
            if error.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            Err(LocalStateError::EncounterAlreadyExists(encounter.slug))
        }
        Err(error) => Err(error.into()),
    }
}

pub(crate) fn delete(connection: &Connection, encounter_ref: &str) -> LocalStateResult<bool> {
    let removed = connection.execute(
        "DELETE FROM encounters WHERE encounter_key = ?1 OR slug = ?1",
        params![encounter_ref],
    )?;
    Ok(removed > 0)
}

pub(crate) fn add_participant(
    connection: &Connection,
    encounter_ref: &str,
    participant: AddEncounterParticipant,
) -> LocalStateResult<String> {
    let Some(encounter_id) = encounter_id(connection, encounter_ref)? else {
        return Err(LocalStateError::EncounterNotFound(
            encounter_ref.to_string(),
        ));
    };
    let now = now_rfc3339()?;
    let position = next_position(connection, encounter_id)?;
    let initiative_order = next_initiative_order(connection, encounter_id, participant.initiative)?;
    for _ in 0..5 {
        let participant_key = new_participant_key();
        let result = connection.execute(
            "INSERT INTO encounter_participants (
                encounter_id, participant_key, record_key, participant_kind, position,
                display_name, record_title_snapshot, record_kind_snapshot, side,
                initiative, initiative_order, max_hp, current_hp, temporary_hp,
                defeated, hidden, note, created_at, updated_at
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, 0, 0, ?15, ?16, ?16)",
            params![
                encounter_id,
                participant_key,
                participant.record_key.as_ref().map(|key| key.to_string()),
                participant.participant_kind.as_str(),
                position,
                participant.display_name,
                participant.record_title_snapshot,
                participant.record_kind_snapshot,
                participant.side.as_str(),
                participant.initiative,
                initiative_order,
                participant.max_hp,
                participant.current_hp,
                participant.temporary_hp,
                participant.note,
                now,
            ],
        );
        match result {
            Ok(_) => {
                touch_encounter(connection, encounter_id)?;
                return Ok(participant_key);
            }
            Err(rusqlite::Error::SqliteFailure(error, _))
                if error.code == rusqlite::ErrorCode::ConstraintViolation => {}
            Err(error) => return Err(error.into()),
        }
    }
    Err(LocalStateError::ParticipantKeyAllocationFailed)
}

pub(crate) fn update_participant(
    connection: &Connection,
    participant: UpdateEncounterParticipant,
) -> LocalStateResult<bool> {
    let Some((encounter_id, old_initiative)) =
        participant_owner_and_initiative(connection, &participant.participant_key)?
    else {
        return Ok(false);
    };
    let initiative_order = if old_initiative == participant.initiative {
        current_initiative_order(connection, &participant.participant_key)?
    } else {
        next_initiative_order(connection, encounter_id, participant.initiative)?
    };
    let now = now_rfc3339()?;
    let updated = connection.execute(
        "UPDATE encounter_participants
         SET display_name = ?1, side = ?2, initiative = ?3, initiative_order = ?4,
             max_hp = ?5, current_hp = ?6, temporary_hp = ?7, defeated = ?8,
             hidden = ?9, note = ?10, updated_at = ?11
         WHERE participant_key = ?12",
        params![
            participant.display_name,
            participant.side.as_str(),
            participant.initiative,
            initiative_order,
            participant.max_hp,
            participant.current_hp,
            participant.temporary_hp,
            participant.defeated,
            participant.hidden,
            participant.note,
            now,
            participant.participant_key,
        ],
    )?;
    if updated > 0 {
        compact_initiative_order(connection, encounter_id, old_initiative)?;
        compact_initiative_order(connection, encounter_id, participant.initiative)?;
        touch_encounter(connection, encounter_id)?;
    }
    Ok(updated > 0)
}

pub(crate) fn reorder_participant(
    connection: &Connection,
    reorder: ReorderEncounterParticipant,
) -> LocalStateResult<bool> {
    let Some((encounter_id, old_initiative)) =
        participant_owner_and_initiative(connection, &reorder.participant_key)?
    else {
        return Ok(false);
    };
    let Some((target_encounter_id, target_initiative)) =
        participant_owner_and_initiative(connection, &reorder.target_participant_key)?
    else {
        return Err(LocalStateError::ParticipantNotFound(
            reorder.target_participant_key,
        ));
    };
    if encounter_id != target_encounter_id {
        return Err(LocalStateError::ParticipantNotFound(
            reorder.target_participant_key,
        ));
    }
    if reorder.participant_key == reorder.target_participant_key {
        return Ok(true);
    }

    let mut keys = if target_initiative.is_some() {
        participant_keys_for_initiative(connection, encounter_id, target_initiative)?
    } else {
        participant_keys_for_unset_initiative(connection, encounter_id)?
    };
    keys.retain(|key| key != &reorder.participant_key);
    let Some(target_index) = keys
        .iter()
        .position(|key| key == &reorder.target_participant_key)
    else {
        return Err(LocalStateError::ParticipantNotFound(
            reorder.target_participant_key,
        ));
    };
    let insert_index = match reorder.placement {
        ReorderPlacement::Before => target_index,
        ReorderPlacement::After => target_index + 1,
    };
    keys.insert(insert_index, reorder.participant_key.clone());
    connection.execute(
        "UPDATE encounter_participants SET initiative_order = ?1 WHERE participant_key = ?2",
        params![9_999_999_i64, reorder.participant_key],
    )?;
    connection.execute(
        "UPDATE encounter_participants SET initiative = ?1 WHERE participant_key = ?2",
        params![target_initiative, reorder.participant_key],
    )?;
    write_initiative_order(connection, &keys)?;
    if old_initiative != target_initiative {
        compact_initiative_order(connection, encounter_id, old_initiative)?;
    }
    touch_encounter(connection, encounter_id)?;
    Ok(true)
}

pub(crate) fn remove_participant(
    connection: &Connection,
    participant_key: &str,
) -> LocalStateResult<bool> {
    let Some((encounter_id, initiative)) =
        participant_owner_and_initiative(connection, participant_key)?
    else {
        return Ok(false);
    };
    let removed = connection.execute(
        "DELETE FROM encounter_participants WHERE participant_key = ?1",
        params![participant_key],
    )?;
    if removed > 0 {
        compact_positions(connection, encounter_id)?;
        compact_initiative_order(connection, encounter_id, initiative)?;
        connection.execute(
            "UPDATE encounters
             SET current_turn_participant_key = NULL
             WHERE id = ?1 AND current_turn_participant_key = ?2",
            params![encounter_id, participant_key],
        )?;
        touch_encounter(connection, encounter_id)?;
    }
    Ok(removed > 0)
}

pub(crate) fn add_condition(
    connection: &Connection,
    condition: AddEncounterParticipantCondition,
) -> LocalStateResult<i64> {
    let Some(participant_id) = participant_id(connection, &condition.participant_key)? else {
        return Err(LocalStateError::ParticipantNotFound(
            condition.participant_key,
        ));
    };
    if let Some(source_participant_key) = &condition.source_participant_key
        && participant_encounter_id(connection, source_participant_key)?
            != participant_encounter_id(connection, &condition.participant_key)?
    {
        return Err(LocalStateError::ParticipantNotFound(
            source_participant_key.clone(),
        ));
    }
    let now = now_rfc3339()?;
    connection.execute(
        "INSERT INTO encounter_participant_conditions (
            participant_id, condition_key, name, value, source_participant_key,
            duration_rounds, note, source_note, created_at, updated_at
         )
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
        params![
            participant_id,
            condition.condition_key,
            condition.name,
            condition.value,
            condition.source_participant_key,
            condition.duration_rounds,
            condition.note,
            condition.source_note,
            now,
        ],
    )?;
    let condition_id = connection.last_insert_rowid();
    if let Some(encounter_id) = participant_encounter_id(connection, &condition.participant_key)? {
        touch_encounter(connection, encounter_id)?;
    }
    Ok(condition_id)
}

pub(crate) fn update_condition(
    connection: &Connection,
    condition: UpdateEncounterParticipantCondition,
) -> LocalStateResult<bool> {
    let Some((participant_id, encounter_id)) = condition_owner(connection, condition.condition_id)?
    else {
        return Ok(false);
    };
    if let Some(source_participant_key) = &condition.source_participant_key
        && participant_encounter_id(connection, source_participant_key)? != Some(encounter_id)
    {
        return Err(LocalStateError::ParticipantNotFound(
            source_participant_key.clone(),
        ));
    }
    let now = now_rfc3339()?;
    let updated = connection.execute(
        "UPDATE encounter_participant_conditions
         SET condition_key = ?1, name = ?2, value = ?3, source_participant_key = ?4,
             duration_rounds = ?5, note = ?6, source_note = ?7, updated_at = ?8
         WHERE id = ?9 AND participant_id = ?10",
        params![
            condition.condition_key,
            condition.name,
            condition.value,
            condition.source_participant_key,
            condition.duration_rounds,
            condition.note,
            condition.source_note,
            now,
            condition.condition_id,
            participant_id,
        ],
    )?;
    if updated > 0 {
        touch_encounter(connection, encounter_id)?;
    }
    Ok(updated > 0)
}

pub(crate) fn remove_condition(
    connection: &Connection,
    condition_id: i64,
) -> LocalStateResult<bool> {
    let owner = condition_owner(connection, condition_id)?;
    let removed = connection.execute(
        "DELETE FROM encounter_participant_conditions WHERE id = ?1",
        params![condition_id],
    )?;
    if removed > 0
        && let Some((_participant_id, encounter_id)) = owner
    {
        touch_encounter(connection, encounter_id)?;
    }
    Ok(removed > 0)
}

pub(crate) fn set_current_turn(
    connection: &Connection,
    encounter_ref: &str,
    participant_key: Option<&str>,
) -> LocalStateResult<bool> {
    set_turn_state(connection, encounter_ref, participant_key, None, false)
}

pub(crate) fn set_turn_state(
    connection: &Connection,
    encounter_ref: &str,
    participant_key: Option<&str>,
    round_number: Option<i64>,
    mark_running: bool,
) -> LocalStateResult<bool> {
    let Some(encounter_id) = encounter_id(connection, encounter_ref)? else {
        return Err(LocalStateError::EncounterNotFound(
            encounter_ref.to_string(),
        ));
    };
    if let Some(participant_key) = participant_key
        && participant_encounter_id(connection, participant_key)? != Some(encounter_id)
    {
        return Err(LocalStateError::ParticipantNotFound(
            participant_key.to_string(),
        ));
    }
    let now = now_rfc3339()?;
    if let Some(round_number) = round_number {
        let status = if mark_running { Some("running") } else { None };
        connection.execute(
            "UPDATE encounters
             SET current_turn_participant_key = ?1, round_number = ?2,
                 status = COALESCE(?3, status), updated_at = ?4
             WHERE id = ?5",
            params![
                participant_key,
                round_number.max(1),
                status,
                now,
                encounter_id
            ],
        )?;
    } else {
        let status = if mark_running { Some("running") } else { None };
        connection.execute(
            "UPDATE encounters
             SET current_turn_participant_key = ?1, status = COALESCE(?2, status), updated_at = ?3
             WHERE id = ?4",
            params![participant_key, status, now, encounter_id],
        )?;
    }
    Ok(true)
}

fn participants(
    connection: &Connection,
    encounter_key: &str,
) -> LocalStateResult<Vec<EncounterParticipant>> {
    let mut statement = connection.prepare(
        "SELECT participant.participant_key, participant.record_key, participant.participant_kind,
                participant.position, participant.display_name, participant.record_title_snapshot,
                participant.record_kind_snapshot, participant.side, participant.initiative,
                participant.initiative_order, participant.max_hp, participant.current_hp,
                participant.temporary_hp, participant.defeated, participant.hidden,
                participant.note, participant.created_at, participant.updated_at
         FROM encounter_participants participant
         JOIN encounters encounter ON encounter.id = participant.encounter_id
         WHERE encounter.encounter_key = ?1
         ORDER BY
           participant.initiative IS NULL ASC,
           participant.initiative DESC,
           participant.initiative_order ASC,
           participant.position ASC",
    )?;
    let rows = statement.query_map(params![encounter_key], participant_from_row)?;
    let mut participants = rows.collect::<Result<Vec<_>, _>>()?;
    for participant in &mut participants {
        participant.conditions = participant_conditions(connection, &participant.participant_key)?;
    }
    Ok(participants)
}

fn encounter_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Encounter> {
    let status: String = row.get(5)?;
    Ok(Encounter {
        encounter_key: row.get(0)?,
        slug: row.get(1)?,
        name: row.get(2)?,
        description: row.get(3)?,
        note: row.get(4)?,
        status: EncounterStatus::from_str(&status),
        round_number: row.get(6)?,
        current_turn_participant_key: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn participant_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<EncounterParticipant> {
    let participant_kind: String = row.get(2)?;
    let side: String = row.get(7)?;
    Ok(EncounterParticipant {
        participant_key: row.get(0)?,
        record_key: row.get(1)?,
        participant_kind: ParticipantKind::from_str(&participant_kind),
        position: row.get(3)?,
        display_name: row.get(4)?,
        record_title_snapshot: row.get(5)?,
        record_kind_snapshot: row.get(6)?,
        side: ParticipantSide::from_str(&side),
        initiative: row.get(8)?,
        initiative_order: row.get(9)?,
        max_hp: row.get(10)?,
        current_hp: row.get(11)?,
        temporary_hp: row.get(12)?,
        defeated: row.get(13)?,
        hidden: row.get(14)?,
        note: row.get(15)?,
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
        conditions: Vec::new(),
    })
}

pub(crate) fn participant(
    connection: &Connection,
    participant_key: &str,
) -> LocalStateResult<Option<EncounterParticipant>> {
    let mut statement = connection.prepare(
        "SELECT participant.participant_key, participant.record_key, participant.participant_kind,
                participant.position, participant.display_name, participant.record_title_snapshot,
                participant.record_kind_snapshot, participant.side, participant.initiative,
                participant.initiative_order, participant.max_hp, participant.current_hp,
                participant.temporary_hp, participant.defeated, participant.hidden,
                participant.note, participant.created_at, participant.updated_at
         FROM encounter_participants participant
         WHERE participant.participant_key = ?1",
    )?;
    let Some(mut participant) = statement
        .query_row(params![participant_key], participant_from_row)
        .optional()?
    else {
        return Ok(None);
    };
    participant.conditions = participant_conditions(connection, participant_key)?;
    Ok(Some(participant))
}

fn participant_conditions(
    connection: &Connection,
    participant_key: &str,
) -> LocalStateResult<Vec<EncounterParticipantCondition>> {
    let mut statement = connection.prepare(
        "SELECT condition.id, condition.condition_key, condition.name, condition.value,
                condition.source_participant_key, condition.duration_rounds, condition.note,
                condition.source_note, condition.created_at, condition.updated_at
         FROM encounter_participant_conditions condition
         JOIN encounter_participants participant ON participant.id = condition.participant_id
         WHERE participant.participant_key = ?1
         ORDER BY condition.created_at ASC, condition.id ASC",
    )?;
    let rows = statement.query_map(params![participant_key], condition_from_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn condition_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<EncounterParticipantCondition> {
    Ok(EncounterParticipantCondition {
        condition_id: row.get(0)?,
        condition_key: row.get(1)?,
        name: row.get(2)?,
        value: row.get(3)?,
        source_participant_key: row.get(4)?,
        duration_rounds: row.get(5)?,
        note: row.get(6)?,
        source_note: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn encounter_id(connection: &Connection, encounter_ref: &str) -> rusqlite::Result<Option<i64>> {
    connection
        .query_row(
            "SELECT id FROM encounters WHERE encounter_key = ?1 OR slug = ?1",
            params![encounter_ref],
            |row| row.get(0),
        )
        .optional()
}

fn participant_encounter_id(
    connection: &Connection,
    participant_key: &str,
) -> rusqlite::Result<Option<i64>> {
    connection
        .query_row(
            "SELECT encounter_id FROM encounter_participants WHERE participant_key = ?1",
            params![participant_key],
            |row| row.get(0),
        )
        .optional()
}

fn participant_id(connection: &Connection, participant_key: &str) -> rusqlite::Result<Option<i64>> {
    connection
        .query_row(
            "SELECT id FROM encounter_participants WHERE participant_key = ?1",
            params![participant_key],
            |row| row.get(0),
        )
        .optional()
}

fn condition_owner(
    connection: &Connection,
    condition_id: i64,
) -> rusqlite::Result<Option<(i64, i64)>> {
    connection
        .query_row(
            "SELECT condition.participant_id, participant.encounter_id
             FROM encounter_participant_conditions condition
             JOIN encounter_participants participant ON participant.id = condition.participant_id
             WHERE condition.id = ?1",
            params![condition_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
}

fn participant_owner_and_initiative(
    connection: &Connection,
    participant_key: &str,
) -> rusqlite::Result<Option<(i64, Option<i64>)>> {
    connection
        .query_row(
            "SELECT encounter_id, initiative FROM encounter_participants WHERE participant_key = ?1",
            params![participant_key],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
}

fn current_initiative_order(
    connection: &Connection,
    participant_key: &str,
) -> rusqlite::Result<i64> {
    connection.query_row(
        "SELECT initiative_order FROM encounter_participants WHERE participant_key = ?1",
        params![participant_key],
        |row| row.get(0),
    )
}

fn next_position(connection: &Connection, encounter_id: i64) -> rusqlite::Result<i64> {
    let max_position: Option<i64> = connection.query_row(
        "SELECT MAX(position) FROM encounter_participants WHERE encounter_id = ?1",
        params![encounter_id],
        |row| row.get(0),
    )?;
    Ok(max_position.map_or(1, |position| position + 1))
}

fn next_initiative_order(
    connection: &Connection,
    encounter_id: i64,
    initiative: Option<i64>,
) -> rusqlite::Result<i64> {
    let max_order: Option<i64> = connection.query_row(
        "SELECT MAX(initiative_order)
         FROM encounter_participants
         WHERE encounter_id = ?1 AND initiative IS ?2",
        params![encounter_id, initiative],
        |row| row.get(0),
    )?;
    Ok(max_order.map_or(1, |order| order + 1))
}

fn compact_positions(connection: &Connection, encounter_id: i64) -> rusqlite::Result<()> {
    let keys = participant_keys_for_order(
        connection,
        encounter_id,
        "ORDER BY position ASC, participant_key ASC",
    )?;
    for (index, key) in keys.iter().enumerate() {
        connection.execute(
            "UPDATE encounter_participants SET position = ?1 WHERE participant_key = ?2",
            params![i64::try_from(index + 1).unwrap_or(i64::MAX), key],
        )?;
    }
    Ok(())
}

fn compact_initiative_order(
    connection: &Connection,
    encounter_id: i64,
    initiative: Option<i64>,
) -> rusqlite::Result<()> {
    let keys = if initiative.is_some() {
        participant_keys_for_initiative(connection, encounter_id, initiative)?
    } else {
        participant_keys_for_unset_initiative(connection, encounter_id)?
    };
    write_initiative_order(connection, &keys)
}

fn write_initiative_order(connection: &Connection, keys: &[String]) -> rusqlite::Result<()> {
    for (index, key) in keys.iter().enumerate() {
        connection.execute(
            "UPDATE encounter_participants SET initiative_order = ?1 WHERE participant_key = ?2",
            params![
                1_000_000_i64 + i64::try_from(index + 1).unwrap_or(i64::MAX),
                key
            ],
        )?;
    }
    for (index, key) in keys.iter().enumerate() {
        connection.execute(
            "UPDATE encounter_participants SET initiative_order = ?1 WHERE participant_key = ?2",
            params![i64::try_from(index + 1).unwrap_or(i64::MAX), key],
        )?;
    }
    Ok(())
}

fn participant_keys_for_initiative(
    connection: &Connection,
    encounter_id: i64,
    initiative: Option<i64>,
) -> rusqlite::Result<Vec<String>> {
    let mut statement = connection.prepare(
        "SELECT participant_key
         FROM encounter_participants
         WHERE encounter_id = ?1 AND initiative = ?2
         ORDER BY initiative_order ASC, position ASC, participant_key ASC",
    )?;
    statement
        .query_map(params![encounter_id, initiative], |row| row.get(0))?
        .collect()
}

fn participant_keys_for_unset_initiative(
    connection: &Connection,
    encounter_id: i64,
) -> rusqlite::Result<Vec<String>> {
    let mut statement = connection.prepare(
        "SELECT participant_key
         FROM encounter_participants
         WHERE encounter_id = ?1 AND initiative IS NULL
         ORDER BY initiative_order ASC, position ASC, participant_key ASC",
    )?;
    statement
        .query_map(params![encounter_id], |row| row.get(0))?
        .collect()
}

fn participant_keys_for_order(
    connection: &Connection,
    encounter_id: i64,
    order_clause: &str,
) -> rusqlite::Result<Vec<String>> {
    let mut statement = connection.prepare(&format!(
        "SELECT participant_key FROM encounter_participants WHERE encounter_id = ?1 {order_clause}"
    ))?;
    statement
        .query_map(params![encounter_id], |row| row.get(0))?
        .collect()
}

fn slug_exists(connection: &Connection, slug: &str) -> rusqlite::Result<bool> {
    connection
        .query_row(
            "SELECT 1 FROM encounters WHERE slug = ?1",
            params![slug],
            |_row| Ok(()),
        )
        .optional()
        .map(|value| value.is_some())
}

fn touch_encounter(connection: &Connection, encounter_id: i64) -> LocalStateResult<()> {
    connection.execute(
        "UPDATE encounters SET updated_at = ?1 WHERE id = ?2",
        params![now_rfc3339()?, encounter_id],
    )?;
    Ok(())
}

fn new_encounter_key() -> String {
    format!("encounter_{:016x}", random::<u64>())
}

fn new_participant_key() -> String {
    format!("participant_{:016x}", random::<u64>())
}

fn now_rfc3339() -> LocalStateResult<String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(Into::into)
}
