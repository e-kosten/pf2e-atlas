use atlas_domain::RecordKey;
use atlas_record::{ContentSourceKind, SupplementalContentDocument};
use serde_json::Value;

use crate::records::{EmbeddedItemContentRef, EmbeddedItemFact};

use super::content_sources::{embedded_item_content_key, embedded_item_id};
use super::{
    extract_traits, normalize_text, normalized_pointer_string, pointer_bool, string_field,
};

pub(super) fn extract_embedded_item_facts(
    raw: &Value,
    host_record_key: &RecordKey,
) -> Vec<EmbeddedItemFact> {
    let Some(items) = raw.pointer("/items").and_then(Value::as_array) else {
        return Vec::new();
    };
    items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| embedded_item_fact(item, host_record_key, index))
        .collect()
}

fn embedded_item_fact(
    item: &Value,
    host_record_key: &RecordKey,
    index: usize,
) -> Option<EmbeddedItemFact> {
    let name = string_field(item, "name")?.trim().to_string();
    if name.is_empty() {
        return None;
    }
    let item_id = embedded_item_id(item, index);
    let foundry_item_type = string_field(item, "type").unwrap_or_default();
    Some(EmbeddedItemFact {
        host_record_key: host_record_key.clone(),
        item_id,
        name: name.clone(),
        normalized_name: normalize_text(&name),
        foundry_item_type,
        traits: extract_traits(item),
        system_category: normalized_pointer_string(item, "/system/category"),
        slug: normalized_pointer_string(item, "/system/slug"),
        compendium_source: normalized_pointer_string(item, "/_stats/compendiumSource"),
        publication_remaster: pointer_bool(item, "/system/publication/remaster").unwrap_or(false),
        content_refs: Vec::new(),
        raw_provenance: Some(item.clone()),
    })
}

pub(super) fn attach_embedded_content_refs(
    embedded_items: &mut [EmbeddedItemFact],
    source_content: &std::collections::BTreeMap<String, SupplementalContentDocument>,
) {
    for item in embedded_items {
        for (source_kind, suffix) in [
            (ContentSourceKind::EmbeddedItemDescription, "description"),
            (
                ContentSourceKind::EmbeddedSpellDescription,
                "spell-description",
            ),
        ] {
            let local_key = embedded_item_content_key(&item.item_id, suffix);
            if source_content.contains_key(&local_key) {
                item.content_refs.push(EmbeddedItemContentRef {
                    source_kind,
                    local_key,
                });
            }
        }
    }
}
