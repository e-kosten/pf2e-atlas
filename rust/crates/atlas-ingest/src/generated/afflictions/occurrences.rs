use atlas_domain::RecordKey;
use serde_json::Value;

use crate::generated::afflictions::AfflictionOccurrence;
use crate::generated::afflictions::identity::build_affliction_occurrence_candidate_keys;
use crate::generated::afflictions::source_facts::{
    affliction_family_label, compendium_source, detect_affliction_family,
    extract_linked_names_from_markup, has_affliction_shape, parse_compendium_source,
    record_description_markup, record_slug,
};
use crate::records::references::record_by_key;
use crate::records::variants;
use crate::records::{LoadedSourceRecord, NormalizedRecord, RecordReferenceIndex};
use crate::source::normalize::{extract_traits, string_field};

pub(super) fn collect_affliction_occurrences(
    records: &[LoadedSourceRecord],
    index: &RecordReferenceIndex,
) -> Vec<AfflictionOccurrence> {
    let mut occurrences = Vec::new();
    for loaded in records {
        let record = &loaded.record;
        let Ok(raw) = serde_json::from_str::<Value>(&record.raw_json) else {
            continue;
        };
        if let Some(occurrence) = collect_top_level_affliction_occurrence(record, &raw) {
            occurrences.push(occurrence);
        }
        occurrences.extend(collect_embedded_affliction_occurrences(record, &raw, index));
    }
    occurrences
}

fn collect_top_level_affliction_occurrence(
    record: &NormalizedRecord,
    raw: &Value,
) -> Option<AfflictionOccurrence> {
    if record.foundry_record_type == "affliction" {
        return None;
    }
    if !can_generate_affliction_from_raw_type(raw) {
        return None;
    }
    let family = detect_affliction_family(raw)?;
    if !has_affliction_shape(raw) {
        return None;
    }
    let slug = record_slug(raw);
    let compendium_source = compendium_source(raw);
    Some(AfflictionOccurrence {
        host_record: record.clone(),
        source_record: Some(record.clone()),
        source_raw: Some(raw.clone()),
        child_raw: raw.clone(),
        family,
        name: record.name.clone(),
        slug: slug.clone(),
        traits: variants::sorted_unique(
            [
                vec![affliction_family_label(family).to_string()],
                extract_traits(raw),
            ]
            .concat(),
        ),
        linked_names: extract_linked_names_from_markup(record_description_markup(raw).as_deref()),
        source_path: format!("{}#self", record.source_path),
        occurrence_ref: "self".to_string(),
        candidate_keys: build_affliction_occurrence_candidate_keys(
            family,
            &record.name,
            slug.as_deref(),
            compendium_source.as_deref(),
            Some(&record.key.to_string()),
        ),
    })
}

fn collect_embedded_affliction_occurrences(
    record: &NormalizedRecord,
    raw: &Value,
    index: &RecordReferenceIndex,
) -> Vec<AfflictionOccurrence> {
    let Some(items) = raw.get("items").and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut occurrences = Vec::new();
    for (item_index, item) in items.iter().enumerate() {
        let family = match detect_affliction_family(item) {
            Some(family)
                if can_generate_affliction_from_raw_type(item) && has_affliction_shape(item) =>
            {
                family
            }
            _ => continue,
        };
        let Some(name) = string_field(item, "name").map(|value| value.trim().to_string()) else {
            continue;
        };
        if name.is_empty() {
            continue;
        }
        let slug = record_slug(item);
        let compendium_source = compendium_source(item);
        let source_record = compendium_source
            .as_deref()
            .and_then(parse_compendium_source)
            .and_then(|(pack_name, id)| {
                index
                    .by_pack_id
                    .get(&(pack_name, id))
                    .and_then(|record_key: &RecordKey| record_by_key(index, record_key))
                    .cloned()
            });
        let source_raw = source_record
            .as_ref()
            .and_then(|source_record| serde_json::from_str::<Value>(&source_record.raw_json).ok());
        let child_id = string_field(item, "_id").unwrap_or_else(|| format!("item-{item_index}"));
        occurrences.push(AfflictionOccurrence {
            host_record: record.clone(),
            source_record: source_record.clone(),
            source_raw,
            child_raw: item.clone(),
            family,
            name: name.clone(),
            slug: slug.clone(),
            traits: variants::sorted_unique(
                [
                    vec![affliction_family_label(family).to_string()],
                    extract_traits(item),
                ]
                .concat(),
            ),
            linked_names: extract_linked_names_from_markup(
                record_description_markup(item).as_deref(),
            ),
            source_path: format!("{}#item:{child_id}", record.source_path),
            occurrence_ref: child_id,
            candidate_keys: build_affliction_occurrence_candidate_keys(
                family,
                &name,
                slug.as_deref(),
                compendium_source.as_deref(),
                source_record
                    .as_ref()
                    .map(|record| record.key.to_string())
                    .as_deref(),
            ),
        });
    }
    occurrences
}

fn can_generate_affliction_from_raw_type(raw: &Value) -> bool {
    matches!(
        string_field(raw, "type").as_deref(),
        Some("action" | "consumable" | "spell")
    )
}
