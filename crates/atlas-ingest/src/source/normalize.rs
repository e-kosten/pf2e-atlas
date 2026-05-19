use std::path::Path;

use atlas_domain::{PackName, RecordId, RecordKey};
use serde_json::Value;

mod content;
mod content_diagnostics;
mod content_html;
mod content_sources;
#[cfg(test)]
mod content_tests;
mod embedded_items;
mod family;
mod journal_pages;
mod json;
mod publication;
#[cfg(test)]
mod source_facts_tests;
mod system;
mod text;
mod time;

use content_sources::extract_content_sources;
use embedded_items::{attach_embedded_content_refs, extract_embedded_item_facts};
use journal_pages::extract_journal_page_facts;

pub(crate) use content::parse_foundry_content;
pub(crate) use content_diagnostics::{ContentParseDiagnostics, DroppedContentMacro};
pub(crate) use family::classify_record;
pub(crate) use json::{
    normalized_pointer_string, pointer_bool, pointer_i64, pointer_string, string_array_at_pointer,
    string_field, typed_collection,
};
pub(crate) use publication::publication_family;
pub(crate) use system::{
    extract_damage_types, extract_disable_skills, extract_prerequisites, extract_sense_types,
    extract_speed_types, extract_traits, normalize_price_cp, parse_bulk_value,
    parse_hands_requirement,
};
pub(crate) use text::normalize_text;
pub(crate) use time::{normalize_activation_time, normalize_time_text};

use crate::error::IngestError;
use crate::records::metrics;
use crate::records::{
    LoadedSourceRecord, NormalizedRecord, SourceConstructionFacts, SourceRecordFacts,
};
use crate::source::ManifestPack;
use crate::source::side_data;

