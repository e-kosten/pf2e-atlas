use atlas_domain::{PackName, RecordFamily, RecordId, TextStatus};
use serde_json::{Value, json};

use crate::generated_afflictions::source_facts::affliction_family_label;
use crate::normalize::normalize_text;
use crate::{
    AfflictionFamily, AfflictionOccurrence, DERIVED_AFFLICTION_INSTANCES_PACK_LABEL,
    DERIVED_AFFLICTION_INSTANCES_PACK_NAME, DERIVED_AFFLICTIONS_PACK_LABEL,
    DERIVED_AFFLICTIONS_PACK_NAME, DerivedAfflictionRecordInput, LoadedRecord, variants,
};

pub(super) fn derived_affliction_record(input: DerivedAfflictionRecordInput) -> LoadedRecord {
    let id = RecordId::new(input.id).expect("derived id is valid");
    let text_status = if input.description_text.is_some() {
        TextStatus::Resolved
    } else {
        TextStatus::Missing
    };
    let raw_json = serde_json::to_string(&input.raw).expect("derived raw JSON serializes");
    LoadedRecord {
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
        description_text: input.description_text,
        blurb_text: input.blurb_text,
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
        text_status,
        is_default_visible: input.is_default_visible,
        search_text_projection: input.search_text_projection,
        reference_candidates: Vec::new(),
        raw_json,
    }
}

pub(super) fn build_affliction_instance_raw(
    instance_id: &str,
    occurrence: &AfflictionOccurrence,
    canonical_record_key: &str,
    normalization_key: &str,
) -> Value {
    let mut raw = occurrence.child_raw.clone();
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

pub(super) fn build_affliction_occurrence_search_text(
    name: &str,
    family: AfflictionFamily,
    traits: &[String],
    linked_names: &[String],
) -> String {
    variants::sorted_unique(
        [
            vec![
                name.to_string(),
                affliction_family_label(family).to_string(),
            ],
            traits.to_vec(),
            linked_names.to_vec(),
        ]
        .concat(),
    )
    .join("\n")
}

pub(super) fn build_affliction_canonical_search_text(
    name: &str,
    family: AfflictionFamily,
    traits: &[String],
    slug: Option<&str>,
    linked_names: &[String],
) -> String {
    let slug_alias = slug.map(|value| value.replace(['-', '_'], " "));
    variants::sorted_unique(
        [
            vec![
                name.to_string(),
                affliction_family_label(family).to_string(),
            ],
            slug_alias.into_iter().collect(),
            traits.to_vec(),
            linked_names.to_vec(),
        ]
        .concat(),
    )
    .join("\n")
}
