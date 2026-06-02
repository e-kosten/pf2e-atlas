use atlas_domain::RecordKey;

use crate::generated::afflictions::AfflictionOccurrence;
use crate::generated::afflictions::identity::build_affliction_occurrence_candidate_keys;
use crate::generated::afflictions::source_facts::{
    affliction_family_label, detect_affliction_family, embedded_item_affliction_document,
    extract_linked_names, has_affliction_shape, parse_compendium_source,
    record_affliction_document,
};
use crate::records::EmbeddedItemFact;
use crate::records::references::record_by_key;
use crate::records::variants;
use crate::records::{AtlasRecord, LoadedSourceRecord, RecordReferenceIndex, SourceRecordFacts};

pub(super) fn collect_affliction_occurrences(
    records: &[LoadedSourceRecord],
    index: &RecordReferenceIndex,
) -> Vec<AfflictionOccurrence> {
    let mut occurrences = Vec::new();
    for loaded in records {
        let record = &loaded.record;
        let source_facts = &loaded.facts.source_facts;
        if let Some(occurrence) = collect_top_level_affliction_occurrence(record, source_facts) {
            occurrences.push(occurrence);
        }
        occurrences.extend(collect_embedded_affliction_occurrences(
            record,
            source_facts,
            index,
        ));
    }
    occurrences
}

fn collect_top_level_affliction_occurrence(
    record: &AtlasRecord,
    source_facts: &SourceRecordFacts,
) -> Option<AfflictionOccurrence> {
    if record.foundry.record_type.as_str() == "affliction" {
        return None;
    }
    if !can_generate_affliction_from_record_type(record.foundry.record_type.as_str()) {
        return None;
    }
    let document = record_affliction_document(record);
    let family = detect_affliction_family(
        &record.classification.traits,
        record
            .mechanics
            .item()
            .and_then(|item| item.category.as_deref()),
    )?;
    if !has_affliction_shape(document.as_ref()) {
        return None;
    }
    Some(AfflictionOccurrence {
        host_record: record.clone(),
        source_record: Some(record.clone()),
        description: document.clone(),
        raw_provenance: None,
        family,
        name: record.identity.name.clone(),
        traits: variants::sorted_unique(
            [
                vec![affliction_family_label(family).to_string()],
                record.classification.traits.clone(),
            ]
            .concat(),
        ),
        linked_names: extract_linked_names(document.as_ref()),
        source_path: format!("{}#self", record.provenance.source_path),
        occurrence_ref: "self".to_string(),
        candidate_keys: build_affliction_occurrence_candidate_keys(
            family,
            &record.identity.name,
            source_facts.slug.as_deref(),
            source_facts.compendium_source.as_deref(),
            Some(&record.identity.key.to_string()),
        ),
    })
}

fn collect_embedded_affliction_occurrences(
    record: &AtlasRecord,
    source_facts: &SourceRecordFacts,
    index: &RecordReferenceIndex,
) -> Vec<AfflictionOccurrence> {
    let mut occurrences = Vec::new();
    for item in &source_facts.embedded_items {
        let document = embedded_item_affliction_document(item, source_facts);
        let family = match detect_affliction_family(&item.traits, item.system_category.as_deref()) {
            Some(family)
                if can_generate_affliction_from_item_type(item)
                    && has_affliction_shape(document.as_ref()) =>
            {
                family
            }
            _ => continue,
        };
        let source_record = item
            .compendium_source
            .as_deref()
            .and_then(parse_compendium_source)
            .and_then(|(pack_name, id)| {
                index
                    .by_pack_id
                    .get(&(pack_name, id))
                    .and_then(|record_key: &RecordKey| record_by_key(index, record_key))
                    .cloned()
            });
        occurrences.push(AfflictionOccurrence {
            host_record: record.clone(),
            source_record: source_record.clone(),
            description: document.clone(),
            raw_provenance: item.raw_provenance.clone(),
            family,
            name: item.name.clone(),
            traits: variants::sorted_unique(
                [
                    vec![affliction_family_label(family).to_string()],
                    item.traits.clone(),
                ]
                .concat(),
            ),
            linked_names: extract_linked_names(document.as_ref()),
            source_path: format!("{}#item:{}", record.provenance.source_path, item.item_id),
            occurrence_ref: item.item_id.clone(),
            candidate_keys: build_affliction_occurrence_candidate_keys(
                family,
                &item.name,
                item.slug.as_deref(),
                item.compendium_source.as_deref(),
                source_record
                    .as_ref()
                    .map(|record| record.identity.key.to_string())
                    .as_deref(),
            ),
        });
    }
    occurrences
}

fn can_generate_affliction_from_record_type(record_type: &str) -> bool {
    matches!(record_type, "action" | "consumable" | "spell")
}

fn can_generate_affliction_from_item_type(item: &EmbeddedItemFact) -> bool {
    can_generate_affliction_from_record_type(&item.foundry_item_type)
}
