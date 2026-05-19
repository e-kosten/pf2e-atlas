use atlas_domain::{PackName, RecordFamily, RecordId};
use serde_json::{Value, json};

use crate::diagnostics::{
    DERIVED_AFFLICTION_INSTANCES_PACK_LABEL, DERIVED_AFFLICTION_INSTANCES_PACK_NAME,
    DERIVED_AFFLICTIONS_PACK_LABEL, DERIVED_AFFLICTIONS_PACK_NAME,
};
use crate::generated::afflictions::source_facts::affliction_family_label;
use crate::generated::afflictions::{AfflictionOccurrence, DerivedAfflictionRecordInput};
use crate::records::{LoadedSourceRecord, NormalizedRecord, SourceConstructionFacts};
use crate::source::normalize::normalize_text;

pub(super) fn derived_affliction_record(input: DerivedAfflictionRecordInput) -> LoadedSourceRecord {
    let id = RecordId::new(input.id).expect("derived id is valid");
    let raw_json = serde_json::to_string(&input.raw).expect("derived raw JSON serializes");
    let record = NormalizedRecord {
        key: input.key,
        id,
        name: input.name.clone(),
        normalized_name: normalize_text(&input.name),
        record_family: RecordFamily::Affliction,
        pack_name: PackName::new(
            if input.is_default_visible {
                DERIVED_AFFLICTIONS_PACK_NAME
            } else {
                DERIVED_AFFLICTION_INSTANCES_PACK_NAME
            }
            .to_string(),
        )
        .expect("static pack name"),
        pack_label: if input.is_default_visible {
            DERIVED_AFFLICTIONS_PACK_LABEL
        } else {
            DERIVED_AFFLICTION_INSTANCES_PACK_LABEL
        }
        .to_string(),
        foundry_document_type: "Item".to_string(),
        foundry_record_type: input.record_type.to_string(),
        level: input.level,
        rarity: input.rarity,
        traits: input.traits,
        prerequisites: Vec::new(),
        system_category: Some(affliction_family_label(input.family).to_string()),
        system_group: None,
        system_base_item: None,
        system_usage: None,
        system_price_json: None,
        system_actions_value: None,
        system_time_value: None,
        system_duration_value: None,
        price_cp: None,
        activation_time: None,
        duration: None,
        metrics: Vec::new(),
        actor_data: None,
        item_data: None,
        spell_data: None,
        publication_title: input.publication_title,
        publication_remaster: input.publication_remaster,
        description: input.description,
        blurb: input.blurb,
        supplemental_content: Vec::new(),
        publication_family: input.publication_family,
        folder_id: None,
        taxonomy_families: Vec::new(),
        variant_group_key: None,
        variant_base_name: None,
        variant_label: None,
        variant_axes: Vec::new(),
        variant_confidence: None,
        variant_source: "none".to_string(),
        source_path: input.source_path,
        is_default_visible: input.is_default_visible,
        raw_json,
    };
    LoadedSourceRecord::new(record, SourceConstructionFacts::empty())
}

pub(super) fn build_affliction_instance_raw(
    instance_id: &str,
    occurrence: &AfflictionOccurrence,
    canonical_record_key: &str,
    normalization_key: &str,
) -> Value {
    let mut raw = occurrence.raw_provenance.clone().unwrap_or_else(|| {
        json!({
            "_id": instance_id,
            "name": occurrence.name,
            "type": occurrence.host_record.foundry_record_type,
        })
    });
    if let Value::Object(object) = &mut raw {
        object.insert("_id".to_string(), Value::String(instance_id.to_string()));
        object.insert(
            "_derived".to_string(),
            json!({
                "kind": "afflictionInstance",
                "hostRecordKey": occurrence.host_record.key.to_string(),
                "sourceRecordKey": occurrence.source_record.as_ref().map(|record| record.key.to_string()),
                "canonicalRecordKey": canonical_record_key,
                "normalizationKey": normalization_key,
                "occurrenceRef": occurrence.occurrence_ref,
                "aliasNormalizationKeys": occurrence.candidate_keys,
            }),
        );
    }
    raw
}
