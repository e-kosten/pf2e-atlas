use std::collections::BTreeMap;

use atlas_artifact::schema::{RECORD_CONTENT_COLUMNS, record_content, record_content_select_sql};
use atlas_domain::RecordKey;
use atlas_record::SupplementalContentDocument;
use rusqlite::{Connection, params_from_iter, types::Value};

use super::RecordLoadError;
use super::parse::{
    content_document, optional_string, parse_content_source_kind, parse_content_visibility,
    required_bool, required_string,
};
use super::scoped::{key_parameters, select_by_keys_sql};

pub(super) fn read_record_content(
    connection: &Connection,
) -> Result<BTreeMap<String, Vec<SupplementalContentDocument>>, RecordLoadError> {
    read_record_content_from_sql(connection, &record_content_select_sql(), Vec::new())
}

pub(super) fn read_record_content_by_keys(
    connection: &Connection,
    keys: &[RecordKey],
) -> Result<BTreeMap<String, Vec<SupplementalContentDocument>>, RecordLoadError> {
    if keys.is_empty() {
        return Ok(BTreeMap::new());
    }
    let parameters = key_parameters(keys);
    let sql = select_by_keys_sql(
        record_content::TABLE.name(),
        RECORD_CONTENT_COLUMNS,
        record_content::columns::RECORD_KEY.name(),
        &[
            record_content::columns::RECORD_KEY.name(),
            record_content::columns::ORDINAL.name(),
        ],
        parameters.len(),
    );
    read_record_content_from_sql(connection, &sql, parameters)
}

fn read_record_content_from_sql(
    connection: &Connection,
    sql: &str,
    parameters: Vec<Value>,
) -> Result<BTreeMap<String, Vec<SupplementalContentDocument>>, RecordLoadError> {
    let mut statement = connection
        .prepare(sql)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut rows = statement
        .query(params_from_iter(parameters.iter()))
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut values: BTreeMap<String, Vec<SupplementalContentDocument>> = BTreeMap::new();
    while let Some(row) = rows
        .next()
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
    {
        let record_key = required_string(row, "record_key")?;
        let source_kind = parse_content_source_kind(&required_string(row, "source_kind")?)?;
        let visibility = parse_content_visibility(&required_string(row, "visibility")?)?;
        let content_json = required_string(row, "content_json")?;
        values
            .entry(record_key)
            .or_default()
            .push(SupplementalContentDocument {
                source_kind,
                visibility,
                contributes_to_search: required_bool(row, "contributes_to_search")?,
                contributes_to_references: required_bool(row, "contributes_to_references")?,
                label: optional_string(row, "label")?,
                document: content_document("record_content.content_json", &content_json)?,
            });
    }
    Ok(values)
}
