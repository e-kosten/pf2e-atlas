use std::collections::BTreeMap;

use atlas_embedding::{
    DocumentEmbeddingContentSource, DocumentEmbeddingSource, EmbeddingUnitKind,
    PendingDocumentEmbedding, build_document_embedding_units,
};
use atlas_record::{
    AtlasRecord, ContentSourceKind, DuplicateContentStatus, OwnedRichContent,
    ProductRetrievalPolicy, RecordAlias, RecordBody, RemasterLink,
    build_search_presentation_document_with_content_filter,
};

use crate::records::LoadedSourceRecord;

pub(crate) mod generation;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct DocumentEmbeddingUnitSummary {
    pub total_units: usize,
    pub parent_units: usize,
    pub child_units: usize,
    pub records_with_child_units: usize,
    pub records_over_20_child_units: usize,
    pub records_over_50_child_units: usize,
    pub records_over_100_child_units: usize,
    pub max_child_units_per_record: usize,
}

pub(crate) fn build_pending_document_embeddings(
    records: &[LoadedSourceRecord],
    aliases: &[RecordAlias],
    remaster_links: &[RemasterLink],
) -> Vec<PendingDocumentEmbedding> {
    let aliases_by_key = aliases_by_record_key(aliases);
    let retrieval_policy = ProductRetrievalPolicy::from_remaster_links(remaster_links);

    let sources = records
        .iter()
        .filter_map(|loaded| {
            let record = &loaded.record;
            if !retrieval_policy.is_ordinary(record) {
                return None;
            }
            let canonical_content = canonical_embedding_content_documents(loaded);
            let record_key = record.identity.key.to_string();
            Some(DocumentEmbeddingSource {
                record_key,
                record_name: record.identity.name.clone(),
                document: build_search_presentation_document_with_content_filter(
                    record,
                    loaded.facts.canonical_body.as_ref(),
                    |content| canonical_content.is_none() && content.contributes_to_search(),
                ),
                aliases: aliases_by_key
                    .get(&record.identity.key.to_string())
                    .cloned()
                    .unwrap_or_default(),
                content_documents: canonical_content
                    .unwrap_or_else(|| embedding_content_documents(record)),
            })
        })
        .collect::<Vec<_>>();

    build_document_embedding_units(&sources)
}

fn canonical_embedding_content_documents(
    loaded: &LoadedSourceRecord,
) -> Option<Vec<DocumentEmbeddingContentSource>> {
    let RecordBody::Creature(creature) = loaded.facts.canonical_body.as_ref()?;
    Some(embedding_content_documents_from_owned(&creature.content))
}

fn embedding_content_documents_from_owned(
    content: &OwnedRichContent,
) -> Vec<DocumentEmbeddingContentSource> {
    content
        .documents
        .iter()
        .filter(|document| matches!(document.duplicate_status, DuplicateContentStatus::Unique))
        .map(|document| DocumentEmbeddingContentSource {
            source_kind: document.source_kind,
            label: document.label.clone().or_else(|| {
                Some(match document.source_kind {
                    ContentSourceKind::Description => "Description".to_string(),
                    ContentSourceKind::Blurb => "Summary".to_string(),
                    other => other.as_str().to_string(),
                })
            }),
            document: document.document.clone(),
        })
        .collect()
}

pub(crate) fn summarize_pending_document_embeddings(
    pending: &[PendingDocumentEmbedding],
) -> DocumentEmbeddingUnitSummary {
    let mut child_units_by_record = BTreeMap::<&str, usize>::new();
    let mut summary = DocumentEmbeddingUnitSummary {
        total_units: pending.len(),
        ..Default::default()
    };
    for unit in pending {
        if unit.unit_kind == EmbeddingUnitKind::Parent {
            summary.parent_units += 1;
        } else {
            summary.child_units += 1;
            *child_units_by_record
                .entry(unit.record_key.as_str())
                .or_default() += 1;
        }
    }
    summary.records_with_child_units = child_units_by_record.len();
    for child_units in child_units_by_record.values().copied() {
        summary.max_child_units_per_record = summary.max_child_units_per_record.max(child_units);
        summary.records_over_20_child_units += usize::from(child_units > 20);
        summary.records_over_50_child_units += usize::from(child_units > 50);
        summary.records_over_100_child_units += usize::from(child_units > 100);
    }
    summary
}

fn embedding_content_documents(record: &AtlasRecord) -> Vec<DocumentEmbeddingContentSource> {
    record
        .content
        .searchable_documents()
        .filter(|content| !content.source_kind.is_embedded())
        .map(|content| DocumentEmbeddingContentSource {
            source_kind: content.source_kind,
            label: content.label.clone().or_else(|| {
                Some(match content.source_kind {
                    ContentSourceKind::Description => "Description".to_string(),
                    ContentSourceKind::Blurb => "Summary".to_string(),
                    other => other.as_str().to_string(),
                })
            }),
            document: content.document.clone(),
        })
        .collect()
}

