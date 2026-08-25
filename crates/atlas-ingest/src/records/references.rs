use std::collections::BTreeSet;

use atlas_domain::RecordKey;
use atlas_record::{
    AtlasRecord, ContentOwner, ContentSourceKind, ContentVisibility, DuplicateContentStatus,
    FoundryLink, FoundryLinkBehavior, RecordBody, RecordContentDocument, ReferenceEdge,
    ReferenceRelationKind, RichDocument, RichLinkTarget, iter_foundry_links, render_plain_text,
    visit_foundry_links_mut,
};

use crate::records::{LoadedSourceRecord, RecordReferenceIndex};
use crate::source::normalize::normalize_text;

pub(crate) fn build_record_reference_index(records: &[LoadedSourceRecord]) -> RecordReferenceIndex {
    let mut index = RecordReferenceIndex::default();
    for loaded in records {
        let record = &loaded.record;
        index
            .by_key
            .insert(record.identity.key.to_string(), record.clone());
        index.by_pack_id.insert(
            (
                record.identity.pack().as_str().to_string(),
                record.identity.id().as_str().to_string(),
            ),
            record.identity.key.clone(),
        );
        index.by_pack_id.insert(
            (
                normalize_text(record.identity.pack().as_str()),
                normalize_text(record.identity.id().as_str()),
            ),
            record.identity.key.clone(),
        );
        index
            .by_pack_name
            .entry((
                record.identity.pack().as_str().to_string(),
                record.identity.normalized_name(),
            ))
            .or_default()
            .push(record.identity.key.clone());
        index
            .by_name
            .entry(record.identity.normalized_name())
            .or_default()
            .push(record.identity.key.clone());
    }
    index
}

pub(crate) fn resolve_reference_edges(records: &[LoadedSourceRecord]) -> Vec<ReferenceEdge> {
    let mut seen = BTreeSet::new();
    let mut references = Vec::new();
    for loaded in records {
        let record = &loaded.record;
        let documents =
            owned_content_documents(loaded).unwrap_or_else(|| record_content_documents(record));
        for (source_kind, visibility, document) in documents {
            collect_document_reference_edges(
                record,
                source_kind,
                visibility,
                document,
                &mut seen,
                &mut references,
            );
        }
    }

    references.sort_by(|left, right| {
        (
            left.from_record_key.to_string(),
            left.to_record_key.to_string(),
            left.reference_text.as_str(),
            left.source_kind.as_str(),
        )
            .cmp(&(
                right.from_record_key.to_string(),
                right.to_record_key.to_string(),
                right.reference_text.as_str(),
                right.source_kind.as_str(),
            ))
    });
    references
}

pub(crate) fn resolve_content_references(
    records: &mut [LoadedSourceRecord],
    index: &RecordReferenceIndex,
) {
    for loaded in records {
        let record_key = loaded.record.identity.key.clone();
        if let Some(RecordBody::Creature(creature)) = &mut loaded.facts.canonical_body {
            for content in &mut creature.content.documents {
                resolve_document_references(&mut content.document, index);
                content.refresh_derived_state();
            }
            loaded.record.content.documents = creature
                .content
                .documents
                .iter()
                .filter(|content| content.owner == ContentOwner::Record(record_key.clone()))
                .map(|content| RecordContentDocument {
                    source_kind: content.source_kind,
                    label: content.label.clone(),
                    document: content.document.clone(),
                })
                .collect();
        } else {
            for content in &mut loaded.record.content.documents {
                if content.contributes_to_reference_occurrences() {
                    resolve_document_references(&mut content.document, index);
                }
            }
        }
    }
}

fn resolve_document_references(document: &mut RichDocument, index: &RecordReferenceIndex) {
    visit_foundry_links_mut(document, |link| {
        if let Some(record_key) = resolve_foundry_link(link, index)
            && let Some(record) = record_by_key(index, &record_key)
        {
            link.target = RichLinkTarget::Record {
                key: record_key,
                name: record.identity.name.clone(),
            };
        }
    });
}

