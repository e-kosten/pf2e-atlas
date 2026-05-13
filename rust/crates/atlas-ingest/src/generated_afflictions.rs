use std::collections::BTreeMap;

use atlas_domain::{PackName, RecordFamily, RecordId, RecordKey, TextStatus};
use serde_json::{Value, json};

use crate::generated_affliction_identity::{
    build_affliction_occurrence_candidate_keys, choose_affliction_canonical_identity_key, hash_text,
};
use crate::normalize::{
    extract_traits, normalize_text, normalized_pointer_string, pointer_string, string_field,
    strip_markup,
};
use crate::references::{extract_reference_candidates_from_text, record_by_key};
use crate::{
    AfflictionFamily, AfflictionOccurrence, DERIVED_AFFLICTION_INSTANCES_PACK_LABEL,
    DERIVED_AFFLICTION_INSTANCES_PACK_NAME, DERIVED_AFFLICTIONS_PACK_LABEL,
    DERIVED_AFFLICTIONS_PACK_NAME, DerivedAfflictionRecordInput, GeneratedAfflictionBuild,
    LoadedRecord, RecordReferenceIndex, ReferenceEdge, variants,
};

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

                generated_references.push(ReferenceEdge {
                    from_record_key: occurrence.host_record.key.clone(),
                    to_record_key: instance_key.clone(),
                    display_text: Some(occurrence.name.clone()),
                    reference_text: format!("derived-affliction-instance:{instance_key}"),
                });
                generated_references.push(ReferenceEdge {
                    from_record_key: instance_key.clone(),
                    to_record_key: canonical_key.clone(),
                    display_text: Some(occurrence.name.clone()),
                    reference_text: format!("derived-affliction-canonical:{canonical_key}"),
                });
                generated_references.push(ReferenceEdge {
                    from_record_key: canonical_key.clone(),
                    to_record_key: occurrence.host_record.key.clone(),
                    display_text: Some(occurrence.host_record.name.clone()),
                    reference_text: format!(
                        "derived-affliction-host:{}:{instance_key}",
                        occurrence.host_record.key
                    ),
                });
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