pub(crate) fn normalize_record(
    manifest_pack: &ManifestPack,
    pack_name: &PackName,
    path: &Path,
    source_root: &Path,
    raw: Value,
) -> Result<LoadedSourceRecord, IngestError> {
    let id = string_field(&raw, "_id").ok_or_else(|| normalization_error(path, "missing _id"))?;
    let name =
        string_field(&raw, "name").ok_or_else(|| normalization_error(path, "missing name"))?;
    let record_type =
        string_field(&raw, "type").unwrap_or_else(|| manifest_pack.document_type.clone());
    let id = RecordId::new(id)
        .map_err(|error| normalization_error(path, &format!("invalid _id: {error}")))?;
    let key = RecordKey::new(pack_name.clone(), id.clone());
    let normalized_name = normalize_text(&name);
    let record_family =
        classify_record(&manifest_pack.document_type, &record_type).ok_or_else(|| {
            normalization_error(
                path,
                &format!(
                    "unsupported Foundry record taxonomy: {}|{}",
                    manifest_pack.document_type, record_type
                ),
            )
        })?;
    let level = pointer_i64(&raw, "/system/level/value").or_else(|| {
        (manifest_pack.document_type == "Actor")
            .then(|| pointer_i64(&raw, "/system/details/level/value"))
            .flatten()
    });
    let rarity = normalized_pointer_string(&raw, "/system/traits/rarity");
    let traits = extract_traits(&raw);
    let prerequisites = extract_prerequisites(&raw);
    let system_category = normalized_pointer_string(&raw, "/system/category");
    let system_group = normalized_pointer_string(&raw, "/system/group");
    let system_base_item = normalized_pointer_string(&raw, "/system/baseItem");
    let system_usage = normalized_pointer_string(&raw, "/system/usage/value");
    let system_price_json = raw
        .pointer("/system/price/value")
        .map(serde_json::to_string)
        .transpose()
        .map_err(|error| normalization_error(path, &format!("price JSON failed: {error}")))?;
    let system_actions_value = pointer_i64(&raw, "/system/actions/value");
    let system_time_value = normalized_pointer_string(&raw, "/system/time/value");
    let system_duration_value = normalized_pointer_string(&raw, "/system/duration/value");
    let price_cp = normalize_price_cp(raw.pointer("/system/price/value"));
    let activation_time =
        normalize_activation_time(system_actions_value, system_time_value.as_deref());
    let duration = system_duration_value
        .as_deref()
        .and_then(normalize_time_text);
    let metrics = metrics::extract_metrics(&raw, &manifest_pack.document_type, &record_type)
        .map_err(|message| normalization_error(path, &message))?;
    let actor_data =
        (manifest_pack.document_type == "Actor").then(|| side_data::extract_actor_side_data(&raw));
    let item_data = (manifest_pack.document_type == "Item").then(|| {
        side_data::extract_item_side_data(
            &raw,
            system_category.clone(),
            system_base_item.clone(),
            system_group.clone(),
            system_usage.clone(),
            price_cp,
        )
    });
    let spell_data = (manifest_pack.document_type == "Item" && record_type == "spell")
        .then(|| side_data::extract_spell_side_data(&raw, &traits));
    let publication_title = pointer_string(&raw, "/system/publication/title")
        .or_else(|| pointer_string(&raw, "/system/details/publication/title"));
    let publication_remaster = pointer_bool(&raw, "/system/publication/remaster")
        .or_else(|| pointer_bool(&raw, "/system/details/publication/remaster"))
        .unwrap_or(false);
    let content_sources = extract_content_sources(&raw);
    let mut source_facts = SourceRecordFacts {
        slug: normalized_pointer_string(&raw, "/system/slug"),
        compendium_source: normalized_pointer_string(&raw, "/_stats/compendiumSource"),
        ..SourceRecordFacts::default()
    };
    for (local_key, document) in &content_sources.supplemental_content {
        if let Some(local_key) = local_key {
            source_facts
                .source_content
                .insert(local_key.clone(), document.clone());
        }
    }
    let mut supplemental_content = content_sources
        .supplemental_content
        .into_iter()
        .map(|(_, document)| document)
        .collect::<Vec<_>>();
    source_facts.embedded_items = extract_embedded_item_facts(&raw, &key);
    attach_embedded_content_refs(
        &mut source_facts.embedded_items,
        &source_facts.source_content,
    );
    let (journal_pages, skipped_journal_pages, journal_diagnostics) =
        extract_journal_page_facts(&raw, &key);
    source_facts.journal_pages = journal_pages;
    source_facts.skipped_journal_pages = skipped_journal_pages;
    let folder_id = pointer_string(&raw, "/folder");
    let source_path = path
        .strip_prefix(source_root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();
    let raw_json = serde_json::to_string(&raw).map_err(|error| {
        normalization_error(path, &format!("raw JSON serialization failed: {error}"))
    })?;
    let publication_family = publication_family(pack_name.as_str(), publication_title.as_deref());

    let record = NormalizedRecord {
        key,
        id,
        name,
        normalized_name,
        record_family,
        pack_name: pack_name.clone(),
        pack_label: manifest_pack.label.clone(),
        foundry_document_type: manifest_pack.document_type.clone(),
        foundry_record_type: record_type,
        level,
        rarity,
        traits,
        prerequisites,
        system_category,
        system_group,
        system_base_item,
        system_usage,
        system_price_json,
        system_actions_value,
        system_time_value,
        system_duration_value,
        price_cp,
        activation_time,
        duration,
        metrics,
        actor_data,
        item_data,
        spell_data,
        publication_title,
        publication_remaster,
        description: content_sources.description,
        blurb: content_sources.blurb,
        supplemental_content: std::mem::take(&mut supplemental_content),
        publication_family,
        folder_id,
        taxonomy_families: Vec::new(),
        variant_group_key: None,
        variant_base_name: None,
        variant_label: None,
        variant_axes: Vec::new(),
        variant_confidence: None,
        variant_source: "none".to_string(),
        source_path,
        is_default_visible: true,
        raw_json,
    };
    let facts = SourceConstructionFacts {
        content_parse_diagnostics: content_sources
            .diagnostics
            .into_iter()
            .chain(journal_diagnostics)
            .collect(),
        source_facts,
    };

    Ok(LoadedSourceRecord::new(record, facts))
}

pub(crate) fn normalization_error(path: &Path, message: &str) -> IngestError {
    IngestError::RecordNormalizationFailed {
        path: path.display().to_string(),
        message: message.to_string(),
    }
}
