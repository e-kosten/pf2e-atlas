use atlas_record::{AtlasRecord, build_record_fts_projection};
use diesel::SqliteConnection;
use diesel::prelude::*;
use sha2::{Digest, Sha256};

use super::labels::{
    metric_domain_label, metric_value_parts, publication_family_label, rarity_label,
    time_kind_label, time_unit_label,
};
use super::models::{
    ActorRecordRow, ItemRecordRow, RecordContentRow, RecordMetricRow, RecordRow, RecordTraitRow,
    RecordsFtsRow, SpellRecordRow,
};
use crate::IndexWriteError;
use crate::write::visibility::RetrievalVisibility;
use atlas_record::{RecordAlias, RemasterLink};

pub(super) fn write_records(
    connection: &mut SqliteConnection,
    records: &[AtlasRecord],
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
        let traits_json = serde_json::to_string(&record.classification.traits)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
        let prerequisites_json = serde_json::to_string(&record.requirements.prerequisites)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
        let taxonomy_families_json = json_array(&record.classification.taxonomy.inferred_groups)?;
        let variant = record.variant.as_ref();
        let variant_axes_json = json_array(
            variant
                .map(|membership| membership.axes.as_slice())
                .unwrap_or_default(),
        )?;
        let activation_time = record.timing.activation_time();
        let duration = record.timing.duration_time();
        let system_actions_value = record.timing.activation_actions_value();
        let system_time_value = record.timing.activation_time_value().map(str::to_string);
        let system_duration_value = record.timing.duration_value_text().map(str::to_string);
        let item_mechanics = record.mechanics.item();
        record_rows.push(RecordRow {
            record_key: record.identity.key.to_string(),
            id: record.identity.id().as_str().to_string(),
            name: record.identity.name.clone(),
            normalized_name: record.identity.normalized_name(),
            record_kind: record.classification.kind.as_str().to_string(),
            pack_name: record.identity.pack().as_str().to_string(),
            pack_label: record.foundry.pack_label.clone(),
            foundry_document_type: record.foundry.document_type.as_str().to_string(),
            foundry_record_type: record.foundry.record_type.as_str().to_string(),
            level: record.classification.level,
            rarity: record
                .classification
                .rarity
                .map(rarity_label)
                .map(str::to_string),
            traits_json,
            prerequisites_json,
            system_category: item_mechanics.and_then(|item| item.category.clone()),
            system_group: item_mechanics.and_then(|item| item.group.clone()),
            system_base_item: item_mechanics.and_then(|item| item.base_item.clone()),
            system_usage: item_mechanics.and_then(|item| item.usage.clone()),
            system_price_json: item_mechanics.and_then(|item| item.price_json.clone()),
            system_actions_value,
            system_time_value,
            system_duration_value,
            price_cp: item_mechanics.and_then(|item| item.price_cp),
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
            publication_title: record.publication.title.clone(),
            publication_remaster: record.publication.remaster,
            publication_family: publication_family_label(record.publication.category).to_string(),
            folder_id: record.foundry.folder_id.clone(),
            taxonomy_families_json,
            variant_group_key: variant.map(|membership| membership.group_key.clone()),
            variant_base_name: variant.map(|membership| membership.base_name.clone()),
            variant_label: variant.and_then(|membership| membership.label.clone()),
            variant_axes_json,
            variant_confidence: variant.and_then(|membership| membership.confidence),
            variant_source: variant
                .map(|membership| membership.source.as_str())
                .unwrap_or("none")
                .to_string(),
            source_path: record.provenance.source_path.clone(),
            is_default_visible,
            raw_json: record.provenance.raw_json.clone().unwrap_or_default(),
        });
        let mut content_inputs = Vec::new();
        for (ordinal, content) in record.content.documents.iter().enumerate() {
            let content_json = serde_json::to_string(&content.document)
                .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
            content_inputs.push((ordinal, content, content_json));
        }
        let content_keys = allocated_content_keys(&content_inputs);
        for ((ordinal, content, content_json), content_key) in
            content_inputs.into_iter().zip(content_keys)
        {
            content_rows.push(RecordContentRow {
                record_key: record.identity.key.to_string(),
                content_key,
                ordinal: to_i64(ordinal, "record_content.ordinal")?,
                source_kind: content.source_kind.as_str().to_string(),
                visibility: content.visibility().as_str().to_string(),
                contributes_to_search: content.contributes_to_search(),
                contributes_to_references: content.contributes_to_reference_occurrences(),
                label: content.label.clone(),
                content_json,
            });
        }
        for trait_value in &record.classification.traits {
            trait_rows.push(RecordTraitRow {
                record_key: record.identity.key.to_string(),
                trait_value: trait_value.clone(),
            });
        }
        if let Some(actor_data) = record.mechanics.actor() {
            actor_rows.push(ActorRecordRow {
                record_key: record.identity.key.to_string(),
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
        if let Some(item_data) = record.mechanics.item() {
            item_rows.push(ItemRecordRow {
                record_key: record.identity.key.to_string(),
                system_category: item_data.category.clone(),
                system_base_item: item_data.base_item.clone(),
                system_group: item_data.group.clone(),
                system_usage: item_data.usage.clone(),
                system_price_json: item_data.price_json.clone(),
                price_cp: item_data.price_cp,
                bulk_value: item_data.bulk_value,
                hands_requirement: item_data.hands_requirement.clone(),
                damage_types_json: json_array(&item_data.damage_types)?,
            });
        }
        if let Some(spell_data) = record.mechanics.spell() {
            let defense = spell_data.defense.as_ref();
            spell_rows.push(SpellRecordRow {
                record_key: record.identity.key.to_string(),
                traditions_json: json_array(&spell_data.traditions)?,
                spell_kinds_json: json_array(&spell_data.kinds)?,
                range_text: spell_data.range.as_ref().map(|range| range.text.clone()),
                range_value: spell_data.range.as_ref().and_then(|range| range.distance),
                target_text: spell_data.target.as_ref().map(|target| target.text.clone()),
                area_type: spell_data.area.as_ref().and_then(|area| area.kind.clone()),
                area_value: spell_data.area.as_ref().and_then(|area| area.value),
                save_type: defense.and_then(|defense| defense.save.clone()),
                sustained: spell_data.sustained,
                basic_save: defense.is_some_and(|defense| defense.basic),
                damage_types_json: json_array(&spell_data.damage_types)?,
            });
        }
        for metric in &record.mechanics.metrics {
            let (value_type, number_value, text_value, bool_value) =
                metric_value_parts(&metric.value);
            metric_rows.push(RecordMetricRow {
                record_key: record.identity.key.to_string(),
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
                .filter(|alias| alias.canonical_record_key == record.identity.key)
                .map(|alias| alias.alias_text.clone())
                .collect::<Vec<_>>();
            let fts = build_record_fts_projection(record, &record_aliases);
            fts_rows.push(RecordsFtsRow {
                record_key: record.identity.key.to_string(),
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
    for rows in record_rows.chunks(super::INSERT_BATCH_ROWS) {
        diesel::insert_into(crate::schema::records::table)
            .values(rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    for rows in content_rows.chunks(super::INSERT_BATCH_ROWS) {
        diesel::insert_into(crate::schema::record_content::table)
            .values(rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    for rows in trait_rows.chunks(super::INSERT_BATCH_ROWS) {
        diesel::insert_into(crate::schema::record_traits::table)
            .values(rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    for rows in actor_rows.chunks(super::INSERT_BATCH_ROWS) {
        diesel::insert_into(crate::schema::actor_records::table)
            .values(rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    for rows in item_rows.chunks(super::INSERT_BATCH_ROWS) {
        diesel::insert_into(crate::schema::item_records::table)
            .values(rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    for rows in spell_rows.chunks(super::INSERT_BATCH_ROWS) {
        diesel::insert_into(crate::schema::spell_records::table)
            .values(rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    for rows in metric_rows.chunks(super::INSERT_BATCH_ROWS) {
        diesel::insert_into(crate::schema::record_metrics::table)
            .values(rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    for rows in fts_rows.chunks(super::INSERT_BATCH_ROWS) {
        diesel::insert_into(crate::schema::records_fts::table)
            .values(rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    Ok(())
}

pub(crate) fn allocated_content_keys(
    inputs: &[(usize, &atlas_record::RecordContentDocument, String)],
) -> Vec<String> {
    let bases = inputs
        .iter()
        .map(|(_, content, content_json)| {
            let mut hasher = Sha256::new();
            hasher.update(content.source_kind.as_str().as_bytes());
            hasher.update(b"\0");
            if let Some(label) = &content.label {
                hasher.update(label.as_bytes());
            }
            hasher.update(b"\0");
            hasher.update(content_json.as_bytes());
            let digest = hasher.finalize();
            let short_hash = digest[..8]
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            format!("{}:{short_hash}", content.source_kind.as_str())
        })
        .collect::<Vec<_>>();
    let mut base_counts = std::collections::BTreeMap::<String, usize>::new();
    for base in &bases {
        *base_counts.entry(base.clone()).or_insert(0) += 1;
    }
    let mut seen = std::collections::BTreeMap::<String, usize>::new();
    bases
        .into_iter()
        .map(|base| {
            if base_counts.get(&base).copied().unwrap_or(0) <= 1 {
                return base;
            }
            let ordinal = seen.entry(base.clone()).or_insert(0);
            let key = format!("{base}:{ordinal}");
            *ordinal += 1;
            key
        })
        .collect()
}

fn json_array(values: &[String]) -> Result<String, IndexWriteError> {
    serde_json::to_string(values).map_err(|error| IndexWriteError::WriteFailed(error.to_string()))
}

fn to_i64(value: usize, field: &'static str) -> Result<i64, IndexWriteError> {
    i64::try_from(value)
        .map_err(|_| IndexWriteError::WriteFailed(format!("{field} does not fit in i64")))
}
