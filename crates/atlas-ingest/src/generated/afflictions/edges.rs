use atlas_domain::RecordKey;
use atlas_record::{ContentSourceKind, ContentVisibility, ReferenceEdge, ReferenceRelationKind};

use crate::generated::afflictions::AfflictionOccurrence;

pub(super) fn generated_affliction_edges(
    occurrence: &AfflictionOccurrence,
    instance_key: &RecordKey,
    canonical_key: &RecordKey,
) -> [ReferenceEdge; 3] {
    [
        ReferenceEdge {
            from_record_key: occurrence.host_record.identity.key.clone(),
            to_record_key: instance_key.clone(),
            display_text: Some(occurrence.name.clone()),
            reference_text: format!("derived-affliction-instance:{instance_key}"),
            relation_kind: ReferenceRelationKind::Reference,
            source_kind: ContentSourceKind::GeneratedAffliction,
            visibility: ContentVisibility::Public,
        },
        ReferenceEdge {
            from_record_key: instance_key.clone(),
            to_record_key: canonical_key.clone(),
            display_text: Some(occurrence.name.clone()),
            reference_text: format!("derived-affliction-canonical:{canonical_key}"),
            relation_kind: ReferenceRelationKind::Reference,
            source_kind: ContentSourceKind::GeneratedAffliction,
            visibility: ContentVisibility::Public,
        },
        ReferenceEdge {
            from_record_key: canonical_key.clone(),
            to_record_key: occurrence.host_record.identity.key.clone(),
            display_text: Some(occurrence.host_record.identity.name.clone()),
            reference_text: format!(
                "derived-affliction-host:{}:{instance_key}",
                occurrence.host_record.identity.key
            ),
            relation_kind: ReferenceRelationKind::Reference,
            source_kind: ContentSourceKind::GeneratedAffliction,
            visibility: ContentVisibility::Public,
        },
    ]
}