fn collect_affliction_occurrences(
    records: &[LoadedRecord],
    index: &RecordReferenceIndex,
) -> Vec<AfflictionOccurrence> {
    let mut occurrences = Vec::new();
    for record in records {
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
    record: &LoadedRecord,
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
    record: &LoadedRecord,
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
                    .and_then(|record_key| record_by_key(index, record_key))
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

fn cluster_affliction_occurrences(
    occurrences: Vec<AfflictionOccurrence>,
) -> Vec<Vec<AfflictionOccurrence>> {
    if occurrences.len() <= 1 {
        return if occurrences.is_empty() {
            Vec::new()
        } else {
            vec![occurrences]
        };
    }

    let mut parent = (0..occurrences.len()).collect::<Vec<_>>();
    for left in 0..occurrences.len() {
        for right in (left + 1)..occurrences.len() {
            if occurrences[left]
                .candidate_keys
                .iter()
                .any(|key| occurrences[right].candidate_keys.contains(key))
            {
                union_parent(&mut parent, left, right);
            }
        }
    }
    let mut clusters = BTreeMap::<usize, Vec<AfflictionOccurrence>>::new();
    for (index, occurrence) in occurrences.into_iter().enumerate() {
        let root = find_parent(&mut parent, index);
        clusters.entry(root).or_default().push(occurrence);
    }
    clusters.into_values().collect()
}

fn find_parent(parent: &mut [usize], index: usize) -> usize {
    if parent[index] != index {
        let root = find_parent(parent, parent[index]);
        parent[index] = root;
    }
    parent[index]
}

fn union_parent(parent: &mut [usize], left: usize, right: usize) {
    let left_root = find_parent(parent, left);
    let right_root = find_parent(parent, right);
    if left_root != right_root {
        parent[right_root] = left_root;
    }
}

fn choose_affliction_authoritative_candidate(
    occurrences: &[AfflictionOccurrence],
) -> (&AfflictionOccurrence, Option<&LoadedRecord>, Option<&Value>) {
    let representative = occurrences
        .iter()
        .min_by(|left, right| {
            right
                .source_record
                .is_some()
                .cmp(&left.source_record.is_some())
                .then_with(|| {
                    left.host_record
                        .key
                        .to_string()
                        .cmp(&right.host_record.key.to_string())
                })
                .then_with(|| left.occurrence_ref.cmp(&right.occurrence_ref))
        })
        .expect("non-empty cluster");
    (
        representative,
        representative.source_record.as_ref(),
        representative.source_raw.as_ref(),
    )
}

fn derived_affliction_record(input: DerivedAfflictionRecordInput) -> LoadedRecord {
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

fn build_affliction_instance_raw(
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

fn build_affliction_occurrence_search_text(
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

fn build_affliction_canonical_search_text(
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

fn detect_affliction_family(raw: &Value) -> Option<AfflictionFamily> {
    let traits = extract_traits(raw);
    let system_category =
        normalized_pointer_string(raw, "/system/category").map(|value| normalize_text(&value));
    if traits.iter().any(|trait_value| trait_value == "disease")
        || system_category.as_deref() == Some("disease")
    {
        return Some(AfflictionFamily::Disease);
    }
    if traits.iter().any(|trait_value| trait_value == "poison")
        || system_category.as_deref() == Some("poison")
    {
        return Some(AfflictionFamily::Poison);
    }
    if traits.iter().any(|trait_value| trait_value == "curse")
        || system_category.as_deref() == Some("curse")
    {
        return Some(AfflictionFamily::Curse);
    }
    None
}

fn affliction_family_label(family: AfflictionFamily) -> &'static str {
    match family {
        AfflictionFamily::Curse => "curse",
        AfflictionFamily::Disease => "disease",
        AfflictionFamily::Poison => "poison",
    }
}

fn has_affliction_shape(raw: &Value) -> bool {
    let Some(description) = record_description_text(raw) else {
        return false;
    };
    let normalized = normalize_text(&description);
    normalized.contains("saving throw") && normalized.contains("stage 1")
}

fn record_description_markup(raw: &Value) -> Option<String> {
    [
        "/system/description/value",
        "/system/details/description",
        "/system/details/publicNotes",
        "/system/details/blurb",
    ]
    .into_iter()
    .find_map(|pointer| pointer_string(raw, pointer))
    .filter(|value| !value.trim().is_empty())
}

fn record_description_text(raw: &Value) -> Option<String> {
    record_description_markup(raw)
        .map(|value| strip_markup(&value))
        .filter(|value| !value.trim().is_empty())
}

fn record_slug(raw: &Value) -> Option<String> {
    normalized_pointer_string(raw, "/system/slug")
}

fn compendium_source(raw: &Value) -> Option<String> {
    normalized_pointer_string(raw, "/_stats/compendiumSource")
}

fn parse_compendium_source(value: &str) -> Option<(String, String)> {
    let parts = value.split('.').collect::<Vec<_>>();
    if parts.len() >= 5 && parts.first() == Some(&"Compendium") && parts.get(1) == Some(&"pf2e") {
        return Some((normalize_text(parts.get(2)?), normalize_text(parts.last()?)));
    }
    None
}

fn extract_linked_names_from_markup(markup: Option<&str>) -> Vec<String> {
    let Some(markup) = markup else {
        return Vec::new();
    };
    variants::sorted_unique(
        extract_reference_candidates_from_text(markup)
            .into_iter()
            .filter_map(|candidate| {
                candidate
                    .display_text
                    .or_else(|| fallback_linked_name(&candidate.raw_target))
            })
            .collect(),
    )
}

fn fallback_linked_name(locator: &str) -> Option<String> {
    let tail = locator.split('.').next_back()?.replace(['-', '_'], " ");
    let trimmed = tail.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}
