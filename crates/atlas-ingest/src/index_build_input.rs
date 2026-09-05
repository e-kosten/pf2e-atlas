use atlas_index::{IndexBuildInput, IndexBuildPack};

use crate::source::SourceLoad;

pub(crate) fn index_build_input(source: SourceLoad) -> IndexBuildInput {
    let mut records = Vec::with_capacity(source.records.len());
    let mut canonical_bodies = Vec::new();
    for loaded in source.records {
        records.push(loaded.record);
        canonical_bodies.extend(loaded.facts.canonical_body);
    }
    IndexBuildInput {
        source_signature: source.source_signature,
        source_record_count: source.source_record_count,
        packs: source
            .packs
            .into_iter()
            .map(|pack| IndexBuildPack {
                name: pack.name,
                label: pack.label,
                document_type: pack.document_type,
                declared_path: pack.declared_path,
                resolved_path: pack.resolved_path,
                record_count: pack.record_count,
            })
            .collect(),
        records,
        canonical_bodies,
        references: source.references,
        aliases: source.aliases,
        remaster_links: source.remaster_links,
        pending_document_embeddings: source.pending_document_embeddings,
        document_embeddings: source.document_embeddings,
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use atlas_domain::{PackName, RecordId, RecordKey, RecordKind, RemasterLinkSource};
    use atlas_embedding::{
        DocumentEmbeddingTokenizationTelemetry, EmbeddingUnitKind, GeneratedDocumentEmbedding,
        PendingDocumentEmbedding,
    };
    use atlas_record::{
        AliasSource, AtlasRecord, ContentExclusion, ContentExclusionReason, ContentKey,
        ContentSourceKind, ContentVisibility, FoundryDocumentType, FoundryRecordInfo,
        FoundryRecordType, RecordAlias, RecordBody, RecordClassification, RecordIdentity,
        RecordProvenance, ReferenceEdge, RemasterLink,
    };

    use super::index_build_input;
    use crate::diagnostics::IngestDiagnostics;
    use crate::records::references::{build_record_reference_index, resolve_content_references};
    use crate::records::{LoadedSourceRecord, SourceConstructionFacts};
    use crate::source::normalize::normalize_record;
    use crate::source::npc_entities::finalize_npc_embedded_entities;
    use crate::source::owned_content::finalize_npc_owned_content;
    use crate::source::{LoadedPack, SourceLoad};
    use serde_json::json;
    use sha2::{Digest, Sha256};

    #[test]
    fn ordinary_fixture_rejects_wrong_valid_relational_values_extra_rows_and_hydration_gaps()
    -> Result<(), Box<dyn std::error::Error>> {
        let input = canonical_fixture_input();
        let path = unique_temp_path("ordinary-canonical-coherence.sqlite");
        atlas_index::IndexArtifactWriter::write(
            &atlas_index::SqliteIndexWriter::new(path.clone()),
            &input,
            atlas_embedding::EmbeddingModelId::BgeSmallEnV15,
        )?;
        write_test_manifest(&path)?;
        let reader = atlas_index::SqliteIndexReader::open_read_only(&path)?;
        assert_eq!(reader.validate()?.status, atlas_index::ValidationStatus::Ok);
        assert!(
            reader
                .load_hydrated_records()?
                .iter()
                .any(|row| row.body.is_some())
        );
        let npc_key = input.records[0].identity.key.clone();
        assert_eq!(
            reader
                .load_hydrated_records_by_key(std::slice::from_ref(&npc_key))?
                .len(),
            1
        );

        // Enum admission is covered in atlas-index's focused visibility and
        // retrieval-policy mutation test. The remaining canonical faults are
        // exercised here through the production writer and reader.
        let cases = [
            (
                "wrong-target",
                "UPDATE canonical_creature_occurrences SET target_entity_id=(SELECT entity_id FROM canonical_creature_entities e WHERE e.record_key=canonical_creature_occurrences.record_key AND e.entity_id<>canonical_creature_occurrences.target_entity_id ORDER BY entity_id LIMIT 1) WHERE family='action' AND authored_order=(SELECT MIN(authored_order) FROM canonical_creature_occurrences WHERE family='action')",
            ),
            (
                "wrong-parent",
                "UPDATE canonical_creature_occurrences SET parent_kind='spellcasting_entry', parent_occurrence_id=(SELECT occurrence_id FROM canonical_creature_occurrences WHERE family='spellcasting-entry' LIMIT 1), parent_occurrence_authored_order=(SELECT authored_order FROM canonical_creature_occurrences WHERE family='spellcasting-entry' LIMIT 1) WHERE family='action' AND authored_order=(SELECT MIN(authored_order) FROM canonical_creature_occurrences WHERE family='action')",
            ),
            (
                "wrong-owner",
                "UPDATE record_content SET owner_entity_id=(SELECT entity_id FROM canonical_creature_entities e WHERE e.record_key=record_content.record_key AND e.entity_id<>record_content.owner_entity_id ORDER BY entity_id LIMIT 1) WHERE owner_kind='creature_entity' AND rowid=(SELECT rowid FROM record_content WHERE owner_kind='creature_entity' LIMIT 1)",
            ),
            (
                "wrong-reference",
                "UPDATE reference_occurrences SET relation_kind=CASE relation_kind WHEN 'reference' THEN 'embed' ELSE 'reference' END WHERE rowid=(SELECT rowid FROM reference_occurrences LIMIT 1)",
            ),
            (
                "wrong-content",
                "UPDATE record_content SET visibility=CASE visibility WHEN 'public' THEN 'gm_only' ELSE 'public' END WHERE rowid=(SELECT rowid FROM record_content LIMIT 1)",
            ),
            (
                "wrong-exclusion",
                "UPDATE record_content_exclusions SET reason=CASE reason WHEN 'missing_typed_owner' THEN 'deferred_entity_family' ELSE 'missing_typed_owner' END WHERE rowid=(SELECT rowid FROM record_content_exclusions LIMIT 1)",
            ),
            (
                "wrong-metric",
                "UPDATE record_metrics SET number_value=number_value+1 WHERE value_type='number' AND rowid=(SELECT rowid FROM record_metrics WHERE value_type='number' LIMIT 1)",
            ),
            (
                "extra-row",
                "INSERT INTO record_metrics(record_key,metric_domain,metric_key,value_type,text_value) SELECT record_key,'actor','fixture.extra','text','extra' FROM canonical_creature_records LIMIT 1",
            ),
            (
                "typed-json",
                "UPDATE canonical_creature_occurrences SET context_json='{}' WHERE rowid=(SELECT rowid FROM canonical_creature_occurrences LIMIT 1)",
            ),
        ];
        for (name, sql) in cases {
            assert_corruption_detected(&path, name, sql, "exact relational projection")?;
        }

        assert_corruption_detected(
            &path,
            "orphan",
            "PRAGMA foreign_keys=OFF; UPDATE canonical_creature_occurrences SET target_entity_id='missing-owner' WHERE target_kind='actor_owned' AND rowid=(SELECT rowid FROM canonical_creature_occurrences WHERE target_kind='actor_owned' LIMIT 1)",
            "foreign key",
        )?;
        for (name, sql) in [
            (
                "wrong-canonical-relationship",
                "UPDATE canonical_creature_relationships SET relationship_kind=CASE relationship_kind WHEN 'granted_by' THEN 'item_grant' ELSE 'granted_by' END WHERE rowid=(SELECT rowid FROM canonical_creature_relationships ORDER BY record_key,relationship_order LIMIT 1)",
            ),
            (
                "wrong-authored-order",
                "PRAGMA foreign_keys=OFF; UPDATE canonical_creature_occurrences SET authored_order=999999 WHERE rowid=(SELECT rowid FROM canonical_creature_occurrences ORDER BY record_key LIMIT 1)",
            ),
            (
                "wrong-entity-label",
                "UPDATE canonical_creature_entities SET label=label || ' corrupt' WHERE rowid=(SELECT rowid FROM canonical_creature_entities ORDER BY record_key LIMIT 1)",
            ),
        ] {
            assert_corruption_detected(&path, name, sql, "exact relational projection")?;
        }

        let duplicate = copy_for_corruption(&path, "duplicate-entity")?;
        let connection = rusqlite::Connection::open(&duplicate)?;
        let (duplicate_record_key, canonical_json): (String, String) = connection.query_row(
            "SELECT c.record_key, c.canonical_json FROM canonical_creature_records c WHERE EXISTS (SELECT 1 FROM canonical_creature_entities e WHERE e.record_key=c.record_key) ORDER BY c.record_key LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        let mut value: serde_json::Value = serde_json::from_str(&canonical_json)?;
        let entities = value
            .pointer_mut("/value/embedded_entities/value/value/entities")
            .and_then(serde_json::Value::as_array_mut)
            .expect("canonical entity array");
        entities.push(entities.first().expect("fixture entity").clone());
        connection.execute(
            "UPDATE canonical_creature_records SET canonical_json=?1 WHERE record_key=?2",
            rusqlite::params![serde_json::to_string(&value)?, duplicate_record_key],
        )?;
        drop(connection);
        assert_validation_message(&duplicate, "duplicate canonical entity ID", false)?;

        let missing = copy_for_corruption(&path, "missing-body")?;
        rusqlite::Connection::open(&missing)?.execute_batch(
            "PRAGMA foreign_keys=OFF;
             DELETE FROM reference_occurrences;
             DELETE FROM canonical_creature_relationships;
             DELETE FROM record_content;
             DELETE FROM record_content_exclusions;
             DELETE FROM canonical_creature_occurrences;
             DELETE FROM canonical_creature_entities;
             DELETE FROM canonical_creature_resources;
             DELETE FROM canonical_creature_records;",
        )?;
        write_test_manifest(&missing)?;
        let missing_reader = atlas_index::SqliteIndexReader::open_read_only(&missing)?;
        let missing_report = missing_reader.validate()?;
        assert_eq!(missing_report.status, atlas_index::ValidationStatus::Error);
        assert!(missing_report.diagnostics.iter().any(|diagnostic| {
            diagnostic.key.as_deref() == Some("canonical_creature_records.missing_npc_body")
        }));
        assert!(
            missing_reader
                .load_hydrated_records()
                .unwrap_err()
                .to_string()
                .contains("missing its required creature body")
        );
        assert!(
            missing_reader
                .load_hydrated_records_by_key(std::slice::from_ref(&npc_key))
                .unwrap_err()
                .to_string()
                .contains("missing its required creature body")
        );
        remove_test_artifact(&missing)?;

        let extra = copy_for_corruption(&path, "extra-body")?;
        rusqlite::Connection::open(&extra)?.execute(
            "UPDATE records SET foundry_record_type='action' WHERE record_key=?1",
            [npc_key.to_string()],
        )?;
        write_test_manifest(&extra)?;
        let extra_reader = atlas_index::SqliteIndexReader::open_read_only(&extra)?;
        let extra_report = extra_reader.validate()?;
        assert_eq!(extra_report.status, atlas_index::ValidationStatus::Error);
        assert!(extra_report.diagnostics.iter().any(|diagnostic| {
            diagnostic.key.as_deref() == Some("canonical_creature_records.non_npc_body")
        }));
        assert!(
            extra_reader
                .load_hydrated_records()
                .unwrap_err()
                .to_string()
                .contains("unexpected canonical creature body")
        );
        assert!(
            extra_reader
                .load_hydrated_records_by_key(std::slice::from_ref(&npc_key))
                .unwrap_err()
                .to_string()
                .contains("unexpected canonical creature body")
        );
        remove_test_artifact(&extra)?;
        remove_test_artifact(&path)?;
        Ok(())
    }

    #[test]
    fn canonical_mechanics_drive_the_written_fts_baseline_contract()
    -> Result<(), Box<dyn std::error::Error>> {
        let input = canonical_fixture_input();
        assert!(
            input.records[0].mechanics.metrics.is_empty(),
            "index input must not inject canonical creature facts into generic record metrics"
        );
        let path = unique_temp_path("canonical-search-projection.sqlite");
        atlas_index::IndexArtifactWriter::write(
            &atlas_index::SqliteIndexWriter::new(path.clone()),
            &input,
            atlas_embedding::EmbeddingModelId::BgeSmallEnV15,
        )?;
        let connection = rusqlite::Connection::open(&path)?;
        let (mechanic_terms, metric_terms): (String, String) = connection.query_row(
            "SELECT mechanic_terms, metric_terms FROM records_fts WHERE record_key='bestiary:actor'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        for expected in [
            "AC 22",
            "Max HP 80",
            "Perception 15",
            "Fortitude 14",
            "Arcana 16",
            "Land Speed 25",
            "Focus 3",
            "Pulse",
            "2 actions",
            "Bolt",
            "2d8 electricity",
            "Innate Spells",
            "Spell Attack 18",
            "Spell DC 27",
            "Reactive Spell",
            "reaction",
        ] {
            assert!(
                mechanic_terms.contains(expected),
                "missing canonical FTS term `{expected}` from:\n{mechanic_terms}"
            );
        }
        for expected in [
            "AC",
            "Max HP",
            "Perception",
            "Fortitude",
            "Arcana",
            "Land Speed",
            "Focus",
            "Bolt",
            "Pulse",
            "Innate Spells",
            "Reactive Spell",
        ] {
            assert!(
                metric_terms.contains(expected),
                "missing canonical metric term `{expected}` from:\n{metric_terms}"
            );
        }
        let matched: String = connection.query_row(
            "SELECT record_key FROM records_fts WHERE records_fts MATCH 'spell AND dc'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(matched, "bestiary:actor");
        drop(connection);
        remove_test_artifact(&path)?;
        Ok(())
    }

    fn assert_corruption_detected(
        source: &std::path::Path,
        name: &str,
        sql: &str,
        expected_message: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = copy_for_corruption(source, name)?;
        let connection = rusqlite::Connection::open(&path)?;
        connection
            .execute_batch(sql)
            .map_err(|error| format!("corruption fixture `{name}` failed to apply: {error}"))?;
        drop(connection);
        assert_validation_message(&path, expected_message, name == "orphan")
    }

    fn assert_validation_message(
        path: &std::path::Path,
        expected_message: &str,
        full_validation: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        write_test_manifest(path)?;
        let reader = atlas_index::SqliteIndexReader::open_read_only(path)?;
        let diagnostics = if full_validation {
            let report = reader.validate()?;
            assert_eq!(report.status, atlas_index::ValidationStatus::Error);
            report.diagnostics
        } else {
            reader.validate_canonical_coherence()?
        };
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains(expected_message)),
            "expected `{expected_message}` for {} in {:#?}",
            path.display(),
            diagnostics
        );
        remove_test_artifact(path)?;
        Ok(())
    }

    fn copy_for_corruption(
        source: &std::path::Path,
        name: &str,
    ) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        let target = unique_temp_path(&format!("canonical-corruption-{name}.sqlite"));
        std::fs::copy(source, &target)?;
        Ok(target)
    }

    fn unique_temp_path(name: &str) -> std::path::PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("atlas-{nonce}-{name}"));
        std::fs::create_dir_all(&root).expect("temporary artifact directory");
        root.join("index.sqlite")
    }

    fn write_test_manifest(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let hash = format!("{:x}", Sha256::digest(std::fs::read(path)?));
        std::fs::write(
            path.parent()
                .unwrap_or_else(|| Path::new("."))
                .join("manifest.json"),
            format!(
                r#"{{"manifest_version":"{}","artifact_contract_version":"{}","schema_version":"{}","build":{{"artifact_sha256":"{hash}"}}}}"#,
                atlas_index::ARTIFACT_MANIFEST_VERSION,
                atlas_index::ARTIFACT_CONTRACT_VERSION,
                atlas_index::ARTIFACT_SCHEMA_VERSION,
            ),
        )?;
        Ok(())
    }

    fn remove_test_artifact(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::remove_dir_all(path.parent().expect("test artifact parent"))?;
        Ok(())
    }

    #[test]
    fn maps_source_load_to_index_build_input_without_changing_boundaries()
    -> Result<(), Box<dyn std::error::Error>> {
        let source_record = record("source-pack", "source-record", RecordKind::Rule);
        let generated_record = record("generated-pack", "generated-record", RecordKind::Rule);
        let source_key = source_record.identity.key.clone();
        let generated_key = generated_record.identity.key.clone();
        let source = SourceLoad {
            manifest_path: PathBuf::from("manifest.json"),
            source_signature: "foundry-pf2e:sha256:test".to_string(),
            source_record_count: 1,
            packs: vec![LoadedPack {
                name: PackName::new("source-pack").expect("pack name parses"),
                label: "Source Pack".to_string(),
                document_type: "Item".to_string(),
                declared_path: "packs/source-pack".to_string(),
                resolved_path: PathBuf::from("/tmp/source-pack"),
                record_count: 1,
            }],
            records: vec![
                LoadedSourceRecord::new(source_record, SourceConstructionFacts::empty()),
                LoadedSourceRecord::new(generated_record, SourceConstructionFacts::empty()),
            ],
            references: vec![ReferenceEdge {
                from_record_key: source_key.clone(),
                to_record_key: generated_key.clone(),
                display_text: Some("Generated Record".to_string()),
                reference_text: "@UUID[generated]".to_string(),
                relation_kind: atlas_record::ReferenceRelationKind::Reference,
                source_kind: ContentSourceKind::Description,
                visibility: ContentVisibility::Public,
            }],
            aliases: vec![RecordAlias {
                canonical_record_key: source_key.clone(),
                alias_text: "Source Alias".to_string(),
                normalized_alias: "source alias".to_string(),
                source: AliasSource::Migration,
                source_ref: "migration".to_string(),
            }],
            remaster_links: vec![RemasterLink {
                remaster_record_key: source_key.clone(),
                legacy_record_key: generated_key.clone(),
                source: RemasterLinkSource::Migration,
                source_ref: "migration".to_string(),
            }],
            pending_document_embeddings: vec![PendingDocumentEmbedding::prepared(
                "source-pack.source-record#parent".to_string(),
                source_key.to_string(),
                EmbeddingUnitKind::Parent,
                None,
                0,
                "source record".to_string(),
                "pending-hash".to_string(),
            )],
            document_embeddings: vec![GeneratedDocumentEmbedding {
                embedding_unit_key: "source-pack.source-record#parent".to_string(),
                record_key: source_key.to_string(),
                unit_kind: EmbeddingUnitKind::Parent,
                label: None,
                ordinal: 0,
                input_hash: "generated-hash".to_string(),
                dimensions: 3,
                vector: vec![0.1, 0.2, 0.3],
            }],
            document_embedding_tokenization: DocumentEmbeddingTokenizationTelemetry::default(),
            diagnostics: IngestDiagnostics::default(),
            skipped_records: Vec::new(),
            warnings: Vec::new(),
        };

        let input = index_build_input(source);

        assert_eq!(input.source_signature, "foundry-pf2e:sha256:test");
        assert_eq!(input.source_record_count, 1);
        assert_eq!(input.artifact_record_count(), 2);
        assert_eq!(input.generated_record_count()?, 1);

        assert_eq!(input.packs.len(), 1);
        assert_eq!(input.packs[0].name.as_str(), "source-pack");
        assert_eq!(input.packs[0].label, "Source Pack");
        assert_eq!(input.packs[0].document_type, "Item");
        assert_eq!(input.packs[0].declared_path, "packs/source-pack");
        assert_eq!(
            input.packs[0].resolved_path,
            PathBuf::from("/tmp/source-pack")
        );
        assert_eq!(input.packs[0].record_count, 1);

        assert_eq!(input.records.len(), 2);
        assert_eq!(input.records[0].identity.key, source_key);
        assert_eq!(input.records[1].identity.key, generated_key);
        assert!(input.canonical_bodies.is_empty());

        assert_eq!(input.references.len(), 1);
        assert_eq!(input.aliases.len(), 1);
        assert_eq!(input.remaster_links.len(), 1);
        assert_eq!(input.pending_document_embeddings.len(), 1);
        assert_eq!(input.document_embeddings.len(), 1);
        Ok(())
    }

    fn canonical_fixture_input() -> atlas_index::IndexBuildInput {
        let pack_name = PackName::new("bestiary").expect("pack name");
        let loaded = normalize_record(
            &crate::source::ManifestPack {
                name: "bestiary".to_string(), label: "Bestiary".to_string(), document_type: "Actor".to_string(), path: "packs/bestiary".to_string(),
            },
            &pack_name,
            Path::new("packs/bestiary/actor.json"),
            Path::new("."),
            json!({
                "_id":"actor", "name":"Coherence Creature", "type":"npc",
                "system": {
                    "details":{"level":{"value":5},"publication":{"title":"Fixture"}},
                    "attributes":{"ac":{"value":22},"hp":{"value":80,"max":80},"speed":{"value":25}},
                    "perception":{"mod":15}, "saves":{"fortitude":{"value":14},"reflex":{"value":12},"will":{"value":13}},
                    "skills":{"arcana":{"base":16}},
                    "resources":{"focus":{"max":3,"value":1}},
                    "traits":{"rarity":"common","size":{"value":"med"},"value":["fiend"]},
                    "description":{"value":"<p>@UUID[Compendium.pf2e.bestiary.Actor.actor]{Self}</p>"}
                },
                "items":[
                    {"_id":"action-a","name":"Pulse","type":"action","system":{"actionType":{"value":"action"},"actions":{"value":2},"bonus":{"value":17},"dc":{"value":26},"damageRolls":{"pulse":{"damage":"2d6","damageType":"mental"}},"description":{"value":"<p>First.</p>"}}},
                    {"_id":"action-b","name":"Second Action","type":"action","system":{"actionType":{"value":"action"},"actions":{"value":1},"description":{"value":"<p>Second.</p>"}}},
                    {"_id":"strike","name":"Bolt","type":"melee","system":{"bonus":{"value":19},"damageRolls":{"bolt":{"damage":"2d8","damageType":"electricity"}}}},
                    {"_id":"entry","name":"Innate Spells","type":"spellcastingEntry","system":{"prepared":{"value":"prepared"},"tradition":{"value":"occult"},"spelldc":{"value":18,"dc":27},"slots":{"slot4":{"max":2,"value":1,"prepared":[{"id":"spell","expended":false}]}}}},
                    {"_id":"spell","name":"Reactive Spell","type":"spell","system":{"level":{"value":4},"location":{"value":"entry"},"time":{"value":"reaction"},"damage":{}}}
                ]
            }),
            None,
        ).expect("fixture normalizes");
        let mut records = vec![loaded];
        let reference_index = build_record_reference_index(&records);
        finalize_npc_embedded_entities(&mut records, &reference_index);
        finalize_npc_owned_content(&mut records);
        resolve_content_references(&mut records, &reference_index);
        let RecordBody::Creature(creature) = records[0]
            .facts
            .canonical_body
            .as_mut()
            .expect("creature body");
        creature.content.exclusions.push(ContentExclusion {
            parent_record_key: creature.identity.record_key.clone(),
            content_key: ContentKey::new("fixture:excluded").expect("content key"),
            relative_source_path: "$.items[excluded].system.description.value".to_string(),
            label: Some("Excluded fixture".to_string()),
            reason: ContentExclusionReason::MissingTypedOwner,
        });
        index_build_input(SourceLoad {
            manifest_path: PathBuf::from("manifest.json"),
            source_signature: "foundry-pf2e:fixture-coherence".to_string(),
            source_record_count: 1,
            packs: vec![LoadedPack {
                name: pack_name,
                label: "Bestiary".to_string(),
                document_type: "Actor".to_string(),
                declared_path: "packs/bestiary".to_string(),
                resolved_path: PathBuf::from("packs/bestiary"),
                record_count: 1,
            }],
            records,
            references: Vec::new(),
            aliases: Vec::new(),
            remaster_links: Vec::new(),
            pending_document_embeddings: Vec::new(),
            document_embeddings: Vec::new(),
            document_embedding_tokenization: DocumentEmbeddingTokenizationTelemetry::default(),
            diagnostics: IngestDiagnostics::default(),
            skipped_records: Vec::new(),
            warnings: Vec::new(),
        })
    }

    fn record(pack_name: &str, id: &str, kind: RecordKind) -> AtlasRecord {
        let pack_name = PackName::new(pack_name).expect("pack name parses");
        let id = RecordId::new(id).expect("record id parses");
        AtlasRecord::new(
            RecordIdentity::new(RecordKey::new(pack_name, id), "Test Record"),
            RecordClassification::new(kind),
            FoundryRecordInfo::new(
                "Test Pack",
                FoundryDocumentType::Item,
                FoundryRecordType::Action,
            ),
            RecordProvenance::new("test.json").with_raw_json("{}"),
        )
    }
}