fn collect_document_reference_edges(
    record: &AtlasRecord,
    source_kind: ContentSourceKind,
    visibility: ContentVisibility,
    document: &RichDocument,
    seen: &mut BTreeSet<(String, String, String, String)>,
    references: &mut Vec<ReferenceEdge>,
) {
    for link in iter_foundry_links(document) {
        let Some(to_record_key) = link.target.record_key() else {
            continue;
        };
        let reference_text = link.source.authored_target.clone();
        let dedupe_key = (
            record.identity.key.to_string(),
            to_record_key.to_string(),
            reference_text.clone(),
            source_kind.as_str().to_string(),
        );
        if seen.insert(dedupe_key) {
            references.push(ReferenceEdge {
                from_record_key: record.identity.key.clone(),
                to_record_key: to_record_key.clone(),
                display_text: reference_display_text(link),
                reference_text,
                relation_kind: reference_relation_kind(link),
                source_kind,
                visibility,
            });
        }
    }
}

fn record_content_documents(
    record: &AtlasRecord,
) -> Vec<(ContentSourceKind, ContentVisibility, &RichDocument)> {
    record
        .content
        .default_backlink_documents()
        .map(|content| (content.source_kind, content.visibility(), &content.document))
        .collect()
}

fn owned_content_documents(
    loaded: &LoadedSourceRecord,
) -> Option<Vec<(ContentSourceKind, ContentVisibility, &RichDocument)>> {
    let RecordBody::Creature(creature) = loaded.facts.canonical_body.as_ref()?;
    Some(
        creature
            .content
            .documents
            .iter()
            .filter(|content| {
                !matches!(
                    content.duplicate_status,
                    DuplicateContentStatus::CopiedFromCanonicalTarget { .. }
                )
            })
            .map(|content| (content.source_kind, content.visibility, &content.document))
            .collect(),
    )
}

fn resolve_foundry_link(link: &FoundryLink, index: &RecordReferenceIndex) -> Option<RecordKey> {
    match &link.target {
        RichLinkTarget::Record { key, .. } => Some(key.clone()),
        RichLinkTarget::LocalContent { .. } => None,
        RichLinkTarget::External { target, .. } | RichLinkTarget::Unresolved { target, .. } => {
            let (pack_name, locator) = reference_pack_and_locator(target)?;
            resolve_record_key(Some(&pack_name), &locator, index)
        }
    }
}

fn reference_display_text(link: &FoundryLink) -> Option<String> {
    link.label
        .as_ref()
        .map(|label| render_plain_text(&RichDocument::new(label.clone())))
        .filter(|label| !label.trim().is_empty())
        .or_else(|| link.target.display_name().map(ToOwned::to_owned))
}

fn reference_relation_kind(link: &FoundryLink) -> ReferenceRelationKind {
    match link.behavior {
        FoundryLinkBehavior::Reference => ReferenceRelationKind::Reference,
        FoundryLinkBehavior::Embed { .. } => ReferenceRelationKind::Embed,
    }
}

pub(crate) fn reference_pack_and_locator(raw_target: &str) -> Option<(String, String)> {
    let parts = raw_target.split('.').collect::<Vec<_>>();
    if parts.len() >= 5 && parts.first() == Some(&"Compendium") && parts.get(1) == Some(&"pf2e") {
        return Some((parts.get(2)?.to_string(), parts.last()?.to_string()));
    }
    if parts.len() >= 3 && parts.first() == Some(&"pf2e") {
        return Some((parts.get(1)?.to_string(), parts.last()?.to_string()));
    }
    None
}

