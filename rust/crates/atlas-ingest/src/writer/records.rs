use std::collections::BTreeSet;

use atlas_artifact::schema::record_insert_sql;
use rusqlite::{Connection, params};

use super::labels::{
    metric_domain_label, metric_value_parts, publication_family_label, time_kind_label,
    time_unit_label,
};
use crate::{IngestError, LoadedRecord, RemasterLink};

pub(super) fn write_records(
    connection: &Connection,
    records: &[LoadedRecord],
    remaster_links: &[RemasterLink],
) -> Result<(), IngestError> {
    let hidden_record_keys = remaster_links
        .iter()
        .map(|link| link.legacy_record_key.to_string())
        .collect::<BTreeSet<_>>();
    let record_insert_sql = record_insert_sql();
    let mut insert_record = connection
        .prepare(&record_insert_sql)
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let mut insert_trait = connection
        .prepare("INSERT INTO record_traits (record_key, trait) VALUES (?1, ?2)")
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let mut insert_metric = connection
        .prepare(
            "INSERT INTO record_metrics (
              record_key, metric_domain, metric_key, value_type, number_value, text_value, bool_value
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let mut insert_actor = connection
        .prepare(
            "INSERT INTO actor_records (
              record_key, size, languages_json, speed_types_json, senses_json, immunities_json,
              resistances_json, weaknesses_json, disable_text, disable_skills_json, is_complex
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let mut insert_item = connection
        .prepare(
            "INSERT INTO item_records (
              record_key, system_category, system_base_item, system_group, system_usage, price_cp,
              bulk_value, hands_requirement, damage_types_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let mut insert_spell = connection
        .prepare(
            "INSERT INTO spell_records (
              record_key, traditions_json, spell_kinds_json, range_text, range_value, target_text,
              area_type, area_value, save_type, sustained, basic_save, damage_types_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let mut insert_fts = connection
        .prepare("INSERT INTO records_fts (record_key, name, search_text_projection) VALUES (?1, ?2, ?3)")
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;

    for record in records {
        let is_default_visible =
            record.is_default_visible && !hidden_record_keys.contains(&record.key.to_string());
        let traits_json = serde_json::to_string(&record.traits)
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        let taxonomy_families_json = json_array(&record.taxonomy_families)?;
        let variant_axes_json = json_array(&record.variant_axes)?;
        let activation_time = record.activation_time.as_ref();
        let duration = record.duration.as_ref();
        insert_record
            .execute(params![
                record.key.to_string(),
                record.id.as_str(),
                record.name.as_str(),
                record.normalized_name.as_str(),
                record.record_family.as_str(),
                record.pack_name.as_str(),
                record.pack_label.as_str(),
                record.foundry_document_type.as_str(),
                record.foundry_record_type.as_str(),
                record.level,
                record.rarity.as_deref(),
                traits_json,
                record.system_category.as_deref(),
                record.system_group.as_deref(),
                record.system_base_item.as_deref(),
                record.system_usage.as_deref(),
                record.system_price_json.as_deref(),
                record.system_actions_value,
                record.system_time_value.as_deref(),
                record.system_duration_value.as_deref(),
                record.price_cp,
                activation_time.map(|time| time_kind_label(time.kind)),
                activation_time.and_then(|time| time.actions),
                activation_time.and_then(|time| time.duration_value),
                activation_time.and_then(|time| time.duration_unit.map(time_unit_label)),
                activation_time.map(|time| time.text.as_str()),
                duration.map(|time| time_kind_label(time.kind)),
                duration.and_then(|time| time.duration_value),
                duration.and_then(|time| time.duration_unit.map(time_unit_label)),
                duration.map(|time| time.text.as_str()),
                record.publication_title.as_deref(),
                i64::from(record.publication_remaster),
                record.description_text.as_deref(),
                record.blurb_text.as_deref(),
                record.description_text.as_deref(),
                publication_family_label(record.publication_family),
                record.folder_id.as_deref(),
                taxonomy_families_json,
                record.variant_group_key.as_deref(),
                record.variant_base_name.as_deref(),
                record.variant_label.as_deref(),
                variant_axes_json,
                record.variant_confidence,
                record.variant_source.as_str(),
                record.source_path.as_str(),
                i64::from(is_default_visible),
                record.search_text_projection.as_str(),
                record.raw_json.as_str(),
            ])
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        for trait_value in &record.traits {
            insert_trait
                .execute((record.key.to_string(), trait_value.as_str()))
                .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        }
        if let Some(actor_data) = &record.actor_data {
            insert_actor
                .execute(params![
                    record.key.to_string(),
                    actor_data.size.as_deref(),
                    json_array(&actor_data.languages)?,
                    json_array(&actor_data.speed_types)?,
                    json_array(&actor_data.senses)?,
                    json_array(&actor_data.immunities)?,
                    json_array(&actor_data.resistances)?,
                    json_array(&actor_data.weaknesses)?,
                    actor_data.disable_text.as_deref(),
                    json_array(&actor_data.disable_skills)?,
                    i64::from(actor_data.is_complex),
                ])
                .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        }
        if let Some(item_data) = &record.item_data {
            insert_item
                .execute(params![
                    record.key.to_string(),
                    item_data.system_category.as_deref(),
                    item_data.system_base_item.as_deref(),
                    item_data.system_group.as_deref(),
                    item_data.system_usage.as_deref(),
                    item_data.price_cp,
                    item_data.bulk_value,
                    item_data.hands_requirement.as_deref(),
                    json_array(&item_data.damage_types)?,
                ])
                .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        }
        if let Some(spell_data) = &record.spell_data {
            insert_spell
                .execute(params![
                    record.key.to_string(),
                    json_array(&spell_data.traditions)?,
                    json_array(&spell_data.spell_kinds)?,
                    spell_data.range_text.as_deref(),
                    spell_data.range_value,
                    spell_data.target_text.as_deref(),
                    spell_data.area_type.as_deref(),
                    spell_data.area_value,
                    spell_data.save_type.as_deref(),
                    i64::from(spell_data.sustained),
                    i64::from(spell_data.basic_save),
                    json_array(&spell_data.damage_types)?,
                ])
                .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        }
        for metric in &record.metrics {
            let (value_type, number_value, text_value, bool_value) =
                metric_value_parts(&metric.value);
            insert_metric
                .execute(params![
                    record.key.to_string(),
                    metric_domain_label(metric.domain),
                    metric.key.as_str(),
                    value_type,
                    number_value,
                    text_value,
                    bool_value,
                ])
                .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        }
        if is_default_visible {
            insert_fts
                .execute((
                    record.key.to_string(),
                    record.name.as_str(),
                    record.search_text_projection.as_str(),
                ))
                .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        }
    }
    Ok(())
}

fn json_array(values: &[String]) -> Result<String, IngestError> {
    serde_json::to_string(values)
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))
}
