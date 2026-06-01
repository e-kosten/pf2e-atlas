use atlas_domain::RecordKey;
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use diesel::{Queryable, Selectable, SelectableHelper, SqliteConnection};

use crate::SearchCandidateRecord;
use crate::schema::records;

use super::RecordLoadError;
use super::parse::{json_string_array, parse_record_family, parse_record_key};

pub(super) fn read_search_candidate_records_by_keys(
    connection: &mut SqliteConnection,
    keys: &[RecordKey],
) -> Result<Vec<SearchCandidateRecord>, RecordLoadError> {
    if keys.is_empty() {
        return Ok(Vec::new());
    }

    let key_strings = keys.iter().map(ToString::to_string).collect::<Vec<_>>();
    let rows = records::table
        .filter(records::record_key.eq_any(key_strings))
        .select(SearchCandidateRecordRow::as_select())
        .order(records::record_key.asc())
        .load::<SearchCandidateRecordRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    rows.into_iter()
        .map(|row| {
            Ok(SearchCandidateRecord {
                key: parse_record_key(&row.record_key)?,
                name: row.name,
                traits: json_string_array("records.traits_json", &row.traits_json)?,
                record_family: parse_record_family(&row.record_family)?,
                foundry_record_type: row.foundry_record_type,
                taxonomy_families: json_string_array(
                    "records.taxonomy_families_json",
                    &row.taxonomy_families_json,
                )?,
                system_category: row.system_category,
                system_group: row.system_group,
            })
        })
        .collect()
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = records)]
#[diesel(check_for_backend(Sqlite))]
struct SearchCandidateRecordRow {
    record_key: String,
    name: String,
    traits_json: String,
    record_family: String,
    foundry_record_type: String,
    taxonomy_families_json: String,
    system_category: Option<String>,
    system_group: Option<String>,
}
