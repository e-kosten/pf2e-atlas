use atlas_domain::RecordKey;
use atlas_record::{PersistedRecord, PersistedRecordSet};
use diesel::SqliteConnection;
use thiserror::Error;

mod candidates;
mod content;
mod metrics;
mod parse;
mod relationships;
mod rows;
mod side_data;

#[derive(Debug, Error)]
pub enum RecordLoadError {
    #[error("index is unavailable: {0}")]
    Unavailable(String),
    #[error("record query failed: {0}")]
    QueryFailed(String),
    #[error("record data is invalid: {0}")]
    InvalidData(String),
}

pub fn load_persisted_record_set_from_diesel_connection(
    connection: &mut SqliteConnection,
) -> Result<PersistedRecordSet, RecordLoadError> {
    Ok(PersistedRecordSet {
        records: load_persisted_records_from_diesel_connection(connection)?,
        reference_edges: relationships::read_reference_edges(connection)?,
        aliases: relationships::read_aliases(connection)?,
        remaster_links: relationships::read_remaster_links(connection)?,
    })
}

pub fn load_persisted_records_from_diesel_connection(
    connection: &mut SqliteConnection,
) -> Result<Vec<PersistedRecord>, RecordLoadError> {
    let mut records = rows::read_record_rows(connection)?;
    attach_record_details(connection, &mut records)?;
    Ok(records)
}

pub fn load_persisted_records_by_key_from_diesel_connection(
    connection: &mut SqliteConnection,
    keys: &[RecordKey],
) -> Result<Vec<PersistedRecord>, RecordLoadError> {
    let mut records = rows::read_record_rows_by_keys(connection, keys)?;
    attach_record_details_by_key(connection, &mut records, keys)?;
    Ok(records)
}

pub fn load_search_candidate_records_from_diesel_connection(
    connection: &mut SqliteConnection,
    keys: &[RecordKey],
) -> Result<Vec<crate::SearchCandidateRecord>, RecordLoadError> {
    candidates::read_search_candidate_records_by_keys(connection, keys)
}

fn attach_record_details(
    connection: &mut SqliteConnection,
    records: &mut [PersistedRecord],
) -> Result<(), RecordLoadError> {
    let metrics = metrics::read_metrics(connection)?;
    let actor_data = side_data::read_actor_data(connection)?;
    let item_data = side_data::read_item_data(connection)?;
    let spell_data = side_data::read_spell_data(connection)?;
    let supplemental_content = content::read_record_content(connection)?;

    for record in records {
        let key = record.key.to_string();
        record.metrics = metrics.get(&key).cloned().unwrap_or_default();
        record.actor_data = actor_data.get(&key).cloned();
        record.item_data = item_data.get(&key).cloned();
        record.spell_data = spell_data.get(&key).cloned();
        record.supplemental_content = supplemental_content.get(&key).cloned().unwrap_or_default();
    }

    Ok(())
}

fn attach_record_details_by_key(
    connection: &mut SqliteConnection,
    records: &mut [PersistedRecord],
    keys: &[RecordKey],
) -> Result<(), RecordLoadError> {
    if records.is_empty() {
        return Ok(());
    }

    let metrics = metrics::read_metrics_by_keys(connection, keys)?;
    let actor_data = side_data::read_actor_data_by_keys(connection, keys)?;
    let item_data = side_data::read_item_data_by_keys(connection, keys)?;
    let spell_data = side_data::read_spell_data_by_keys(connection, keys)?;
    let supplemental_content = content::read_record_content_by_keys(connection, keys)?;

    for record in records {
        let key = record.key.to_string();
        record.metrics = metrics.get(&key).cloned().unwrap_or_default();
        record.actor_data = actor_data.get(&key).cloned();
        record.item_data = item_data.get(&key).cloned();
        record.spell_data = spell_data.get(&key).cloned();
        record.supplemental_content = supplemental_content.get(&key).cloned().unwrap_or_default();
    }

    Ok(())
}
