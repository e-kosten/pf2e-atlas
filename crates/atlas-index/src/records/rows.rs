use atlas_artifact::schema::{PERSISTED_RECORD_COLUMNS, persisted_record_select_sql, records};
use atlas_domain::{PackName, RecordId, RecordKey};
use atlas_record::PersistedRecord;
use rusqlite::{Connection, Row, params_from_iter, types::Value};

use crate::RecordLoadOptions;

use super::RecordLoadError;
use super::parse::{
    bool_column, invalid_parse, json_string_array, normalized_time, optional_content_document,
    optional_f64, optional_i64, optional_string, parse_publication_family, parse_record_family,
    parse_record_key, required_string,
};

pub(super) fn read_record_rows(
    connection: &Connection,
) -> Result<Vec<PersistedRecord>, RecordLoadError> {
    let persisted_record_select_sql = persisted_record_select_sql();
    let mut statement = connection
        .prepare(&persisted_record_select_sql)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut records = Vec::new();
    let mut rows = statement
        .query([])
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    while let Some(row) = rows
        .next()
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
    {
        records.push(record_from_row(row)?);
    }
    Ok(records)
}

pub(super) fn read_record_rows_by_keys(
    connection: &Connection,
    keys: &[RecordKey],
    options: RecordLoadOptions,
) -> Result<Vec<PersistedRecord>, RecordLoadError> {
    if keys.is_empty() {
        return Ok(Vec::new());
    }
    let parameters: Vec<Value> = keys
        .iter()
        .map(|key| Value::Text(key.to_string()))
        .collect();
    let placeholders = (1..=parameters.len())
        .map(|index| format!("?{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    let columns = persisted_record_columns_sql(options);
    let sql = format!(
        "SELECT {columns} FROM {table} WHERE {record_key} IN ({placeholders}) ORDER BY {record_key}",
        table = records::TABLE.name(),
        record_key = records::columns::RECORD_KEY.name(),
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
        records.push(record_from_row(row)?);
    }
    Ok(records)
}

fn persisted_record_columns_sql(options: RecordLoadOptions) -> String {
    PERSISTED_RECORD_COLUMNS
        .iter()
        .map(|column| {
            if column == &records::columns::RAW_JSON && !options.include_raw_json {
                "'' AS raw_json".to_string()
            } else {
                column.name().to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn record_from_row(row: &Row<'_>) -> Result<PersistedRecord, RecordLoadError> {
    let record_key = required_string(row, "record_key")?;
    let id = required_string(row, "id")?;
    let record_family = required_string(row, "record_family")?;
    let pack_name = required_string(row, "pack_name")?;
    let traits_json = required_string(row, "traits_json")?;
    let prerequisites_json = required_string(row, "prerequisites_json")?;
    let activation_time_kind = optional_string(row, "activation_time_kind")?;
    let duration_kind = optional_string(row, "duration_kind")?;
    let publication_family = required_string(row, "publication_family")?;
    let taxonomy_families_json = required_string(row, "taxonomy_families_json")?;
    let variant_axes_json = required_string(row, "variant_axes_json")?;

    Ok(PersistedRecord {
        key: parse_record_key(&record_key)?,
        id: RecordId::new(id).map_err(invalid_parse("id"))?,
        name: required_string(row, "name")?,
        normalized_name: required_string(row, "normalized_name")?,
        record_family: parse_record_family(&record_family)?,
        pack_name: PackName::new(pack_name).map_err(invalid_parse("pack_name"))?,
        pack_label: required_string(row, "pack_label")?,
        foundry_document_type: required_string(row, "foundry_document_type")?,
        foundry_record_type: required_string(row, "foundry_record_type")?,
        level: optional_i64(row, "level")?,
        rarity: optional_string(row, "rarity")?,
        traits: json_string_array("records.traits_json", &traits_json)?,
        prerequisites: json_string_array("records.prerequisites_json", &prerequisites_json)?,
        system_category: optional_string(row, "system_category")?,
        system_group: optional_string(row, "system_group")?,
        system_base_item: optional_string(row, "system_base_item")?,
        system_usage: optional_string(row, "system_usage")?,
        system_price_json: optional_string(row, "system_price_json")?,
        system_actions_value: optional_i64(row, "system_actions_value")?,
        system_time_value: optional_string(row, "system_time_value")?,
        system_duration_value: optional_string(row, "system_duration_value")?,
        price_cp: optional_i64(row, "price_cp")?,
        activation_time: normalized_time(
            "activation_time",
            activation_time_kind,
            optional_i64(row, "activation_time_actions")?,
            optional_i64(row, "activation_time_duration_value")?,
            optional_string(row, "activation_time_duration_unit")?,
            optional_string(row, "activation_time_text")?,
        )?,
        duration: normalized_time(
            "duration",
            duration_kind,
            None,
            optional_i64(row, "duration_value")?,
            optional_string(row, "duration_unit")?,
            optional_string(row, "duration_text")?,
        )?,
        metrics: Vec::new(),
        actor_data: None,
        item_data: None,
        spell_data: None,
        publication_title: optional_string(row, "publication_title")?,
        publication_remaster: bool_column(
            "records.publication_remaster",
            row,
            "publication_remaster",
        )?,
        description: optional_content_document(
            "records.description_json",
            row,
            "description_json",
        )?,
        blurb: optional_content_document("records.blurb_json", row, "blurb_json")?,
        supplemental_content: Vec::new(),
        publication_family: parse_publication_family(&publication_family)?,
        folder_id: optional_string(row, "folder_id")?,
        taxonomy_families: json_string_array(
            "records.taxonomy_families_json",
            &taxonomy_families_json,
        )?,
        variant_group_key: optional_string(row, "variant_group_key")?,
        variant_base_name: optional_string(row, "variant_base_name")?,
        variant_label: optional_string(row, "variant_label")?,
        variant_axes: json_string_array("records.variant_axes_json", &variant_axes_json)?,
        variant_confidence: optional_f64(row, "variant_confidence")?,
        variant_source: required_string(row, "variant_source")?,
        source_path: required_string(row, "source_path")?,
        is_default_visible: bool_column("records.is_default_visible", row, "is_default_visible")?,
        raw_json: required_string(row, "raw_json")?,
    })
}