fn aliases_by_record_key(aliases: &[RecordAlias]) -> BTreeMap<String, Vec<String>> {
    let mut by_key = BTreeMap::<String, Vec<String>>::new();
    for alias in aliases {
        by_key
            .entry(alias.canonical_record_key.to_string())
            .or_default()
            .push(alias.alias_text.clone());
    }
    for aliases in by_key.values_mut() {
        aliases.sort();
        aliases.dedup();
    }
    by_key
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::Path;

    use atlas_domain::{PackName, RecordId, RecordKey, RecordKind, RemasterLinkSource};
    use atlas_record::{
        AliasSource, AtlasRecord, ContentId, ContentIdentityStability, ContentKey, ContentOrigin,
        ContentOwner, ContentProvenance, ContentRole, ContentSourceKind, DuplicateContentStatus,
        FactValue, FoundryDocumentType, FoundryRecordInfo, FoundryRecordType, OwnedRichContent,
        OwnedRichContentDocument, RecordAlias, RecordBody, RecordClassification,
        RecordContentDocument, RecordIdentity, RecordProvenance, RemasterLink, RichDocument,
        RichNode,
    };
    use serde_json::json;

    use super::{
        build_pending_document_embeddings, embedding_content_documents_from_owned,
        summarize_pending_document_embeddings,
    };

    #[test]
    fn embedded_content_is_excluded_from_embedding_inputs_until_promoted() {
        let mut record = base_record();
        record.content.documents.push(RecordContentDocument {
            source_kind: ContentSourceKind::Description,
            label: None,
            document: text_document("Primary description"),
        });
        record.content.documents.push(RecordContentDocument {
            source_kind: ContentSourceKind::EmbeddedItemDescription,
            label: Some("Embedded Strike".to_string()),
            document: RichDocument::new(vec![
                html_element("h2", vec![text_node("Embedded Strike")]),
                html_element("p", vec![text_node("Embedded capability text")]),
            ]),
        });

        let pending = build_pending_document_embeddings(
            &[crate::records::LoadedSourceRecord::new(
                record,
                crate::records::SourceConstructionFacts::empty(),
            )],
            &[],
            &[],
        );

        let parent = pending
            .iter()
            .find(|unit| unit.embedding_unit_key == "test-pack:TestRecord#parent")
            .expect("parent unit exists");
        assert!(!parent.input_text.contains("Embedded capability text"));
        assert_eq!(pending.len(), 1);
    }

    #[test]
    fn canonical_embedding_content_keeps_unique_authored_prose_and_drops_copied_prose() {
        let record_key = RecordKey::parse("test-pack:TestRecord").expect("record key parses");
        let content = OwnedRichContent {
            documents: vec![
                owned_content_document(
                    &record_key,
                    "gm-notes",
                    ContentSourceKind::GmNotes,
                    "Unique GM-authored context",
                    DuplicateContentStatus::Unique,
                ),
                owned_content_document(
                    &record_key,
                    "unique-embedded",
                    ContentSourceKind::EmbeddedItemDescription,
                    "Unique embedded capability",
                    DuplicateContentStatus::Unique,
                ),
                owned_content_document(
                    &record_key,
                    "copied-embedded",
                    ContentSourceKind::EmbeddedSpellDescription,
                    "Copied canonical spell prose",
                    DuplicateContentStatus::CopiedFromCanonicalTarget {
                        target_record_key: RecordKey::parse("spells:CanonicalSpell")
                            .expect("target key parses"),
                    },
                ),
            ],
            exclusions: Vec::new(),
        };

        let documents = embedding_content_documents_from_owned(&content);
        let text = documents
            .iter()
            .map(|document| atlas_record::render_plain_text(&document.document))
            .collect::<Vec<_>>();

        assert_eq!(documents.len(), 2);
        assert!(
            text.iter()
                .any(|value| value == "Unique GM-authored context")
        );
        assert!(
            text.iter()
                .any(|value| value == "Unique embedded capability")
        );
        assert!(
            !text
                .iter()
                .any(|value| value == "Copied canonical spell prose")
        );
    }

    #[test]
    fn canonical_source_record_builds_stable_search_units_and_remaster_demotion() {
        let mut loaded = canonical_loaded_fixture();
        assert!(loaded.record.mechanics.metrics.is_empty());
        let record_key = loaded.record.identity.key.clone();
        let RecordBody::Creature(creature) = loaded
            .facts
            .canonical_body
            .as_mut()
            .expect("canonical creature body");
        let long_unique_prose =
            format!("Unique embedded tactical context {}", "prose ".repeat(600));
        creature.content.documents = vec![
            owned_content_document(
                &record_key,
                "unique-embedded",
                ContentSourceKind::EmbeddedItemDescription,
                &long_unique_prose,
                DuplicateContentStatus::Unique,
            ),
            owned_content_document(
                &record_key,
                "copied-spell",
                ContentSourceKind::EmbeddedSpellDescription,
                "Copied canonical spell prose must not enter semantic input",
                DuplicateContentStatus::CopiedFromCanonicalTarget {
                    target_record_key: RecordKey::parse("spells:CanonicalSpell")
                        .expect("target key parses"),
                },
            ),
        ];
        let aliases = vec![
            RecordAlias {
                canonical_record_key: record_key.clone(),
                alias_text: "Zeta Alias".to_string(),
                normalized_alias: "zeta alias".to_string(),
                source: AliasSource::Migration,
                source_ref: "fixture-zeta".to_string(),
            },
            RecordAlias {
                canonical_record_key: record_key.clone(),
                alias_text: "Alpha Alias".to_string(),
                normalized_alias: "alpha alias".to_string(),
                source: AliasSource::Migration,
                source_ref: "fixture-alpha".to_string(),
            },
            RecordAlias {
                canonical_record_key: record_key.clone(),
                alias_text: "Alpha Alias".to_string(),
                normalized_alias: "alpha alias".to_string(),
                source: AliasSource::Migration,
                source_ref: "fixture-duplicate".to_string(),
            },
        ];

        let first = build_pending_document_embeddings(&[loaded.clone()], &aliases, &[]);
        let second = build_pending_document_embeddings(&[loaded.clone()], &aliases, &[]);
        assert_eq!(first, second);
        assert_eq!(first.len(), 1);
        let parent = &first[0];
        assert_eq!(parent.embedding_unit_key, "bestiary:search-fixture#parent");
        assert_eq!(parent.ordinal, 0);
        for expected in [
            "AC: 22",
            "Max HP: 80",
            "Perception: 15",
            "Fortitude: 14",
            "Arcana: 16",
            "Land Speed: 25",
            "Focus: 3",
            "Strike: Bolt",
            "Bolt Attack: +19",
            "Bolt bolt: 2d8 electricity",
            "Action: Pulse",
            "Pulse Action cost: 2 actions",
            "Spellcasting: Innate Spells",
            "Innate Spells Spell DC: 27",
            "Spell: Reactive Spell",
            "Reactive Spell Action cost: reaction",
            "Aliases: Alpha Alias, Zeta Alias",
            "Unique embedded tactical context",
        ] {
            assert!(
                parent.input_text.contains(expected),
                "missing canonical semantic input `{expected}` from:\n{}",
                parent.input_text
            );
        }
        assert!(!parent.input_text.contains("Copied canonical spell prose"));
        assert!(
            parent.input_text.find("AC: 22")
                < parent.input_text.find("Unique embedded tactical context")
        );
        assert_eq!(
            parent.input_hash,
            atlas_embedding::hash_document_embedding_input(&parent.input_text)
        );

        let mut mutated = loaded.clone();
        let RecordBody::Creature(creature) = mutated
            .facts
            .canonical_body
            .as_mut()
            .expect("canonical creature body");
        let FactValue::Value(defenses) = &mut creature.defenses.value else {
            panic!("defenses")
        };
        let FactValue::Value(armor_class) = &mut defenses.armor_class else {
            panic!("armor class")
        };
        armor_class.value = FactValue::Value(23);
        let changed = build_pending_document_embeddings(&[mutated], &aliases, &[]);
        assert!(changed[0].input_text.contains("AC: 23"));
        assert_ne!(changed[0].input_hash, parent.input_hash);

        let remaster_links = [RemasterLink {
            remaster_record_key: RecordKey::parse("bestiary:remaster")
                .expect("remaster key parses"),
            legacy_record_key: record_key,
            source: RemasterLinkSource::Migration,
            source_ref: "fixture".to_string(),
        }];
        assert!(build_pending_document_embeddings(&[loaded], &aliases, &remaster_links).is_empty());
    }

    #[test]
    fn summarizes_embedding_unit_fanout() {
        let mut record = base_record();
        record.content.documents.push(RecordContentDocument {
            source_kind: ContentSourceKind::Description,
            label: None,
            document: RichDocument::new(vec![
                html_element("h2", vec![text_node("First")]),
                html_element("p", vec![text_node("First section text")]),
                html_element("h2", vec![text_node("Second")]),
                html_element("p", vec![text_node("Second section text")]),
            ]),
        });
        let pending = build_pending_document_embeddings(
            &[crate::records::LoadedSourceRecord::new(
                record,
                crate::records::SourceConstructionFacts::empty(),
            )],
            &[],
            &[],
        );

        let summary = summarize_pending_document_embeddings(&pending);

        assert_eq!(summary.total_units, 1);
        assert_eq!(summary.parent_units, 1);
        assert_eq!(summary.child_units, 0);
        assert_eq!(summary.records_with_child_units, 0);
        assert_eq!(summary.max_child_units_per_record, 0);
    }

    fn text_document(text: &str) -> RichDocument {
        RichDocument::new(vec![html_element("p", vec![text_node(text)])])
    }

    fn owned_content_document(
        record_key: &RecordKey,
        content_key: &str,
        source_kind: ContentSourceKind,
        text: &str,
        duplicate_status: DuplicateContentStatus,
    ) -> OwnedRichContentDocument {
        OwnedRichContentDocument::new(
            ContentId::new(
                record_key.clone(),
                ContentKey::new(content_key).expect("content key parses"),
            ),
            ContentIdentityStability::StableSourceIdentity,
            ContentOwner::Record(record_key.clone()),
            ContentRole::SupplementalRules,
            ContentOrigin::RecordField {
                source_kind,
                relative_source_path: format!("$.{content_key}"),
            },
            source_kind.default_visibility(),
            ContentProvenance {
                source_record_key: record_key.clone(),
                relative_source_path: "packs/test.json".to_string(),
                field_or_pointer_family: format!("$.{content_key}"),
                nested_source_id: None,
                authored_ordinal_or_range: None,
                authored_label: None,
            },
            source_kind,
            0,
            None,
            text_document(text),
            duplicate_status,
            Vec::new(),
        )
    }

    fn html_element(tag: &str, children: Vec<RichNode>) -> RichNode {
        RichNode::HtmlElement {
            tag: tag.to_string(),
            attributes: BTreeMap::new(),
            children,
        }
    }

    fn text_node(text: &str) -> RichNode {
        RichNode::Text {
            text: text.to_string(),
        }
    }

    fn base_record() -> AtlasRecord {
        let pack_name = PackName::new("test-pack").expect("pack parses");
        let id = RecordId::new("TestRecord").expect("id parses");
        AtlasRecord::new(
            RecordIdentity::new(RecordKey::new(pack_name, id), "Test Record"),
            RecordClassification::new(RecordKind::Rule),
            FoundryRecordInfo::new(
                "Test Pack",
                FoundryDocumentType::Item,
                FoundryRecordType::Action,
            ),
            RecordProvenance::new("test.json").with_raw_json("{}"),
        )
    }

    fn canonical_loaded_fixture() -> crate::records::LoadedSourceRecord {
        let loaded = crate::source::normalize::normalize_record(
            &crate::source::ManifestPack {
                name: "bestiary".to_string(),
                label: "Bestiary".to_string(),
                document_type: "Actor".to_string(),
                path: "packs/bestiary".to_string(),
            },
            &PackName::new("bestiary").expect("pack name parses"),
            Path::new("packs/bestiary/search-fixture.json"),
            Path::new("."),
            json!({
                "_id":"search-fixture", "name":"Canonical Search Fixture", "type":"npc",
                "system": {
                    "details":{"level":{"value":5},"publication":{"title":"Fixture"}},
                    "attributes":{"ac":{"value":22},"hp":{"value":80,"max":80},"speed":{"value":25}},
                    "perception":{"mod":15},
                    "saves":{"fortitude":{"value":14},"reflex":{"value":12},"will":{"value":13}},
                    "skills":{"arcana":{"base":16}},
                    "resources":{"focus":{"max":3,"value":1}},
                    "traits":{"rarity":"common","size":{"value":"med"},"value":["fiend"]}
                },
                "items":[
                    {"_id":"action","name":"Pulse","type":"action","system":{"actionType":{"value":"action"},"actions":{"value":2},"bonus":{"value":17},"dc":{"value":26},"damageRolls":{"pulse":{"damage":"2d6","damageType":"mental"}}}},
                    {"_id":"strike","name":"Bolt","type":"melee","system":{"bonus":{"value":19},"damageRolls":{"bolt":{"damage":"2d8","damageType":"electricity"}}}},
                    {"_id":"entry","name":"Innate Spells","type":"spellcastingEntry","system":{"prepared":{"value":"innate"},"tradition":{"value":"occult"},"spelldc":{"value":18,"dc":27},"slots":{"slot4":{"max":2,"value":1}}}},
                    {"_id":"spell","name":"Reactive Spell","type":"spell","system":{"level":{"value":4},"location":{"value":"entry"},"time":{"value":"reaction"},"damage":{}}}
                ]
            }),
            None,
        )
        .expect("fixture normalizes");
        let mut records = vec![loaded];
        let reference_index = crate::records::references::build_record_reference_index(&records);
        crate::source::npc_entities::finalize_npc_embedded_entities(&mut records, &reference_index);
        crate::source::owned_content::finalize_npc_owned_content(&mut records);
        records.pop().expect("one canonical fixture")
    }
}
