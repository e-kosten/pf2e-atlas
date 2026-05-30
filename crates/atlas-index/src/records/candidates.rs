use atlas_artifact::schema::records as record_table;
use atlas_domain::RecordKey;
use rusqlite::{Connection, params_from_iter};

use crate::SearchCandidateRecord;

use super::RecordLoadError;
use super::parse::{json_string_array, parse_record_family, parse_record_key};
use super::scoped::{key_parameters, select_by_keys_sql};

pub(super) fn read_search_candidate_records_by_keys(
    connection: &Connection,
    keys: &[RecordKey],
) -> Result<Vec<SearchCandidateRecord>, RecordLoadError> {
    if keys.is_empty() {
        return Ok(Vec::new());
    }

    let parameters = key_parameters(keys);
    let sql = select_by_keys_sql(
        record_table::TABLE.name(),
        &[
            record_table::columns::RECORD_KEY,
            record_table::columns::NAME,
            record_table::columns::TRAITS_JSON,
            record_table::columns::RECORD_FAMILY,
            record_table::columns::FOUNDRY_RECORD_TYPE,
            record_table::columns::TAXONOMY_FAMILIES_JSON,
            record_table::columns::SYSTEM_CATEGORY,
            record_table::columns::SYSTEM_GROUP,
        ],
        record_table::columns::RECORD_KEY.name(),
        &[record_table::columns::RECORD_KEY.name()],
        parameters.len(),
    );
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut rows = statement
        .query(params_from_iter(parameters.iter()))
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut records = Vec::new();
    while let Some(row) = rows
        .next()
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
    {
        let record_key = row
            .get::<_, String>(record_table::columns::RECORD_KEY.name())
            .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
        let traits_json = row
            .get::<_, String>(record_table::columns::TRAITS_JSON.name())
            .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
        let record_family = row
            .get::<_, String>(record_table::columns::RECORD_FAMILY.name())
            .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
        let taxonomy_families_json = row
            .get::<_, String>(record_table::columns::TAXONOMY_FAMILIES_JSON.name())
            .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;

        records.push(SearchCandidateRecord {
            key: parse_record_key(&record_key)?,
            name: row
                .get(record_table::columns::NAME.name())
                .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?,
            traits: json_string_array("records.traits_json", &traits_json)?,
            record_family: parse_record_family(&record_family)?,
            foundry_record_type: row
                .get(record_table::columns::FOUNDRY_RECORD_TYPE.name())
                .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?,
            taxonomy_families: json_string_array(
                "records.taxonomy_families_json",
                &taxonomy_families_json,
            )?,
            system_category: row
                .get(record_table::columns::SYSTEM_CATEGORY.name())
                .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?,
            system_group: row
                .get(record_table::columns::SYSTEM_GROUP.name())
                .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?,
        });
    }
    Ok(records)
}
