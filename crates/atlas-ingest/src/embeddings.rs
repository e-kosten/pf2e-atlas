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
    match loaded.facts.canonical_body.as_ref()? {
        RecordBody::Creature(creature) => {
            Some(embedding_content_documents_from_owned(&creature.content))
        }
        RecordBody::Hazard(hazard) => Some(embedding_content_documents_from_owned(&hazard.content)),
        RecordBody::Spell(spell) => Some(embedding_content_documents_from_owned(
            &spell.definition.content,
        )),
        RecordBody::Journal(journal) => {
            Some(embedding_content_documents_from_owned(&journal.content))
        }
        RecordBody::RollTable(table) => {
            Some(embedding_content_documents_from_owned(&table.content))
        }
    }
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
    use std::path::{Path, PathBuf};

    use atlas_domain::{PackName, RecordId, RecordKey, RecordKind, RemasterLinkSource};
    use atlas_embedding::{
        DistanceMetric, EmbeddingModelSpec, Normalization, PoolingStrategy, TextEmbeddingTokenizer,
        VectorDType, apply_document_embedding_token_budget,
    };
    use atlas_record::{
        AliasSource, AtlasRecord, ContentId, ContentIdentityStability, ContentKey, ContentOrigin,
        ContentOwner, ContentProvenance, ContentRole, ContentSourceKind, DuplicateContentStatus,
        FactValue, FoundryDocumentType, FoundryRecordInfo, FoundryRecordType, OwnedRichContent,
        OwnedRichContentDocument, RecordAlias, RecordBody, RecordClassification,
        RecordContentDocument, RecordIdentity, RecordProvenance, RemasterLink, RichDocument,
        RichNode, SpellClassification, SpellDamageDiceRule, SpellIdentity, SpellProvenance,
        SpellRangeValue, SpellRecord, SpellRollOptionRule, SpellRule, SpellRuleElement,
        SpellSourceId, SpellSourceValue, SpellTargeting, SpellTradition, SpellTrait,
    };
    use serde_json::{Value, json};

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
    fn canonical_spell_embedding_uses_spell_owned_content_not_legacy_record_content() {
        let mut record = base_record();
        record.classification.kind = RecordKind::Spell;
        record.content.documents.push(RecordContentDocument {
            source_kind: ContentSourceKind::Description,
            label: None,
            document: text_document("legacy spell prose must not leak"),
        });
        let record_key = record.identity.key.clone();
        let mut spell = SpellRecord::new(
            SpellIdentity {
                record_key: record_key.clone(),
                source_id: SpellSourceId::new("TestRecord").expect("source id"),
                name: "Canonical Spell".to_string(),
            },
            SpellProvenance {
                source_path: "packs/spells/test.json".to_string(),
                source_contract_version: "fixture".to_string(),
                source_system_version: "6.12.4".to_string(),
                source_upstream_commit: "fixture".to_string(),
                standalone_location: FactValue::Null,
            },
        );
        spell
            .definition
            .content
            .documents
            .push(owned_content_document(
                &record_key,
                "description",
                ContentSourceKind::Description,
                "canonical spell prose",
                DuplicateContentStatus::Unique,
            ));
        spell.definition.classification =
            FactValue::Value(SpellSourceValue::Known(SpellClassification {
                rank: FactValue::Value(SpellSourceValue::Known(3)),
                traits: FactValue::Value(SpellSourceValue::Known(vec![
                    SpellTrait::new("teleportation").expect("trait"),
                ])),
                traditions: FactValue::Value(SpellSourceValue::Known(vec![
                    SpellTradition::new("occult").expect("tradition"),
                ])),
            }));
        spell.definition.targeting = FactValue::Value(SpellSourceValue::Known(SpellTargeting {
            target: FactValue::Value(SpellSourceValue::Known("1 willing ally".to_string())),
            range: FactValue::Value(SpellSourceValue::Known(
                SpellRangeValue::from_authored_text("30 feet"),
            )),
            area: FactValue::Missing,
        }));
        spell.definition.rules = FactValue::Value(SpellSourceValue::Known(vec![
            SpellRuleElement {
                authored_order: 0,
                source_path: "system.rules.0".to_string(),
                authored_key: "RollOption".to_string(),
                authored_object_json:
                    r#"{"key":"RollOption","option":"heavens-thunder","img":"raw-secret.webp","license":"raw-secret-license"}"#
                        .to_string(),
                rule: SpellRule::RollOption(SpellRollOptionRule {
                    option: FactValue::Value(SpellSourceValue::Known(
                        "heavens-thunder".to_string(),
                    )),
                    toggleable: FactValue::Value(SpellSourceValue::Known(true)),
                    ..SpellRollOptionRule::default()
                }),
            },
            SpellRuleElement {
                authored_order: 1,
                source_path: "system.rules.1".to_string(),
                authored_key: "DamageDice".to_string(),
                authored_object_json:
                    r#"{"key":"DamageDice","hideIfDisabled":true,"raw":"raw-dice-secret"}"#
                        .to_string(),
                rule: SpellRule::DamageDice(SpellDamageDiceRule {
                    hide_if_disabled: FactValue::Value(SpellSourceValue::Known(true)),
                    ..SpellDamageDiceRule::default()
                }),
            },
        ]));
        let mut facts = crate::records::SourceConstructionFacts::empty();
        facts.canonical_body = Some(RecordBody::Spell(spell.clone()));

        let loaded = crate::records::LoadedSourceRecord::new(record.clone(), facts);
        let pending = build_pending_document_embeddings(std::slice::from_ref(&loaded), &[], &[]);
        assert_eq!(pending.len(), 1);
        assert!(pending[0].input_text.contains("canonical spell prose"));
        assert!(pending[0].input_text.contains("Rank: 3"));
        assert!(pending[0].input_text.contains("Range: 30 feet"));
        assert!(pending[0].input_text.contains("Traditions: occult"));
        assert!(
            pending[0]
                .input_text
                .contains("Roll option rule: heavens-thunder toggleable true")
        );
        assert!(
            pending[0]
                .input_text
                .contains("Damage dice rule: hide if disabled true")
        );
        assert!(!pending[0].input_text.contains("legacy spell prose"));
        assert!(!pending[0].input_text.contains("raw-secret"));

        let unchanged = build_pending_document_embeddings(std::slice::from_ref(&loaded), &[], &[]);
        assert_eq!(unchanged[0].input_hash, pending[0].input_hash);

        let mut provenance_only = loaded.clone();
        let RecordBody::Spell(provenance_spell) = provenance_only
            .facts
            .canonical_body
            .as_mut()
            .expect("spell body")
        else {
            panic!("spell body")
        };
        provenance_spell.definition.source_context.image =
            FactValue::Value(SpellSourceValue::Known("icons/new-image.webp".to_string()));
        let provenance_input = build_pending_document_embeddings(&[provenance_only], &[], &[]);
        assert_eq!(provenance_input[0].input_hash, pending[0].input_hash);
        assert!(!provenance_input[0].input_text.contains("new-image"));

        let mut raw_only = loaded.clone();
        let RecordBody::Spell(raw_spell) =
            raw_only.facts.canonical_body.as_mut().expect("spell body")
        else {
            panic!("spell body")
        };
        let FactValue::Value(SpellSourceValue::Known(raw_rules)) = &mut raw_spell.definition.rules
        else {
            panic!("rules")
        };
        raw_rules[0].authored_object_json =
            r#"{"key":"RollOption","raw":"changed-raw-secret"}"#.to_string();
        let raw_input = build_pending_document_embeddings(&[raw_only], &[], &[]);
        assert_eq!(raw_input[0].input_hash, pending[0].input_hash);
        assert!(!raw_input[0].input_text.contains("changed-raw-secret"));

        let mut semantic_change = loaded.clone();
        let RecordBody::Spell(changed_spell) = semantic_change
            .facts
            .canonical_body
            .as_mut()
            .expect("spell body")
        else {
            panic!("spell body")
        };
        let FactValue::Value(SpellSourceValue::Known(rules)) = &mut changed_spell.definition.rules
        else {
            panic!("rules")
        };
        let SpellRule::RollOption(rule) = &mut rules[0].rule else {
            panic!("roll option")
        };
        rule.toggleable = FactValue::Value(SpellSourceValue::Known(false));
        let changed = build_pending_document_embeddings(&[semantic_change], &[], &[]);
        assert!(
            changed[0]
                .input_text
                .contains("Roll option rule: heavens-thunder toggleable false")
        );
        assert_ne!(changed[0].input_hash, pending[0].input_hash);

        let mut damage_dice_change = loaded;
        let RecordBody::Spell(changed_spell) = damage_dice_change
            .facts
            .canonical_body
            .as_mut()
            .expect("spell body")
        else {
            panic!("spell body")
        };
        let FactValue::Value(SpellSourceValue::Known(rules)) = &mut changed_spell.definition.rules
        else {
            panic!("rules")
        };
        let SpellRule::DamageDice(rule) = &mut rules[1].rule else {
            panic!("damage dice")
        };
        rule.hide_if_disabled = FactValue::Value(SpellSourceValue::Known(false));
        let changed = build_pending_document_embeddings(&[damage_dice_change], &[], &[]);
        assert!(
            changed[0]
                .input_text
                .contains("Damage dice rule: hide if disabled false")
        );
        assert_ne!(changed[0].input_hash, pending[0].input_hash);
    }

    #[test]
    fn canonical_source_record_builds_stable_search_units_and_remaster_demotion() {
        let mut loaded = canonical_loaded_fixture();
        assert!(loaded.record.mechanics.metrics.is_empty());
        let record_key = loaded.record.identity.key.clone();
        let creature = loaded
            .facts
            .canonical_body
            .as_mut()
            .and_then(RecordBody::creature_mut)
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
        let creature = mutated
            .facts
            .canonical_body
            .as_mut()
            .and_then(RecordBody::creature_mut)
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
    fn hidden_pit_uses_canonical_hazard_semantic_input_without_source_only_metadata() {
        let loaded = hazard_loaded_fixture("hazards", "packs/hazards/hidden-pit.json");
        let first = build_pending_document_embeddings(std::slice::from_ref(&loaded), &[], &[]);
        let second = build_pending_document_embeddings(&[loaded], &[], &[]);

        assert_eq!(first, second);
        assert_eq!(first.len(), 1);
        let parent = &first[0];
        for expected in [
            "Name: Hidden Pit",
            "Level: 0",
            "Complexity: Simple",
            "Stealth: +8",
            "Detection DC: 18",
            "Armor Class: 10",
            "Maximum Hit Points: 12",
            "Broken Threshold: 6",
            "Pitfall",
            "Trigger A creature walks onto the trapdoor.",
            "Effect The triggering creature falls in",
        ] {
            assert!(
                parent.input_text.contains(expected),
                "missing canonical hazard input `{expected}` from:\n{}",
                parent.input_text
            );
        }
        assert!(!parent.input_text.contains("ORC"));
        assert!(!parent.input_text.contains("OGL"));
        assert_eq!(
            parent.input_hash,
            atlas_embedding::hash_document_embedding_input(&parent.input_text)
        );
    }

    #[test]
    fn dragon_pillar_budgeted_units_invalidate_only_changed_owned_content() {
        let relative = "packs/age-of-ashes-bestiary/book-2-cult-of-cinders/dragon-pillar.json";
        let original = hazard_loaded_fixture("age-of-ashes-bestiary", relative);
        let mut raw: Value = serde_json::from_slice(
            &std::fs::read(hazard_fixture_root().join(relative)).expect("dragon fixture"),
        )
        .expect("dragon fixture JSON");
        let description = raw
            .pointer_mut("/system/details/description")
            .and_then(|value| value.as_str())
            .expect("hazard description")
            .to_string();
        *raw.pointer_mut("/system/details/description")
            .expect("hazard description") =
            Value::String(format!("{description}<p>Changed unit sentinel.</p>"));
        let changed = hazard_loaded_raw("age-of-ashes-bestiary", relative, raw);

        let mut original_pending = build_pending_document_embeddings(&[original], &[], &[]);
        let mut changed_pending = build_pending_document_embeddings(&[changed], &[], &[]);
        let tokenizer = deterministic_token_budget_tokenizer(96);
        apply_document_embedding_token_budget(&mut original_pending, &tokenizer)
            .expect("original Dragon Pillar token budget");
        apply_document_embedding_token_budget(&mut changed_pending, &tokenizer)
            .expect("changed Dragon Pillar token budget");

        let original_units = original_pending
            .iter()
            .map(|unit| {
                (
                    unit.embedding_unit_key.as_str(),
                    (unit.input_hash.as_str(), unit.input_text.as_str()),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let changed_units = changed_pending
            .iter()
            .map(|unit| {
                (
                    unit.embedding_unit_key.as_str(),
                    (unit.input_hash.as_str(), unit.input_text.as_str()),
                )
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            original_units.keys().collect::<Vec<_>>(),
            changed_units.keys().collect::<Vec<_>>()
        );
        assert!(
            original_pending
                .iter()
                .any(|unit| { unit.unit_kind != atlas_embedding::EmbeddingUnitKind::Parent })
        );

        let invalidated = original_units
            .iter()
            .filter(|(key, (hash, _))| changed_units[*key].0 != *hash)
            .map(|(key, _)| *key)
            .collect::<Vec<_>>();
        assert_eq!(
            invalidated.len(),
            1,
            "unexpected invalidated units: {invalidated:?}"
        );
        assert_ne!(
            invalidated[0], "age-of-ashes-bestiary:zNIjGSxkG8xyDLgR#parent",
            "the owned-content mutation must invalidate its emitted child, not the budgeted parent"
        );
        let invalidated_unit = changed_units[invalidated[0]];
        assert!(invalidated_unit.1.contains("Changed unit sentinel"));
        assert_eq!(
            original_units["age-of-ashes-bestiary:zNIjGSxkG8xyDLgR#parent"].0,
            changed_units["age-of-ashes-bestiary:zNIjGSxkG8xyDLgR#parent"].0,
            "the unaffected emitted parent remains reusable"
        );
        assert!(
            original_units.iter().any(|(key, (hash, _))| {
                !invalidated.contains(key) && changed_units[key].0 == *hash
            }),
            "at least one emitted sibling must retain its reusable key/hash"
        );
        for (_, input_text) in original_units.values().chain(changed_units.values()) {
            for source_only in ["ORC", "OGL", "systems/pf2e/icons/"] {
                assert!(!input_text.contains(source_only));
            }
        }
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

    fn hazard_loaded_fixture(pack: &str, relative: &str) -> crate::records::LoadedSourceRecord {
        let source_root = hazard_fixture_root();
        let serialized = std::fs::read(source_root.join(relative)).expect("hazard fixture");
        let loaded = crate::source::normalize::normalize_record_from_source_bytes(
            &crate::source::ManifestPack {
                name: pack.to_string(),
                label: pack.to_string(),
                document_type: "Actor".to_string(),
                path: format!("packs/{pack}"),
            },
            &PackName::new(pack.to_string()).expect("pack"),
            &source_root.join(relative),
            &source_root,
            &serialized,
            None,
        )
        .expect("hazard normalization");
        finalize_hazard_fixture(loaded)
    }

    fn hazard_loaded_raw(
        pack: &str,
        relative: &str,
        raw: Value,
    ) -> crate::records::LoadedSourceRecord {
        let source_root = hazard_fixture_root();
        let loaded = crate::source::normalize::normalize_record(
            &crate::source::ManifestPack {
                name: pack.to_string(),
                label: pack.to_string(),
                document_type: "Actor".to_string(),
                path: format!("packs/{pack}"),
            },
            &PackName::new(pack.to_string()).expect("pack"),
            &source_root.join(relative),
            &source_root,
            raw,
            None,
        )
        .expect("hazard normalization");
        finalize_hazard_fixture(loaded)
    }

    fn finalize_hazard_fixture(
        loaded: crate::records::LoadedSourceRecord,
    ) -> crate::records::LoadedSourceRecord {
        let mut records = vec![loaded];
        crate::source::owned_content::finalize_hazard_owned_content(&mut records);
        records.pop().expect("one hazard fixture")
    }

    fn hazard_fixture_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hazards/pinned")
    }

    fn deterministic_token_budget_tokenizer(max_input_tokens: usize) -> TextEmbeddingTokenizer {
        TextEmbeddingTokenizer::load_from_model_dir(
            EmbeddingModelSpec {
                provider_family: "test",
                model_id: "test-wordlevel",
                model_revision: "test",
                tokenizer_id: "test-wordlevel",
                max_input_tokens: Some(max_input_tokens),
                pooling: PoolingStrategy::Mean,
                normalization: Normalization::L2,
                dimensions: 1,
                dtype: VectorDType::F32,
                distance_metric: DistanceMetric::Cosine,
                document_prefix: "",
                query_prefix: "",
            },
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/embedding-tokenizer"),
        )
        .expect("deterministic no-model tokenizer fixture")
    }
}