pub(crate) fn resolve_record_key(
    pack_name: Option<&str>,
    locator_or_name: &str,
    index: &RecordReferenceIndex,
) -> Option<RecordKey> {
    let normalized = normalize_text(locator_or_name);
    if normalized.is_empty() {
        return None;
    }

    if let Some(pack_name) = pack_name {
        if let Some(record_key) = index
            .by_pack_id
            .get(&(pack_name.to_string(), locator_or_name.to_string()))
        {
            return Some(record_key.clone());
        }

        let matches = index
            .by_pack_name
            .get(&(pack_name.to_string(), normalized.clone()))?;
        return (matches.len() == 1).then(|| matches[0].clone());
    }

    let matches = index.by_name.get(&normalized)?;
    (matches.len() == 1).then(|| matches[0].clone())
}

pub(crate) fn record_by_key<'a>(
    index: &'a RecordReferenceIndex,
    record_key: &RecordKey,
) -> Option<&'a AtlasRecord> {
    index.by_key.get(&record_key.to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use atlas_domain::{RecordKey, RecordKind};
    use atlas_record::{
        AtlasRecord, ContentSourceKind, FoundryDocumentType, FoundryLink, FoundryLinkBehavior,
        FoundryLinkMacroKind, FoundryLinkSource, FoundryRecordInfo, FoundryRecordType,
        RecordClassification, RecordContentDocument, RecordIdentity, RecordProvenance,
        RichDocument, RichLinkTarget, RichNode, iter_foundry_links,
    };

    use super::{
        build_record_reference_index, resolve_content_references, resolve_reference_edges,
    };
    use crate::records::{LoadedSourceRecord, SourceConstructionFacts};

    #[test]
    fn embedded_content_resolves_occurrences_without_default_backlink_edges() {
        let target = loaded_record("actions:targetAction", "Target Action", Vec::new());
        let host = loaded_record(
            "actions:hostAction",
            "Host Action",
            vec![RecordContentDocument {
                source_kind: ContentSourceKind::EmbeddedItemDescription,
                label: Some("Embedded Item".to_string()),
                document: RichDocument::new(vec![RichNode::HtmlElement {
                    tag: "p".to_string(),
                    attributes: BTreeMap::new(),
                    children: vec![RichNode::FoundryLink {
                        link: FoundryLink {
                            target: RichLinkTarget::Unresolved {
                                target: "Compendium.pf2e.actions.Item.Target Action".to_string(),
                                fallback_label: "Target Action".to_string(),
                            },
                            label: None,
                            source: FoundryLinkSource {
                                macro_kind: FoundryLinkMacroKind::Uuid,
                                authored_target: "Compendium.pf2e.actions.Item.Target Action"
                                    .to_string(),
                                relation: None,
                            },
                            behavior: FoundryLinkBehavior::Reference,
                        },
                    }],
                }]),
            }],
        );
        let mut records = vec![host, target];
        let index = build_record_reference_index(&records);

        resolve_content_references(&mut records, &index);
        let embedded_document = &records[0].record.content.documents[0].document;
        let references = iter_foundry_links(embedded_document).collect::<Vec<_>>();

        assert_eq!(references.len(), 1);
        let RichLinkTarget::Record { key, name } = &references[0].target else {
            panic!("reference should resolve to record target");
        };
        assert_eq!(key.to_string(), "actions:targetAction");
        assert_eq!(name, "Target Action");
        assert!(
            resolve_reference_edges(&records).is_empty(),
            "embedded content should resolve occurrences but stay out of default backlink edges"
        );
    }

    fn loaded_record(
        key: &str,
        name: &str,
        content: Vec<RecordContentDocument>,
    ) -> LoadedSourceRecord {
        let key = RecordKey::parse(key).expect("valid test record key");
        let mut record = AtlasRecord::new(
            RecordIdentity::new(key, name),
            RecordClassification::new(RecordKind::Rule),
            FoundryRecordInfo::new(
                "Actions",
                FoundryDocumentType::Item,
                FoundryRecordType::Action,
            ),
            RecordProvenance::new(format!("packs/actions/{name}.json")),
        );
        record.content.documents = content;
        LoadedSourceRecord::new(record, SourceConstructionFacts::empty())
    }
}
