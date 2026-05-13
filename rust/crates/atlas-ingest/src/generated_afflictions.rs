use std::collections::BTreeMap;

use atlas_domain::{PackName, RecordId, RecordKey};
use serde_json::json;

mod clustering;
mod edges;
mod occurrences;
mod records;
mod source_facts;

use crate::generated_affliction_identity::{choose_affliction_canonical_identity_key, hash_text};
use crate::{
    AfflictionFamily, AfflictionOccurrence, DERIVED_AFFLICTION_INSTANCES_PACK_NAME,
    DERIVED_AFFLICTIONS_PACK_NAME, DerivedAfflictionRecordInput, GeneratedAfflictionBuild,
    LoadedRecord, RecordReferenceIndex, variants,
};
use clustering::{choose_affliction_authoritative_candidate, cluster_affliction_occurrences};
use edges::generated_affliction_edges;
use occurrences::collect_affliction_occurrences;
use records::{
    build_affliction_canonical_search_text, build_affliction_instance_raw,
    build_affliction_occurrence_search_text, derived_affliction_record,
};
use source_facts::{affliction_family_label, record_description_markup, record_description_text};

pub(super) fn build_generated_afflictions(
    records: &[LoadedRecord],
    index: &RecordReferenceIndex,
) -> GeneratedAfflictionBuild {
    let occurrences = collect_affliction_occurrences(records, index);
    if occurrences.is_empty() {
        return GeneratedAfflictionBuild {
            records: Vec::new(),
            references: Vec::new(),
        };
    }

    let mut occurrences_by_family = BTreeMap::<AfflictionFamily, Vec<AfflictionOccurrence>>::new();
    for occurrence in occurrences {
        occurrences_by_family
            .entry(occurrence.family)
            .or_default()
            .push(occurrence);
    }

    let mut generated_records = Vec::new();
    let mut generated_references = Vec::new();
    for (family, family_occurrences) in occurrences_by_family {
        for cluster in cluster_affliction_occurrences(family_occurrences) {
            let candidate = choose_affliction_authoritative_candidate(&cluster);
            let identity_key = choose_affliction_canonical_identity_key(
                &cluster
                    .iter()
                    .flat_map(|occurrence| occurrence.candidate_keys.clone())
                    .collect::<Vec<_>>(),
            );
            let representative = candidate.0;
            let authoritative_record = candidate.1;
            let authoritative_raw = candidate.2;
            let all_traits = variants::sorted_unique(
                cluster
                    .iter()
                    .flat_map(|occurrence| occurrence.traits.clone())
                    .collect(),
            );
            let all_linked_names = variants::sorted_unique(
                cluster
                    .iter()
                    .flat_map(|occurrence| occurrence.linked_names.clone())
                    .collect(),
            );
            let canonical_id = hash_text(&identity_key);
            let canonical_key = RecordKey::new(
                PackName::new(DERIVED_AFFLICTIONS_PACK_NAME.to_string()).expect("static pack name"),
                RecordId::new(canonical_id.clone()).expect("hash id is valid"),
            );
            let canonical_description_text =
                authoritative_record.and_then(|record| record.description_text.clone());
            let canonical_description_markup =
                authoritative_raw.and_then(record_description_markup);
            let representative_instance_key = cluster.first().map(|occurrence| {
                let instance_id = hash_text(&format!(
                    "{}:{}:{}",
                    identity_key, occurrence.host_record.key, occurrence.occurrence_ref
                ));
                format!("{DERIVED_AFFLICTION_INSTANCES_PACK_NAME}:{instance_id}")
            });
            let canonical_raw = json!({
                "_id": canonical_id,
                "name": representative.name,
                "type": "affliction",
                "system": {
                    "category": affliction_family_label(family),
                    "traits": {
                        "rarity": "common",
                        "value": all_traits,
                    },
                    "description": {
                        "value": canonical_description_markup.unwrap_or_default(),
                    },
                },
                "_derived": {
                    "kind": "canonicalAffliction",
                    "normalizationKey": identity_key,
                    "representativeInstanceRecordKey": representative_instance_key,
                    "linkedNames": all_linked_names,
                    "aliasNormalizationKeys": variants::sorted_unique(
                        cluster.iter().flat_map(|occurrence| occurrence.candidate_keys.clone()).collect()
                    ),
                    "groupKey": affliction_family_label(family),
                }
            });
            let canonical_record = derived_affliction_record(DerivedAfflictionRecordInput {
                key: canonical_key.clone(),
                id: canonical_id,
                name: representative.name.clone(),
                record_type: "affliction",
                family,
                traits: all_traits.clone(),
                description_text: canonical_description_text,
                blurb_text: authoritative_record.and_then(|record| record.blurb_text.clone()),
                level: authoritative_record
                    .and_then(|record| record.level)
                    .or(representative.host_record.level),
                rarity: authoritative_record
                    .and_then(|record| record.rarity.clone())
                    .or_else(|| representative.host_record.rarity.clone()),
                publication_title: authoritative_record
                    .and_then(|record| record.publication_title.clone())
                    .or_else(|| representative.host_record.publication_title.clone()),
                publication_remaster: authoritative_record
                    .map(|record| record.publication_remaster)
                    .unwrap_or(representative.host_record.publication_remaster),
                publication_family: authoritative_record
                    .map(|record| record.publication_family)
                    .unwrap_or(representative.host_record.publication_family),
                source_path: format!("derived://afflictions/{}", canonical_key.id()),
                is_default_visible: true,
                search_text_projection: build_affliction_canonical_search_text(
                    &representative.name,
                    family,
                    &all_traits,
                    representative.slug.as_deref(),
                    &all_linked_names,
                ),
                raw: canonical_raw,
            });
            generated_records.push(canonical_record.clone());

            for occurrence in &cluster {
                let instance_id = hash_text(&format!(
                    "{}:{}:{}",
                    identity_key, occurrence.host_record.key, occurrence.occurrence_ref
                ));
                let instance_key = RecordKey::new(
                    PackName::new(DERIVED_AFFLICTION_INSTANCES_PACK_NAME.to_string())
                        .expect("static pack name"),
                    RecordId::new(instance_id.clone()).expect("hash id is valid"),
                );
                let instance_description_text = record_description_text(&occurrence.child_raw);
                let instance_raw = build_affliction_instance_raw(
                    &instance_id,
                    occurrence,
                    &canonical_key.to_string(),
                    &identity_key,
                );
                let instance_record = derived_affliction_record(DerivedAfflictionRecordInput {
                    key: instance_key.clone(),
                    id: instance_id,
                    name: occurrence.name.clone(),
                    record_type: "affliction-instance",
                    family: occurrence.family,
                    traits: occurrence.traits.clone(),
                    description_text: instance_description_text,
                    blurb_text: None,
                    level: occurrence
                        .source_record
                        .as_ref()
                        .and_then(|record| record.level)
                        .or(occurrence.host_record.level),
                    rarity: occurrence
                        .source_record
                        .as_ref()
                        .and_then(|record| record.rarity.clone())
                        .or_else(|| occurrence.host_record.rarity.clone()),
                    publication_title: occurrence
                        .source_record
                        .as_ref()
                        .and_then(|record| record.publication_title.clone())
                        .or_else(|| occurrence.host_record.publication_title.clone()),
                    publication_remaster: occurrence
                        .source_record
                        .as_ref()
                        .map(|record| record.publication_remaster)
                        .unwrap_or(occurrence.host_record.publication_remaster),
                    publication_family: occurrence.host_record.publication_family,
                    source_path: occurrence.source_path.clone(),
                    is_default_visible: false,
                    search_text_projection: build_affliction_occurrence_search_text(
                        &occurrence.name,
                        occurrence.family,
                        &occurrence.traits,
                        &occurrence.linked_names,
                    ),
                    raw: instance_raw,
                });

                generated_references.extend(generated_affliction_edges(
                    occurrence,
                    &instance_key,
                    &canonical_key,
                ));
                generated_records.push(instance_record);
            }
        }
    }

    generated_records.sort_by_key(|record| record.key.to_string());
    generated_references.sort_by(|left, right| {
        (
            left.from_record_key.to_string(),
            left.to_record_key.to_string(),
            left.reference_text.as_str(),
        )
            .cmp(&(
                right.from_record_key.to_string(),
                right.to_record_key.to_string(),
                right.reference_text.as_str(),
            ))
    });
    GeneratedAfflictionBuild {
        records: generated_records,
        references: generated_references,
    }
}
