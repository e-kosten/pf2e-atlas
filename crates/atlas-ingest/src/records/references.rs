use std::collections::{BTreeMap, BTreeSet};

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
    for loaded in records.iter_mut() {
        let record_key = loaded.record.identity.key.clone();
        for child in &mut loaded.facts.canonical_spell_children {
            for document in &mut child.definition.content.documents {
                resolve_document_references(&mut document.document, index);
                document.refresh_derived_state();
            }
        }
        let canonical_content = match &mut loaded.facts.canonical_body {
            Some(RecordBody::Creature(creature)) => Some(&mut creature.content),
            Some(RecordBody::Hazard(hazard)) => Some(&mut hazard.content),
            Some(RecordBody::Spell(spell)) => Some(&mut spell.definition.content),
            Some(RecordBody::Consumable(consumable)) => Some(&mut consumable.content),
            None => None,
        };
        if let Some(content) = canonical_content {
            for document in &mut content.documents {
                resolve_document_references(&mut document.document, index);
                document.refresh_derived_state();
            }
            loaded.record.content.documents = content
                .documents
                .iter()
                .filter(|content| content.owner == ContentOwner::Record(record_key.clone()))
                .map(|content| RecordContentDocument {
                    source_kind: content.source_kind,
                    label: content.label.clone(),
                    document: content.document.clone(),
                })
                .collect();
        } else if let Some(RecordBody::Hazard(hazard)) = &mut loaded.facts.canonical_body {
            for content in &mut hazard.content.documents {
                resolve_document_references(&mut content.document, index);
                content.refresh_derived_state();
            }
            loaded.record.content.documents = hazard
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

        for occurrence in &mut loaded.facts.consumable_occurrences.occurrences {
            for document in &mut occurrence.authored_content.documents {
                resolve_document_references(&mut document.document, index);
                document.refresh_derived_state();
            }
        }
    }

    let consumable_content_hashes = records
        .iter()
        .filter_map(|loaded| loaded.facts.canonical_body.as_ref())
        .filter_map(RecordBody::as_consumable)
        .flat_map(|consumable| {
            consumable.content.documents.iter().map(|document| {
                (
                    (
                        document.id.parent_record_key.to_string(),
                        document.id.content_key.as_str().to_string(),
                    ),
                    document.content_hash.as_str().to_string(),
                )
            })
        })
        .collect::<BTreeMap<_, _>>();
    for document in records
        .iter_mut()
        .flat_map(|loaded| &mut loaded.facts.consumable_occurrences.occurrences)
        .flat_map(|occurrence| &mut occurrence.authored_content.documents)
    {
        if let DuplicateContentStatus::CopiedFromConsumableTarget {
            target_record_key,
            target_content_key,
            target_content_hash,
        } = &mut document.duplicate_status
            && let Some(resolved_hash) = consumable_content_hashes.get(&(
                target_record_key.to_string(),
                target_content_key.as_str().to_string(),
            ))
        {
            target_content_hash.clone_from(resolved_hash);
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
    let mut documents = match loaded.facts.canonical_body.as_ref() {
        Some(RecordBody::Creature(creature)) => owned_documents(&creature.content),
        Some(RecordBody::Hazard(hazard)) => owned_documents(&hazard.content),
        Some(RecordBody::Spell(spell)) => owned_documents(&spell.definition.content),
        Some(RecordBody::Consumable(consumable)) => owned_documents(&consumable.content),
        None if !loaded.facts.canonical_spell_children.is_empty() => {
            record_content_documents(&loaded.record)
        }
        None => return None,
    };
    documents.extend(
        loaded
            .facts
            .canonical_spell_children
            .iter()
            .flat_map(|child| owned_documents(&child.definition.content)),
    );
    Some(documents)
}

fn owned_documents(
    content: &atlas_record::OwnedRichContent,
) -> Vec<(ContentSourceKind, ContentVisibility, &RichDocument)> {
    content
        .documents
        .iter()
        .filter(|content| {
            !matches!(
                content.duplicate_status,
                DuplicateContentStatus::CopiedFromCanonicalTarget { .. }
                    | DuplicateContentStatus::CopiedFromConsumableTarget { .. }
            )
        })
        .map(|content| (content.source_kind, content.visibility, &content.document))
        .collect()
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
        AtlasRecord, ContentSourceKind, FactValue, FoundryDocumentType, FoundryLink,
        FoundryLinkBehavior, FoundryLinkMacroKind, FoundryLinkSource, FoundryRecordInfo,
        FoundryRecordType, RecordClassification, RecordContentDocument, RecordIdentity,
        RecordProvenance, RichDocument, RichLinkTarget, RichNode, SpellIdentity, SpellProvenance,
        SpellRecord, SpellSourceId, iter_foundry_links,
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

    #[test]
    fn canonical_spell_content_blocks_legacy_record_reference_fallback() {
        let target = loaded_record("spells-srd:targetSpell", "Target Spell", Vec::new());
        let mut host = loaded_record(
            "spells-srd:hostSpell",
            "Host Spell",
            vec![RecordContentDocument {
                source_kind: ContentSourceKind::Description,
                label: None,
                document: RichDocument::new(vec![RichNode::FoundryLink {
                    link: FoundryLink {
                        target: RichLinkTarget::Unresolved {
                            target: "Compendium.pf2e.spells-srd.Item.Target Spell".to_string(),
                            fallback_label: "Target Spell".to_string(),
                        },
                        label: None,
                        source: FoundryLinkSource {
                            macro_kind: FoundryLinkMacroKind::Uuid,
                            authored_target: "Compendium.pf2e.spells-srd.Item.Target Spell"
                                .to_string(),
                            relation: None,
                        },
                        behavior: FoundryLinkBehavior::Reference,
                    },
                }]),
            }],
        );
        let host_key = host.record.identity.key.clone();
        host.facts.canonical_body = Some(atlas_record::RecordBody::Spell(SpellRecord::new(
            SpellIdentity {
                record_key: host_key,
                source_id: SpellSourceId::new("hostSpell").expect("source id"),
                name: "Host Spell".to_string(),
            },
            SpellProvenance {
                source_path: "packs/spells/host-spell.json".to_string(),
                source_contract_version: "fixture".to_string(),
                source_system_version: "6.12.4".to_string(),
                source_upstream_commit: "fixture".to_string(),
                standalone_location: FactValue::Null,
            },
        )));
        let mut records = vec![host, target];
        let index = build_record_reference_index(&records);

        resolve_content_references(&mut records, &index);

        assert!(records[0].record.content.documents.is_empty());
        assert!(resolve_reference_edges(&records).is_empty());
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
