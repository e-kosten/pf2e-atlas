use atlas_artifact::schema::{
    actor_record_insert_sql, item_record_insert_sql, record_content_insert_sql, record_insert_sql,
    record_metric_insert_sql, record_trait_insert_sql, records_fts_insert_sql,
    spell_record_insert_sql,
};
use atlas_record::{NormalizedRecord, build_record_fts_projection};
use rusqlite::{Connection, params};

use super::labels::{
    metric_domain_label, metric_value_parts, publication_family_label, time_kind_label,
    time_unit_label,
};
use crate::IndexWriteError;
use crate::writer_visibility::RetrievalVisibility;
use atlas_record::{RecordAlias, RemasterLink};

pub(super) fn write_records(
    connection: &Connection,
    records: &[&NormalizedRecord],
    aliases: &[RecordAlias],
    remaster_links: &[RemasterLink],
) -> Result<(), IndexWriteError> {
    let retrieval_visibility = RetrievalVisibility::from_remaster_links(remaster_links);
    let record_insert_sql = record_insert_sql();
    let record_trait_insert_sql = record_trait_insert_sql();
    let record_content_insert_sql = record_content_insert_sql();
    let record_metric_insert_sql = record_metric_insert_sql();
    let actor_record_insert_sql = actor_record_insert_sql();
    let item_record_insert_sql = item_record_insert_sql();
    let spell_record_insert_sql = spell_record_insert_sql();
    let records_fts_insert_sql = records_fts_insert_sql();
    let mut insert_record = connection
        .prepare(&record_insert_sql)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    let mut insert_trait = connection
        .prepare(&record_trait_insert_sql)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    let mut insert_content = connection
        .prepare(&record_content_insert_sql)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    let mut insert_metric = connection
        .prepare(&record_metric_insert_sql)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    let mut insert_actor = connection
        .prepare(&actor_record_insert_sql)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    let mut insert_item = connection
        .prepare(&item_record_insert_sql)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    let mut insert_spell = connection
        .prepare(&spell_record_insert_sql)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    let mut insert_fts = connection
        .prepare(&records_fts_insert_sql)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;

    for record in records {
        let is_default_visible = retrieval_visibility.is_default_visible(record);
        let traits_json = serde_json::to_string(&record.traits)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
        let prerequisites_json = serde_json::to_string(&record.prerequisites)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
        let taxonomy_families_json = json_array(&record.taxonomy_families)?;
        let variant_axes_json = json_array(&record.variant_axes)?;
        let description_json = optional_json(&record.description)?;
        let blurb_json = optional_json(&record.blurb)?;
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
                prerequisites_json,
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
                description_json.as_deref(),
                blurb_json.as_deref(),
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
                record.raw_json.as_str(),
            ])
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
        for (ordinal, supplemental) in record.supplemental_content.iter().enumerate() {
            let content_json = serde_json::to_string(&supplemental.document)
                .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
            insert_content
                .execute(params![
                    record.key.to_string(),
                    ordinal as i64,
                    supplemental.source_kind.as_str(),
                    supplemental.visibility.as_str(),
                    i64::from(supplemental.contributes_to_search),
                    i64::from(supplemental.contributes_to_references),
                    supplemental.label.as_deref(),
                    content_json,
                ])
                .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
        }
        for trait_value in &record.traits {
            insert_trait
                .execute((record.key.to_string(), trait_value.as_str()))
                .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
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
                .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
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
                .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
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
                .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
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
                .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
        }
        if is_default_visible {
            let record_aliases = aliases
                .iter()
                .filter(|alias| alias.canonical_record_key == record.key)
                .map(|alias| alias.alias_text.clone())
                .collect::<Vec<_>>();
            let fts = build_record_fts_projection(record, &record_aliases);
            insert_fts
                .execute(params![
                    record.key.to_string(),
                    fts.title,
                    fts.aliases,
                    fts.traits,
                    fts.taxonomy_terms,
                    fts.constraint_terms,
                    fts.mechanic_terms,
                    fts.source_terms,
                    fts.metric_terms,
                    fts.headings,
                    fts.body,
                    fts.facts,
                    fts.references,
                    fts.embedded_content,
                ])
                .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
        }
    }
    Ok(())
}

fn json_array(values: &[String]) -> Result<String, IndexWriteError> {
    serde_json::to_string(values).map_err(|error| IndexWriteError::WriteFailed(error.to_string()))
}

fn optional_json<T: serde::Serialize>(
    value: &Option<T>,
) -> Result<Option<String>, IndexWriteError> {
    value
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))
}
