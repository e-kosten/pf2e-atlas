use atlas_record::{NormalizedRecord, build_record_fts_projection};
use diesel::SqliteConnection;
use diesel::prelude::*;

use super::labels::{
    metric_domain_label, metric_value_parts, publication_family_label, time_kind_label,
    time_unit_label,
};
use super::models::{
    ActorRecordRow, ItemRecordRow, RecordContentRow, RecordMetricRow, RecordRow, RecordTraitRow,
    RecordsFtsRow, SpellRecordRow,
};
use crate::IndexWriteError;
use crate::writer_visibility::RetrievalVisibility;
use atlas_record::{RecordAlias, RemasterLink};

pub(super) fn write_records(
    connection: &mut SqliteConnection,
    records: &[&NormalizedRecord],
    aliases: &[RecordAlias],
    remaster_links: &[RemasterLink],
) -> Result<(), IndexWriteError> {
    let retrieval_visibility = RetrievalVisibility::from_remaster_links(remaster_links);
    let mut record_rows = Vec::new();
    let mut content_rows = Vec::new();
    let mut trait_rows = Vec::new();
    let mut actor_rows = Vec::new();
    let mut item_rows = Vec::new();
    let mut spell_rows = Vec::new();
    let mut metric_rows = Vec::new();
    let mut fts_rows = Vec::new();
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
        record_rows.push(RecordRow {
            record_key: record.key.to_string(),
            id: record.id.as_str().to_string(),
            name: record.name.clone(),
            normalized_name: record.normalized_name.clone(),
            record_family: record.record_family.as_str().to_string(),
            pack_name: record.pack_name.as_str().to_string(),
            pack_label: record.pack_label.clone(),
            foundry_document_type: record.foundry_document_type.clone(),
            foundry_record_type: record.foundry_record_type.clone(),
            level: record.level,
            rarity: record.rarity.clone(),
            traits_json,
            prerequisites_json,
            system_category: record.system_category.clone(),
            system_group: record.system_group.clone(),
            system_base_item: record.system_base_item.clone(),
            system_usage: record.system_usage.clone(),
            system_price_json: record.system_price_json.clone(),
            system_actions_value: record.system_actions_value,
            system_time_value: record.system_time_value.clone(),
            system_duration_value: record.system_duration_value.clone(),
            price_cp: record.price_cp,
            activation_time_kind: activation_time
                .map(|time| time_kind_label(time.kind).to_string()),
            activation_time_actions: activation_time.and_then(|time| time.actions),
            activation_time_duration_value: activation_time.and_then(|time| time.duration_value),
            activation_time_duration_unit: activation_time
                .and_then(|time| time.duration_unit.map(time_unit_label))
                .map(str::to_string),
            activation_time_text: activation_time.map(|time| time.text.clone()),
            duration_kind: duration.map(|time| time_kind_label(time.kind).to_string()),
            duration_value: duration.and_then(|time| time.duration_value),
            duration_unit: duration
                .and_then(|time| time.duration_unit.map(time_unit_label))
                .map(str::to_string),
            duration_text: duration.map(|time| time.text.clone()),
            publication_title: record.publication_title.clone(),
            publication_remaster: record.publication_remaster,
            description_json,
            blurb_json,
            publication_family: publication_family_label(record.publication_family).to_string(),
            folder_id: record.folder_id.clone(),
            taxonomy_families_json,
            variant_group_key: record.variant_group_key.clone(),
            variant_base_name: record.variant_base_name.clone(),
            variant_label: record.variant_label.clone(),
            variant_axes_json,
            variant_confidence: record.variant_confidence,
            variant_source: record.variant_source.clone(),
            source_path: record.source_path.clone(),
            is_default_visible,
            raw_json: record.raw_json.clone(),
        });
        for (ordinal, supplemental) in record.supplemental_content.iter().enumerate() {
            let content_json = serde_json::to_string(&supplemental.document)
                .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
            content_rows.push(RecordContentRow {
                record_key: record.key.to_string(),
                content_key: supplemental_content_key(ordinal),
                ordinal: to_i64(ordinal, "record_content.ordinal")?,
                source_kind: supplemental.source_kind.as_str().to_string(),
                visibility: supplemental.visibility.as_str().to_string(),
                contributes_to_search: supplemental.contributes_to_search,
                contributes_to_references: supplemental.contributes_to_references,
                label: supplemental.label.clone(),
                content_json,
            });
        }
        for trait_value in &record.traits {
            trait_rows.push(RecordTraitRow {
                record_key: record.key.to_string(),
                trait_value: trait_value.clone(),
            });
        }
        if let Some(actor_data) = &record.actor_data {
            actor_rows.push(ActorRecordRow {
                record_key: record.key.to_string(),
                size: actor_data.size.clone(),
                languages_json: json_array(&actor_data.languages)?,
                speed_types_json: json_array(&actor_data.speed_types)?,
                senses_json: json_array(&actor_data.senses)?,
                immunities_json: json_array(&actor_data.immunities)?,
                resistances_json: json_array(&actor_data.resistances)?,
                weaknesses_json: json_array(&actor_data.weaknesses)?,
                disable_text: actor_data.disable_text.clone(),
                disable_skills_json: json_array(&actor_data.disable_skills)?,
                is_complex: actor_data.is_complex,
            });
        }
        if let Some(item_data) = &record.item_data {
            item_rows.push(ItemRecordRow {
                record_key: record.key.to_string(),
                system_category: item_data.system_category.clone(),
                system_base_item: item_data.system_base_item.clone(),
                system_group: item_data.system_group.clone(),
                system_usage: item_data.system_usage.clone(),
                price_cp: item_data.price_cp,
                bulk_value: item_data.bulk_value,
                hands_requirement: item_data.hands_requirement.clone(),
                damage_types_json: json_array(&item_data.damage_types)?,
            });
        }
        if let Some(spell_data) = &record.spell_data {
            spell_rows.push(SpellRecordRow {
                record_key: record.key.to_string(),
                traditions_json: json_array(&spell_data.traditions)?,
                spell_kinds_json: json_array(&spell_data.spell_kinds)?,
                range_text: spell_data.range_text.clone(),
                range_value: spell_data.range_value,
                target_text: spell_data.target_text.clone(),
                area_type: spell_data.area_type.clone(),
                area_value: spell_data.area_value,
                save_type: spell_data.save_type.clone(),
                sustained: spell_data.sustained,
                basic_save: spell_data.basic_save,
                damage_types_json: json_array(&spell_data.damage_types)?,
            });
        }
        for metric in &record.metrics {
            let (value_type, number_value, text_value, bool_value) =
                metric_value_parts(&metric.value);
            metric_rows.push(RecordMetricRow {
                record_key: record.key.to_string(),
                metric_domain: metric_domain_label(metric.domain).to_string(),
                metric_key: metric.key.clone(),
                value_type: value_type.to_string(),
                number_value,
                text_value: text_value.map(str::to_string),
                bool_value: bool_value.map(|value| value != 0),
            });
        }
        if is_default_visible {
            let record_aliases = aliases
                .iter()
                .filter(|alias| alias.canonical_record_key == record.key)
                .map(|alias| alias.alias_text.clone())
                .collect::<Vec<_>>();
            let fts = build_record_fts_projection(record, &record_aliases);
            fts_rows.push(RecordsFtsRow {
                record_key: record.key.to_string(),
                title: Some(fts.title),
                aliases: Some(fts.aliases),
                traits: Some(fts.traits),
                taxonomy_terms: Some(fts.taxonomy_terms),
                constraint_terms: Some(fts.constraint_terms),
                mechanic_terms: Some(fts.mechanic_terms),
                source_terms: Some(fts.source_terms),
                metric_terms: Some(fts.metric_terms),
                headings: Some(fts.headings),
                body: Some(fts.body),
                facts: Some(fts.facts),
                reference_terms: Some(fts.references),
                embedded_content: Some(fts.embedded_content),
            });
        }
    }
    if !record_rows.is_empty() {
        diesel::insert_into(crate::schema::records::table)
            .values(&record_rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    if !content_rows.is_empty() {
        diesel::insert_into(crate::schema::record_content::table)
            .values(&content_rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    if !trait_rows.is_empty() {
        diesel::insert_into(crate::schema::record_traits::table)
            .values(&trait_rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    if !actor_rows.is_empty() {
        diesel::insert_into(crate::schema::actor_records::table)
            .values(&actor_rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    if !item_rows.is_empty() {
        diesel::insert_into(crate::schema::item_records::table)
            .values(&item_rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    if !spell_rows.is_empty() {
        diesel::insert_into(crate::schema::spell_records::table)
            .values(&spell_rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    if !metric_rows.is_empty() {
        diesel::insert_into(crate::schema::record_metrics::table)
            .values(&metric_rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    if !fts_rows.is_empty() {
        diesel::insert_into(crate::schema::records_fts::table)
            .values(&fts_rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    Ok(())
}

pub(crate) fn supplemental_content_key(ordinal: usize) -> String {
    format!("content:{ordinal}")
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

fn to_i64(value: usize, field: &'static str) -> Result<i64, IndexWriteError> {
    i64::try_from(value)
        .map_err(|_| IndexWriteError::WriteFailed(format!("{field} does not fit in i64")))
}
