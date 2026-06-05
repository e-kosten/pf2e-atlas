use crate::schema::records;
use atlas_domain::RecordKey;
use atlas_record::{
    ActivationTimeSourceField, AtlasRecord, DurationTimeSourceField, FoundryDocumentType,
    FoundryRecordInfo, FoundryRecordType, RecordActivationTiming, RecordClassification,
    RecordContent, RecordDurationTiming, RecordIdentity, RecordMechanics, RecordProvenance,
    RecordPublication, RecordRequirements, RecordTaxonomy, RecordTiming, RecordVariantMembership,
    RecordVisibility, RecordVisibilityReason, VariantSource,
};
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use diesel::{Queryable, Selectable, SelectableHelper, SqliteConnection};

use super::RecordLoadError;
use super::parse::{
    json_string_array, normalized_time, parse_publication_family, parse_rarity, parse_record_key,
    parse_record_kind, parse_variant_source,
};

pub(super) fn read_record_rows(
    connection: &mut SqliteConnection,
) -> Result<Vec<AtlasRecord>, RecordLoadError> {
    let rows = records::table
        .select(RecordRow::as_select())
        .order(records::record_key.asc())
        .load::<RecordRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    rows.into_iter().map(record_from_row).collect()
}

pub(super) fn read_record_rows_by_keys(
    connection: &mut SqliteConnection,
    keys: &[RecordKey],
) -> Result<Vec<AtlasRecord>, RecordLoadError> {
    if keys.is_empty() {
        return Ok(Vec::new());
    }
    let key_strings = keys.iter().map(ToString::to_string).collect::<Vec<_>>();
    let rows = records::table
        .filter(records::record_key.eq_any(key_strings))
        .select(RecordRow::as_select())
        .order(records::record_key.asc())
        .load::<RecordRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    rows.into_iter().map(record_from_row).collect()
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = records)]
#[diesel(check_for_backend(Sqlite))]
struct RecordRow {
    record_key: String,
    name: String,
    record_kind: String,
    pack_label: String,
    foundry_document_type: String,
    foundry_record_type: String,
    level: Option<i64>,
    rarity: Option<String>,
    traits_json: String,
    prerequisites_json: String,
    system_actions_value: Option<i64>,
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

fn record_from_row(row: RecordRow) -> Result<AtlasRecord, RecordLoadError> {
    let variant_group = variant_group_from_row(&row)?;
    let activation_time = normalized_time(
        "activation_time",
        row.activation_time_kind,
        row.activation_time_actions,
        row.activation_time_duration_value,
        row.activation_time_duration_unit,
        row.activation_time_text,
    )?;
    let activation = activation_time.map(|time| {
        let source_field = if row.system_actions_value.is_some() {
            ActivationTimeSourceField::ActionsValue
        } else {
            ActivationTimeSourceField::TimeValue
        };
        RecordActivationTiming { time, source_field }
    });
    let duration_time = normalized_time(
        "duration",
        row.duration_kind,
        None,
        row.duration_value,
        row.duration_unit,
        row.duration_text,
    )?;
    let duration = duration_time.map(|time| RecordDurationTiming {
        time,
        source_field: DurationTimeSourceField::DurationValue,
    });
    Ok(AtlasRecord {
        identity: RecordIdentity {
            key: parse_record_key(&row.record_key)?,
            name: row.name,
        },
        classification: RecordClassification {
            kind: parse_record_kind(&row.record_kind)?,
            level: row.level,
            rarity: row.rarity.as_deref().map(parse_rarity).transpose()?,
            traits: json_string_array("records.traits_json", &row.traits_json)?,
            taxonomy: RecordTaxonomy {
                inferred_groups: json_string_array(
                    "records.taxonomy_families_json",
                    &row.taxonomy_families_json,
                )?,
            },
        },
        foundry: FoundryRecordInfo {
            pack_label: row.pack_label,
            document_type: FoundryDocumentType::from_foundry(&row.foundry_document_type),
            record_type: FoundryRecordType::from_foundry(&row.foundry_record_type),
            folder_id: row.folder_id,
        },
        provenance: RecordProvenance {
            source_path: row.source_path,
            raw_json: Some(row.raw_json),
        },
        publication: RecordPublication {
            title: row.publication_title,
            remaster: row.publication_remaster,
            category: parse_publication_family(&row.publication_family)?,
        },
        requirements: RecordRequirements {
            prerequisites: json_string_array(
                "records.prerequisites_json",
                &row.prerequisites_json,
            )?,
        },
        timing: RecordTiming {
            activation,
            duration,
        },
        mechanics: RecordMechanics::default(),
        content: RecordContent {
            documents: Vec::new(),
        },
        variant: variant_group,
        visibility: if row.is_default_visible {
            RecordVisibility::visible(RecordVisibilityReason::SourceRecord)
        } else {
            RecordVisibility::hidden(RecordVisibilityReason::SourceRecord)
        },
    })
}

fn variant_group_from_row(
    row: &RecordRow,
) -> Result<Option<RecordVariantMembership>, RecordLoadError> {
    let axes = json_string_array("records.variant_axes_json", &row.variant_axes_json)?;
    let source = parse_variant_source(&row.variant_source)?;
    let has_variant_signal = row.variant_group_key.is_some()
        || row.variant_base_name.is_some()
        || row.variant_label.is_some()
        || row.variant_confidence.is_some()
        || !axes.is_empty()
        || source != VariantSource::None;
    if !has_variant_signal {
        return Ok(None);
    }
    let key = row
        .variant_group_key
        .clone()
        .ok_or_else(|| missing_variant_field("records.variant_group_key"))?;
    let base_name = row
        .variant_base_name
        .clone()
        .ok_or_else(|| missing_variant_field("records.variant_base_name"))?;
    Ok(Some(RecordVariantMembership {
        group_key: key,
        base_name,
        label: row.variant_label.clone(),
        axes,
        confidence: row.variant_confidence,
        source,
    }))
}

fn missing_variant_field(field: &'static str) -> RecordLoadError {
    RecordLoadError::InvalidData(format!(
        "{field} is required when records contain variant membership data"
    ))
}
