use crate::schema::records;
use atlas_domain::{PackName, RecordId, RecordKey};
use atlas_record::PersistedRecord;
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use diesel::{Queryable, Selectable, SelectableHelper, SqliteConnection};

use super::RecordLoadError;
use super::parse::{
    content_document, invalid_parse, json_string_array, normalized_time, parse_publication_family,
    parse_record_family, parse_record_key,
};

pub(super) fn read_record_rows(
    connection: &mut SqliteConnection,
) -> Result<Vec<PersistedRecord>, RecordLoadError> {
    let rows = records::table
        .select(PersistedRecordRow::as_select())
        .order(records::record_key.asc())
        .load::<PersistedRecordRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    rows.into_iter().map(record_from_row).collect()
}

pub(super) fn read_record_rows_by_keys(
    connection: &mut SqliteConnection,
    keys: &[RecordKey],
) -> Result<Vec<PersistedRecord>, RecordLoadError> {
    if keys.is_empty() {
        return Ok(Vec::new());
    }
    let key_strings = keys.iter().map(ToString::to_string).collect::<Vec<_>>();
    let rows = records::table
        .filter(records::record_key.eq_any(key_strings))
        .select(PersistedRecordRow::as_select())
        .order(records::record_key.asc())
        .load::<PersistedRecordRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    rows.into_iter().map(record_from_row).collect()
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = records)]
#[diesel(check_for_backend(Sqlite))]
struct PersistedRecordRow {
    record_key: String,
    id: String,
    name: String,
    normalized_name: String,
    record_family: String,
    pack_name: String,
    pack_label: String,
    foundry_document_type: String,
    foundry_record_type: String,
    level: Option<i64>,
    rarity: Option<String>,
    traits_json: String,
    prerequisites_json: String,
    system_category: Option<String>,
    system_group: Option<String>,
    system_base_item: Option<String>,
    system_usage: Option<String>,
    system_price_json: Option<String>,
    system_actions_value: Option<i64>,
    system_time_value: Option<String>,
    system_duration_value: Option<String>,
    price_cp: Option<i64>,
    activation_time_kind: Option<String>,
    activation_time_actions: Option<i64>,
    activation_time_duration_value: Option<i64>,
    activation_time_duration_unit: Option<String>,
    activation_time_text: Option<String>,
    duration_kind: Option<String>,
    duration_value: Option<i64>,
    duration_unit: Option<String>,
    duration_text: Option<String>,
    publication_title: Option<String>,
    publication_remaster: bool,
    description_json: Option<String>,
    blurb_json: Option<String>,
    publication_family: String,
    folder_id: Option<String>,
    taxonomy_families_json: String,
    variant_group_key: Option<String>,
    variant_base_name: Option<String>,
    variant_label: Option<String>,
    variant_axes_json: String,
    variant_confidence: Option<f64>,
    variant_source: String,
    source_path: String,
    is_default_visible: bool,
    raw_json: String,
}

fn record_from_row(row: PersistedRecordRow) -> Result<PersistedRecord, RecordLoadError> {
    Ok(PersistedRecord {
        key: parse_record_key(&row.record_key)?,
        id: RecordId::new(row.id).map_err(invalid_parse("id"))?,
        name: row.name,
        normalized_name: row.normalized_name,
        record_family: parse_record_family(&row.record_family)?,
        pack_name: PackName::new(row.pack_name).map_err(invalid_parse("pack_name"))?,
        pack_label: row.pack_label,
        foundry_document_type: row.foundry_document_type,
        foundry_record_type: row.foundry_record_type,
        level: row.level,
        rarity: row.rarity,
        traits: json_string_array("records.traits_json", &row.traits_json)?,
        prerequisites: json_string_array("records.prerequisites_json", &row.prerequisites_json)?,
        system_category: row.system_category,
        system_group: row.system_group,
        system_base_item: row.system_base_item,
        system_usage: row.system_usage,
        system_price_json: row.system_price_json,
        system_actions_value: row.system_actions_value,
        system_time_value: row.system_time_value,
        system_duration_value: row.system_duration_value,
        price_cp: row.price_cp,
        activation_time: normalized_time(
            "activation_time",
            row.activation_time_kind,
            row.activation_time_actions,
            row.activation_time_duration_value,
            row.activation_time_duration_unit,
            row.activation_time_text,
        )?,
        duration: normalized_time(
            "duration",
            row.duration_kind,
            None,
            row.duration_value,
            row.duration_unit,
            row.duration_text,
        )?,
        metrics: Vec::new(),
        actor_data: None,
        item_data: None,
        spell_data: None,
        publication_title: row.publication_title,
        publication_remaster: row.publication_remaster,
        description: row
            .description_json
            .as_deref()
            .map(|value| content_document("records.description_json", value))
            .transpose()?,
        blurb: row
            .blurb_json
            .as_deref()
            .map(|value| content_document("records.blurb_json", value))
            .transpose()?,
        supplemental_content: Vec::new(),
        publication_family: parse_publication_family(&row.publication_family)?,
        folder_id: row.folder_id,
        taxonomy_families: json_string_array(
            "records.taxonomy_families_json",
            &row.taxonomy_families_json,
        )?,
        variant_group_key: row.variant_group_key,
        variant_base_name: row.variant_base_name,
        variant_label: row.variant_label,
        variant_axes: json_string_array("records.variant_axes_json", &row.variant_axes_json)?,
        variant_confidence: row.variant_confidence,
        variant_source: row.variant_source,
        source_path: row.source_path,
        is_default_visible: row.is_default_visible,
        raw_json: row.raw_json,
    })
}
