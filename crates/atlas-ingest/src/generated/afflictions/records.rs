use atlas_domain::RecordKind;
use serde_json::{Value, json};

use crate::diagnostics::{DERIVED_AFFLICTION_INSTANCES_PACK_LABEL, DERIVED_AFFLICTIONS_PACK_LABEL};
use crate::generated::afflictions::source_facts::affliction_family_label;
use crate::generated::afflictions::{AfflictionOccurrence, DerivedAfflictionRecordInput};
use crate::records::{
    AtlasRecord, ContentSourceKind, FoundryDocumentMechanics, FoundryDocumentType,
    FoundryRecordInfo, FoundryRecordType, ItemMechanics, LoadedSourceRecord, RecordClassification,
    RecordContent, RecordContentDocument, RecordIdentity, RecordMechanics, RecordProvenance,
    RecordPublication, RecordRequirements, RecordTaxonomy, RecordTiming, RecordVisibility,
    RecordVisibilityReason, SourceConstructionFacts,
};

pub(super) fn derived_affliction_record(input: DerivedAfflictionRecordInput) -> LoadedSourceRecord {
    let raw_json = input.raw.to_string();
    let mut documents = Vec::new();
    if let Some(document) = input.description {
        documents.push(RecordContentDocument {
            source_kind: ContentSourceKind::Description,
            label: None,
            document,
        });
    }
    if let Some(document) = input.blurb {
        documents.push(RecordContentDocument {
            source_kind: ContentSourceKind::Blurb,
            label: None,
            document,
        });
    }
    let record = AtlasRecord {
        identity: RecordIdentity {
            key: input.key,
            name: input.name.clone(),
        },
        classification: RecordClassification {
            kind: RecordKind::Affliction,
            level: input.level,
            rarity: input.rarity,
            traits: input.traits,
            taxonomy: RecordTaxonomy::default(),
        },
        foundry: FoundryRecordInfo {
            pack_label: if input.is_default_visible {
                DERIVED_AFFLICTIONS_PACK_LABEL
            } else {
                DERIVED_AFFLICTION_INSTANCES_PACK_LABEL
            }
            .to_string(),
            document_type: FoundryDocumentType::Item,
            record_type: FoundryRecordType::from_foundry(input.record_type),
            folder_id: None,
        },
        provenance: RecordProvenance {
            source_path: input.source_path,
            raw_json: Some(raw_json),
        },
        publication: RecordPublication {
            title: input.publication_title,
            remaster: input.publication_remaster,
            category: input.category,
        },
        requirements: RecordRequirements::default(),
        timing: RecordTiming::default(),
        mechanics: RecordMechanics {
            metrics: Vec::new(),
            document: FoundryDocumentMechanics::Item(ItemMechanics {
                category: Some(affliction_family_label(input.family).to_string()),
                ..ItemMechanics::default()
            }),
        },
        content: RecordContent { documents },
        variant: None,
        visibility: if input.is_default_visible {
            RecordVisibility::visible(RecordVisibilityReason::GeneratedCanonical)
        } else {
            RecordVisibility::hidden(RecordVisibilityReason::GeneratedInstance)
        },
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
            "type": occurrence.host_record.foundry.record_type.as_str(),
        })
    });
    if let Value::Object(object) = &mut raw {
        object.insert("_id".to_string(), Value::String(instance_id.to_string()));
        object.insert(
            "_derived".to_string(),
            json!({
                "kind": "afflictionInstance",
                "hostRecordKey": occurrence.host_record.identity.key.to_string(),
                "sourceRecordKey": occurrence.source_record.as_ref().map(|record| record.identity.key.to_string()),
                "canonicalRecordKey": canonical_record_key,
                "normalizationKey": normalization_key,
                "occurrenceRef": occurrence.occurrence_ref,
                "aliasNormalizationKeys": occurrence.candidate_keys,
            }),
        );
    }
    raw
}
