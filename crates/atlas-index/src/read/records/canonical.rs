use std::collections::BTreeMap;

use atlas_domain::RecordKey;
use atlas_record::RecordBody;
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use diesel::{Queryable, Selectable, SelectableHelper, SqliteConnection};

use crate::artifact::canonical_json;
use crate::schema::canonical_creature_records;

use super::RecordLoadError;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = canonical_creature_records)]
#[diesel(check_for_backend(Sqlite))]
struct CanonicalRecordRow {
    record_key: String,
    canonical_json: String,
}

pub(super) fn read_canonical_record_bodies(
    connection: &mut SqliteConnection,
) -> Result<Vec<RecordBody>, RecordLoadError> {
    let rows = canonical_creature_records::table
        .select(CanonicalRecordRow::as_select())
        .order(canonical_creature_records::record_key.asc())
        .load::<CanonicalRecordRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    decode_rows(rows)
}

pub(super) fn read_canonical_record_bodies_by_key(
    connection: &mut SqliteConnection,
    keys: &[RecordKey],
) -> Result<Vec<RecordBody>, RecordLoadError> {
    if keys.is_empty() {
        return Ok(Vec::new());
    }
    let keys = keys.iter().map(ToString::to_string).collect::<Vec<_>>();
    let rows = canonical_creature_records::table
        .filter(canonical_creature_records::record_key.eq_any(keys))
        .select(CanonicalRecordRow::as_select())
        .order(canonical_creature_records::record_key.asc())
        .load::<CanonicalRecordRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    decode_rows(rows)
}

fn decode_rows(rows: Vec<CanonicalRecordRow>) -> Result<Vec<RecordBody>, RecordLoadError> {
    rows.into_iter()
        .map(|row| {
            let path = format!(
                "canonical_creature_records[{}].canonical_json",
                row.record_key
            );
            let body = canonical_json::decode::<RecordBody>(&row.canonical_json, &path)
                .map_err(RecordLoadError::InvalidData)?;
            let body_key = match &body {
                RecordBody::Creature(creature) => creature.identity.record_key.to_string(),
            };
            if body_key != row.record_key {
                return Err(RecordLoadError::InvalidData(format!(
                    "{path}: hydrated record key `{body_key}` does not match row key `{}`",
                    row.record_key
                )));
            }
            Ok(body)
        })
        .collect()
}

pub(super) fn bodies_by_key(bodies: Vec<RecordBody>) -> BTreeMap<RecordKey, RecordBody> {
    bodies
        .into_iter()
        .map(|body| {
            let key = match &body {
                RecordBody::Creature(creature) => creature.identity.record_key.clone(),
            };
            (key, body)
        })
        .collect()
}
