use std::collections::BTreeMap;

use atlas_artifact::schema::record_content_select_sql;
use atlas_record::SupplementalContentDocument;
use rusqlite::Connection;

use super::RecordLoadError;
use super::parse::{
    content_document, optional_string, parse_content_source_kind, parse_content_visibility,
    required_bool, required_string,
};

pub(super) fn read_record_content(
    connection: &Connection,
) -> Result<BTreeMap<String, Vec<SupplementalContentDocument>>, RecordLoadError> {
    let mut statement = connection
        .prepare(&record_content_select_sql())
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut rows = statement
        .query([])
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
