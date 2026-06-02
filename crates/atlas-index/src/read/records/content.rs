use std::collections::BTreeMap;

use atlas_domain::RecordKey;
use atlas_record::{ContentSourceKind, RecordContentDocument};
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use diesel::{Queryable, Selectable, SelectableHelper, SqliteConnection};

use crate::schema::record_content;

use super::RecordLoadError;
use super::parse::{content_document, parse_content_source_kind, parse_content_visibility};

pub(super) fn read_record_content(
    connection: &mut SqliteConnection,
) -> Result<BTreeMap<String, Vec<RecordContentDocument>>, RecordLoadError> {
    let rows = record_content::table
        .select(RecordContentRow::as_select())
        .order((
            record_content::record_key.asc(),
            record_content::ordinal.asc(),
        ))
        .load::<RecordContentRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    content_from_rows(rows)
}

pub(super) fn read_record_content_by_keys(
    connection: &mut SqliteConnection,
    keys: &[RecordKey],
) -> Result<BTreeMap<String, Vec<RecordContentDocument>>, RecordLoadError> {
    if keys.is_empty() {
        return Ok(BTreeMap::new());
    }
    let key_strings = keys.iter().map(ToString::to_string).collect::<Vec<_>>();
    let rows = record_content::table
        .filter(record_content::record_key.eq_any(key_strings))
        .select(RecordContentRow::as_select())
        .order((
            record_content::record_key.asc(),
            record_content::ordinal.asc(),
        ))
        .load::<RecordContentRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    content_from_rows(rows)
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = record_content)]
#[diesel(check_for_backend(Sqlite))]
struct RecordContentRow {
    record_key: String,
    source_kind: String,
    visibility: String,
    contributes_to_search: bool,
    contributes_to_references: bool,
    label: Option<String>,
    content_json: String,
}

fn content_from_rows(
    rows: Vec<RecordContentRow>,
) -> Result<BTreeMap<String, Vec<RecordContentDocument>>, RecordLoadError> {
    let mut values: BTreeMap<String, Vec<RecordContentDocument>> = BTreeMap::new();
    for row in rows {
        let source_kind = parse_content_source_kind(&row.source_kind)?;
        validate_content_policy(&row, source_kind)?;
        values
            .entry(row.record_key)
            .or_default()
            .push(RecordContentDocument {
                source_kind,
                label: row.label,
                document: content_document("record_content.content_json", &row.content_json)?,
            });
    }
    Ok(values)
}

fn validate_content_policy(
    row: &RecordContentRow,
    source_kind: ContentSourceKind,
) -> Result<(), RecordLoadError> {
    let visibility = parse_content_visibility(&row.visibility)?;
    if visibility != source_kind.default_visibility() {
        return Err(RecordLoadError::InvalidData(format!(
            "record_content visibility `{}` does not match default `{}` for source kind `{}`",
            row.visibility,
            source_kind.default_visibility().as_str(),
            source_kind.as_str()
        )));
    }
    if row.contributes_to_search != source_kind.default_contributes_to_search() {
        return Err(RecordLoadError::InvalidData(format!(
            "record_content contributes_to_search `{}` does not match default `{}` for source kind `{}`",
            row.contributes_to_search,
            source_kind.default_contributes_to_search(),
            source_kind.as_str()
        )));
    }
    if row.contributes_to_references != source_kind.default_contributes_to_references() {
        return Err(RecordLoadError::InvalidData(format!(
            "record_content contributes_to_references `{}` does not match default `{}` for source kind `{}`",
            row.contributes_to_references,
            source_kind.default_contributes_to_references(),
            source_kind.as_str()
        )));
    }
    Ok(())
}
