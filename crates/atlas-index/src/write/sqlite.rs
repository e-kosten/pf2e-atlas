use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::{ArtifactPublicationReceipt, IndexArtifactWriter, IndexBuildInput};
use atlas_embedding::EmbeddingModelId;
use diesel::connection::SimpleConnection;
use diesel::{Connection, SqliteConnection};
use tracing::info;

mod discovery_catalogs;
mod embeddings;
mod labels;
mod metadata;
mod metric_catalogs;
mod models;
mod output;
mod packs;
mod records;
mod relationships;
mod schema;
mod vector_index;

use canonical::write_canonical_records;
use discovery_catalogs::write_discovery_catalogs;
use embeddings::write_document_embedding_cache;
use metadata::write_artifact_metadata;
use metric_catalogs::write_metric_catalogs;
use output::{ArtifactOutput, sqlite_database_url};
use packs::write_packs;
use records::write_records;
use relationships::{
    write_record_aliases, write_reference_edges, write_reference_occurrences, write_remaster_links,
};
use vector_index::write_record_vector_index;

use crate::IndexWriteError;

const INSERT_BATCH_ROWS: usize = 16;

pub struct SqliteIndexWriter {
    path: PathBuf,
    publication_target: PathBuf,
}

impl SqliteIndexWriter {
    pub fn new(path: PathBuf) -> Self {
        Self {
            publication_target: path.clone(),
            path,
        }
    }

    pub fn new_for_publication(path: PathBuf, publication_target: PathBuf) -> Self {
        Self {
            path,
            publication_target,
        }
    }
}

impl IndexArtifactWriter for SqliteIndexWriter {
    fn label(&self) -> &'static str {
        "SQLite"
    }

    fn output_path(&self) -> &Path {
        &self.path
    }

    fn write(
        &self,
        input: &IndexBuildInput,
        embedding_model: EmbeddingModelId,
    ) -> Result<ArtifactPublicationReceipt, IndexWriteError> {
        write_artifact(&self.path, &self.publication_target, input, embedding_model)
    }
}

fn write_artifact(
    path: &Path,
    publication_target: &Path,
    input: &IndexBuildInput,
    embedding_model: EmbeddingModelId,
) -> Result<ArtifactPublicationReceipt, IndexWriteError> {
    let write_started = Instant::now();
    artifact_progress("artifact_write", "Preparing artifact output");
    info!(output = %path.display(), "preparing artifact output");
    let output = ArtifactOutput::prepare(path)?;

    if !input.document_embeddings.is_empty() {
        artifact_progress("artifact_write", "Loading sqlite vector extension");
        atlas_sqlite_vec::register_sqlite_vec_auto_extension()
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    let database_url = sqlite_database_url(output.temp_path())?;
    let mut connection = SqliteConnection::establish(&database_url)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    enable_writer_foreign_keys(&mut connection)?;
    connection.transaction::<_, IndexWriteError, _>(|connection| {
        let canonical_record_keys = input
            .canonical_bodies
            .iter()
            .map(|body| body.record_key().to_string())
            .collect::<std::collections::BTreeSet<_>>();
        artifact_progress("artifact_write", "Creating artifact schema");
        info!("creating artifact schema");
        schema::create_artifact_schema(connection)?;
        artifact_progress("artifact_write", "Writing artifact metadata");
        info!("writing artifact metadata");
        write_artifact_metadata(
            connection,
            input.source_record_count,
            input.artifact_record_count(),
            input.generated_record_count()?,
            &input.source_signature,
            embedding_model,
        )?;
        artifact_progress("artifact_write", "Writing packs");
        info!(packs = input.packs.len(), "writing packs");
        write_packs(connection, &input.packs)?;
        artifact_progress("artifact_write", "Writing records");
        info!(records = input.records.len(), "writing records");
        write_records(
            connection,
            &input.records,
            &input.aliases,
            &input.remaster_links,
            &input.canonical_bodies,
            &canonical_record_keys,
            &input.consumable_occurrence_sets,
        )?;
        artifact_progress("artifact_write", "Writing canonical record artifact");
        write_canonical_records(
            connection,
            &input.records,
            &input.canonical_bodies,
            &input.canonical_spell_children,
            &input.consumable_occurrence_sets,
        )?;
        artifact_progress("artifact_write", "Writing reference edges");
        info!(
            reference_edges = input.references.len(),
            "writing reference edges"
        );
        write_reference_edges(connection, &input.references)?;
        artifact_progress("artifact_write", "Writing reference occurrences");
        info!(
            records = input.records.len(),
            "writing reference occurrences"
        );
        write_reference_occurrences(
            connection,
            &input.records,
            &canonical_record_keys,
            &input.consumable_occurrence_sets,
        )?;
        artifact_progress("artifact_write", "Writing record aliases");
        info!(aliases = input.aliases.len(), "writing record aliases");
        write_record_aliases(connection, &input.aliases)?;
        artifact_progress("artifact_write", "Writing remaster links");
        info!(
            remaster_links = input.remaster_links.len(),
            "writing remaster links"
        );
        write_remaster_links(connection, &input.remaster_links)?;
        artifact_progress("artifact_write", "Writing document embedding cache");
        info!(
            document_embeddings = input.document_embeddings.len(),
            "writing document embedding cache"
        );
        write_document_embedding_cache(connection, &input.document_embeddings)?;
        if !input.document_embeddings.is_empty() {
            artifact_progress("artifact_write", "Writing record vector index");
            info!(
                document_embeddings = input.document_embeddings.len(),
                "writing record vector index"
            );
            write_record_vector_index(connection)?;
        }
        artifact_progress("artifact_write", "Writing metric catalogs");
        info!("writing metric catalogs");
        write_metric_catalogs(connection)?;
        artifact_progress("artifact_write", "Writing filter discovery catalogs");
        info!("writing filter discovery catalogs");
        write_discovery_catalogs(connection)?;
        artifact_progress("artifact_write", "Finalizing SQLite artifact tables");
        info!("committing SQLite artifact tables");
        Ok(())
    })?;
    drop(connection);

    artifact_progress("artifact_write", "Sealing candidate artifact");
    output.commit()?;
    let write_ms = write_started.elapsed().as_millis();
    artifact_progress("artifact_write", "Validating complete candidate artifact");
    match ArtifactPublicationReceipt::issue(path, publication_target, write_ms) {
        Ok(receipt) => Ok(receipt),
        Err(error) => {
            let _ = output::remove_sqlite_files(path);
            Err(error)
        }
    }
}

fn enable_writer_foreign_keys(connection: &mut SqliteConnection) -> Result<(), IndexWriteError> {
    connection
        .batch_execute("PRAGMA foreign_keys = ON")
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))
}

fn artifact_progress(phase: &'static str, message: &'static str) {
    info!(target: "atlas_progress", phase, "{message}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::Connection as _;
    use std::collections::BTreeMap;
    use std::ffi::OsString;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use atlas_domain::{
        MetricDomain, PackName, PublicationCategory, Rarity, RecordId, RecordKey, RecordKind,
        TimeKind, TimeUnit,
    };
    use atlas_embedding::EmbeddingModelId;
    use atlas_record::{
        ActivationTimeSourceField, ActorMechanics, AliasSource, AtlasRecord, ContentSourceKind,
        ContentVisibility, DurationTimeSourceField, FoundryDocumentMechanics, FoundryDocumentType,
        FoundryRecordInfo, FoundryRecordType, ItemMechanics, MetricRow, MetricValue,
        NormalizedTime, RecordActivationTiming, RecordAlias, RecordClassification, RecordContent,
        RecordContentDocument, RecordDurationTiming, RecordIdentity, RecordMechanics,
        RecordProvenance, RecordPublication, RecordRequirements, RecordTaxonomy, RecordTiming,
        RecordVariantMembership, RecordVisibility, RecordVisibilityReason, ReferenceEdge,
        RemasterLink, RichDocument, RichNode, VariantSource,
    };
    use rusqlite::Connection;

    use crate::{IndexBuildPack, ValidationStatus};
    use output::{move_existing_sqlite_files, remove_sqlite_files, sqlite_paths};

    #[test]
    fn writer_connection_enforces_foreign_keys_before_transactions() {
        let mut connection = SqliteConnection::establish(":memory:").unwrap();
        enable_writer_foreign_keys(&mut connection).unwrap();
        connection
            .batch_execute(
                "CREATE TABLE parent(id INTEGER PRIMARY KEY);\
                 CREATE TABLE child(parent_id INTEGER NOT NULL REFERENCES parent(id));",
            )
            .unwrap();

        let error = connection
            .transaction::<_, IndexWriteError, _>(|connection| {
                connection
                    .batch_execute("INSERT INTO child(parent_id) VALUES (99)")
                    .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))
            })
            .expect_err("foreign-key violations must abort the writer transaction");

        assert!(error.to_string().contains("FOREIGN KEY constraint failed"));
    }

    #[test]
    fn writer_rejects_creature_generic_document_mechanics() {
        let target_path = unique_temp_path("creature-generic-mechanics.sqlite");
        let pack_name = PackName::new("bestiary").expect("pack parses");
        let mut record = fixture_record(&pack_name, "testCreature", "Test Creature");
        record.classification.kind = RecordKind::Creature;
        record.foundry.document_type = FoundryDocumentType::Actor;
        record.foundry.record_type = FoundryRecordType::Npc;
        record.mechanics.document = FoundryDocumentMechanics::Actor(ActorMechanics::default());

        let error = write_fixture_records(&target_path, vec![record], Vec::new())
            .expect_err("creature generic mechanics must fail closed");

        assert!(
            error
                .to_string()
                .contains("retains forbidden generic document mechanics")
        );
    }

    #[test]
    fn writer_rejects_creature_generic_metrics() {
        let target_path = unique_temp_path("creature-generic-metrics.sqlite");
        let pack_name = PackName::new("bestiary").expect("pack parses");
        let mut record = fixture_record(&pack_name, "testCreature", "Test Creature");
        record.classification.kind = RecordKind::Creature;
        record.foundry.document_type = FoundryDocumentType::Actor;
        record.foundry.record_type = FoundryRecordType::Npc;
        record.mechanics.document = FoundryDocumentMechanics::None;

        let error = write_fixture_records(&target_path, vec![record], Vec::new())
            .expect_err("creature generic metrics must fail closed");

        assert!(
            error
                .to_string()
                .contains("retains forbidden generic metrics")
        );
    }

    #[test]
    fn writer_derives_creature_metrics_from_the_matching_canonical_body()
    -> Result<(), Box<dyn std::error::Error>> {
        let target_path = unique_temp_path("canonical-creature-metrics.sqlite");
        let pack_name = PackName::new("bestiary")?;
        let mut record = fixture_record(&pack_name, "testCreature", "Test Creature");
        record.classification.kind = RecordKind::Creature;
        record.classification.level = None;
        record.classification.rarity = None;
        record.classification.traits.clear();
        record.foundry.document_type = FoundryDocumentType::Actor;
        record.foundry.record_type = FoundryRecordType::Npc;
        record.mechanics = RecordMechanics::default();
        let body = fixture_creature_body(&record, 22, 80, 15);

        write_fixture_records_with_canonical_bodies(
            &target_path,
            vec![record],
            vec![body],
            Vec::new(),
        )?;

        let connection = Connection::open(&target_path)?;
        let mut statement = connection.prepare(
            "SELECT metric_key,number_value FROM record_metrics
             WHERE record_key='bestiary:testCreature' ORDER BY ordinal",
        )?;
        let actual = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(
            actual,
            vec![
                ("perception.mod".to_string(), 15.0),
                ("ac.value".to_string(), 22.0),
                ("hp.value".to_string(), 80.0),
                ("hp.max".to_string(), 80.0),
            ]
        );
        let summary: (i64, String) = connection.query_row(
            "SELECT metric_count,metric_order_sha256 FROM records
             WHERE record_key='bestiary:testCreature'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        let atlas_record::RecordBody::Creature(creature) = fixture_creature_body(
            &fixture_record(&pack_name, "testCreature", "Test Creature"),
            22,
            80,
            15,
        ) else {
            panic!("creature body")
        };
        let projected = atlas_record::project_creature_facts(&creature).metrics;
        assert_eq!(summary.0, i64::try_from(projected.len())?);
        assert_eq!(
            summary.1,
            crate::read::records::children::metric_order_digest(&projected)?
        );
        drop(statement);
        drop(connection);
        let _ = fs::remove_file(&target_path);
        Ok(())
    }

    #[test]
    fn writer_preserves_non_creature_generic_metrics() -> Result<(), Box<dyn std::error::Error>> {
        let target_path = unique_temp_path("non-creature-generic-metrics.sqlite");
        let pack_name = PackName::new("actions")?;
        let record = fixture_record(&pack_name, "testAction", "Test Action");

        write_fixture_records(&target_path, vec![record], Vec::new())?;

        let connection = Connection::open(&target_path)?;
        let actual: Vec<(String, f64)> = {
            let mut statement = connection.prepare(
                "SELECT metric_key,number_value FROM record_metrics
                 WHERE record_key='actions:testAction' ORDER BY ordinal",
            )?;
            statement
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
                .collect::<Result<Vec<_>, _>>()?
        };
        assert_eq!(
            actual,
            vec![
                ("level.value".to_string(), 1.0),
                ("rank.value".to_string(), 2.0),
            ]
        );
        drop(connection);
        let _ = fs::remove_file(&target_path);
        Ok(())
    }

    #[test]
    fn writer_rejects_missing_or_unexpected_canonical_creature_bodies() {
        let pack_name = PackName::new("bestiary").expect("pack parses");
        let mut creature = fixture_record(&pack_name, "testCreature", "Test Creature");
        creature.classification.kind = RecordKind::Creature;
        creature.foundry.document_type = FoundryDocumentType::Actor;
        creature.foundry.record_type = FoundryRecordType::Npc;
        creature.mechanics = RecordMechanics::default();
        let missing_path = unique_temp_path("missing-canonical-creature.sqlite");
        let missing = write_fixture_records(&missing_path, vec![creature.clone()], Vec::new())
            .expect_err("a creature must have a matching canonical body");
        assert!(
            missing
                .to_string()
                .contains("is missing its required canonical body")
        );

        let mut non_creature = creature.clone();
        non_creature.classification.kind = RecordKind::Rule;
        non_creature.foundry.document_type = FoundryDocumentType::Item;
        non_creature.foundry.record_type = FoundryRecordType::Action;
        let body = fixture_creature_body(&creature, 22, 80, 15);
        let unexpected_path = unique_temp_path("unexpected-canonical-creature.sqlite");
        let unexpected = write_fixture_records_with_canonical_bodies(
            &unexpected_path,
            vec![non_creature],
            vec![body],
            Vec::new(),
        )
        .expect_err("a non-creature must not have a canonical creature body");
        assert!(
            unexpected
                .to_string()
                .contains("has an unexpected canonical body")
        );
    }

    #[test]
    fn writes_valid_artifact_through_diesel_writer() -> Result<(), Box<dyn std::error::Error>> {
        let target_path = unique_temp_path("successful-artifact-write.sqlite");
        let pack_name = PackName::new("actions")?;
        let records = (0..(INSERT_BATCH_ROWS * 50 + 1))
            .map(|index| {
                fixture_record(
                    &pack_name,
                    &format!("testAction{index:02}"),
                    &format!("Test Action {index:02}"),
                )
            })
            .collect::<Vec<_>>();
        let references = records
            .windows(2)
            .map(|pair| ReferenceEdge {
                from_record_key: pair[1].identity.key.clone(),
                to_record_key: pair[0].identity.key.clone(),
                display_text: Some(pair[0].identity.name.clone()),
                reference_text: format!("Compendium.pf2e.actions.{}", pair[0].identity.id()),
                relation_kind: atlas_record::ReferenceRelationKind::Reference,
                source_kind: ContentSourceKind::Description,
                visibility: ContentVisibility::Public,
            })
            .collect::<Vec<_>>();
        let aliases = records
            .iter()
            .map(|record| RecordAlias {
                canonical_record_key: record.identity.key.clone(),
                alias_text: format!("{} Alias", record.identity.name),
                normalized_alias: format!("{} alias", record.identity.normalized_name()),
                source: AliasSource::CompendiumSource,
                source_ref: "fixture".to_string(),
            })
            .collect::<Vec<_>>();
        let remaster_links = vec![RemasterLink {
            remaster_record_key: records[1].identity.key.clone(),
            legacy_record_key: records[0].identity.key.clone(),
            source: atlas_domain::RemasterLinkSource::Migration,
            source_ref: "fixture".to_string(),
        }];
        let records_len = records.len();
        let references_len = references.len();
        let aliases_len = aliases.len();
        let remaster_links_expected = remaster_links.clone();
        let resolved_path = Path::new("packs/actions").to_path_buf();
        let input = IndexBuildInput {
            source_signature: "foundry-pf2e:fixture".to_string(),
            source_record_count: records_len,
            packs: vec![IndexBuildPack {
                name: pack_name,
                label: "Actions".to_string(),
                document_type: "Item".to_string(),
                declared_path: "packs/actions".to_string(),
                resolved_path,
                record_count: records_len,
            }],
            records,
            canonical_bodies: Vec::new(),
            canonical_spell_children: Vec::new(),
            consumable_occurrence_sets: Vec::new(),
            references,
            aliases,
            remaster_links,
            pending_document_embeddings: Vec::new(),
            document_embeddings: Vec::new(),
        };

        SqliteIndexWriter::new(target_path.clone())
            .write(&input, EmbeddingModelId::BgeSmallEnV15)
            .expect("writer should produce a valid artifact");
        let reader = crate::SqliteIndexReader::open_unpublished_read_only(&target_path)?;
        let validation = reader.validate()?;
        assert_eq!(validation.status, ValidationStatus::Ok, "{validation:?}");
        let record_set = reader.load_record_set()?;
        assert_eq!(record_set.records.len(), records_len);
        assert_eq!(record_set.reference_edges.len(), references_len);
        assert_eq!(record_set.aliases.len(), aliases_len);
        assert_eq!(record_set.remaster_links, remaster_links_expected);
        let loaded = record_set
            .records
            .iter()
            .find(|record| record.identity.key.to_string() == "actions:testAction00")
            .expect("fixture record should load");
        assert_eq!(loaded.classification.rarity, Some(Rarity::Rare));
        assert_eq!(loaded.publication.title.as_deref(), Some("Fixture Book"));
        assert_eq!(loaded.publication.category, PublicationCategory::Core);
        assert!(loaded.publication.remaster);
        assert_eq!(loaded.foundry.folder_id.as_deref(), Some("folder-1"));
        assert_eq!(loaded.requirements.prerequisites, vec!["trained in Arcana"]);
        assert_eq!(
            loaded
                .timing
                .activation
                .as_ref()
                .map(|timing| timing.source_field),
            Some(ActivationTimeSourceField::TimeValue)
        );
        assert_eq!(
            loaded
                .timing
                .duration
                .as_ref()
                .map(|timing| timing.source_field),
            Some(DurationTimeSourceField::DurationValue)
        );
        let variant = loaded.variant.as_ref().expect("variant should load");
        assert_eq!(variant.source, VariantSource::Parenthetical);
        assert_eq!(variant.axes, vec!["grade"]);
        assert_eq!(
            loaded.provenance.raw_json.as_deref(),
            Some(r#"{"fixture":true}"#)
        );
        assert!(loaded.visibility.visible_by_default());
        assert_eq!(
            loaded.visibility.reason(),
            RecordVisibilityReason::SourceRecord
        );
        assert_eq!(loaded.mechanics.metrics[0].key, "level.value");
        assert_eq!(loaded.mechanics.metrics[1].key, "rank.value");
        assert!(loaded.content.description().is_some());
        assert!(loaded.content.blurb().is_some());
        assert!(
            loaded
                .content
                .documents
                .iter()
                .any(|document| document.source_kind == ContentSourceKind::PublicNotes)
        );
        let item = loaded
            .mechanics
            .item()
            .expect("item mechanics should round trip");
        assert_eq!(item.price_json.as_deref(), Some(r#"{"gp":1}"#));
        assert_eq!(item.price_cp, Some(100));
        let connection = Connection::open(&target_path)?;
        let independent_visibility: (String, String, String, String, String, i64) = connection
            .query_row(
                "SELECT visibility_state,visibility_reason,record_role,retrieval_disposition,
                        retrieval_rationale,is_default_visible
                 FROM records WHERE record_key='actions:testAction00'",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
                },
            )?;
        assert_eq!(
            independent_visibility,
            (
                "visible".to_string(),
                "source_record".to_string(),
                "source".to_string(),
                "direct_only".to_string(),
                "canonical_edition_duplicate".to_string(),
                0,
            )
        );
        let duplicate_metric_ordinal = connection.execute(
            "INSERT INTO record_metrics (
                 record_key,ordinal,metric_domain,metric_key,value_type,number_value
             ) VALUES ('actions:testAction00',0,'item','duplicate.metric','number',1)",
            [],
        );
        assert!(
            duplicate_metric_ordinal
                .expect_err("duplicate metric ordinal must be rejected")
                .to_string()
                .contains("record_metrics.record_key, record_metrics.ordinal")
        );
        let metric_count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM metric_key_catalog WHERE metric_key = 'level.value'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(metric_count, 2);
        let discovery_count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM filter_value_catalog WHERE field = 'record_kind' AND value = 'rule'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(discovery_count, 2);
        let alias_count: i64 =
            connection.query_row("SELECT COUNT(*) FROM record_aliases", [], |row| row.get(0))?;
        assert_eq!(alias_count, aliases_len as i64);
        let reference_count: i64 =
            connection.query_row("SELECT COUNT(*) FROM reference_edges", [], |row| row.get(0))?;
        assert_eq!(reference_count, references_len as i64);

        let _ = fs::remove_file(&target_path);
        Ok(())
    }

    #[test]
    fn visibility_and_retrieval_policy_round_trip_independently_with_exact_170_regression()
    -> Result<(), Box<dyn std::error::Error>> {
        let target_path = unique_temp_path("visibility-policy-170.sqlite");
        let pack_name = PackName::new("actions")?;
        let mut records = Vec::new();
        for index in 0..150 {
            let mut record = fixture_record(
                &pack_name,
                &format!("tooling{index:03}"),
                &format!("Tooling {index:03}"),
            );
            record.classification.kind = RecordKind::Tooling;
            records.push(record);
        }
        let mut remaster_links = Vec::new();
        for index in 0..20 {
            let legacy = fixture_record(
                &pack_name,
                &format!("legacy{index:03}"),
                &format!("Legacy {index:03}"),
            );
            let remaster = fixture_record(
                &pack_name,
                &format!("remaster{index:03}"),
                &format!("Remaster {index:03}"),
            );
            remaster_links.push(RemasterLink {
                remaster_record_key: remaster.identity.key.clone(),
                legacy_record_key: legacy.identity.key.clone(),
                source: atlas_domain::RemasterLinkSource::Migration,
                source_ref: "focused exact-170 fixture".to_string(),
            });
            records.extend([legacy, remaster]);
        }
        write_fixture_records(&target_path, records, remaster_links)?;

        let connection = Connection::open(&target_path)?;
        let visible_source_but_not_default: i64 = connection.query_row(
            "SELECT COUNT(*) FROM records WHERE visibility_state='visible' AND visibility_reason='source_record' AND is_default_visible=0",
            [],
            |row| row.get(0),
        )?;
        let inspection_only: i64 = connection.query_row(
            "SELECT COUNT(*) FROM records WHERE retrieval_disposition='inspection_only' AND retrieval_rationale='tooling_no_addressable_product_meaning'",
            [],
            |row| row.get(0),
        )?;
        let direct_only: i64 = connection.query_row(
            "SELECT COUNT(*) FROM records WHERE retrieval_disposition='direct_only' AND retrieval_rationale='canonical_edition_duplicate'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(visible_source_but_not_default, 170);
        assert_eq!(inspection_only, 150);
        assert_eq!(direct_only, 20);
        drop(connection);

        let reader = crate::SqliteIndexReader::open_unpublished_read_only(&target_path)?;
        let full = reader.load_record_set()?.records;
        assert_eq!(full.len(), 190);
        assert_eq!(
            full.iter()
                .filter(|record| record.visibility.visible_by_default()
                    && record.visibility.reason() == RecordVisibilityReason::SourceRecord)
                .count(),
            190
        );
        let non_default_keys = full
            .iter()
            .filter(|record| {
                record.classification.kind == RecordKind::Tooling
                    || record.identity.id().as_str().starts_with("legacy")
            })
            .map(|record| record.identity.key.clone())
            .collect::<Vec<_>>();
        let by_key = reader.load_records_by_key(&non_default_keys)?;
        assert_eq!(by_key.len(), 170);
        assert!(by_key.iter().all(|record| {
            record.visibility.visible_by_default()
                && record.visibility.reason() == RecordVisibilityReason::SourceRecord
        }));
        let validation = reader.validate()?;
        assert_eq!(validation.status, ValidationStatus::Ok, "{validation:?}");

        let _ = fs::remove_file(&target_path);
        Ok(())
    }

    #[test]
    fn visibility_and_retrieval_policy_mutations_have_typed_independent_diagnostics()
    -> Result<(), Box<dyn std::error::Error>> {
        let base_path = unique_temp_path("visibility-policy-mutation-base.sqlite");
        let pack_name = PackName::new("actions")?;
        write_fixture_records(
            &base_path,
            vec![fixture_record(&pack_name, "testAction00", "Test Action 00")],
            Vec::new(),
        )?;

        for (name, sql, expected_key) in [
            (
                "unknown-visibility-state",
                "PRAGMA ignore_check_constraints=ON; UPDATE records SET visibility_state='unknown'",
                "records.visibility_state",
            ),
            (
                "unknown-visibility-reason",
                "PRAGMA ignore_check_constraints=ON; UPDATE records SET visibility_reason='unknown'",
                "records.visibility_reason",
            ),
            (
                "unknown-record-role",
                "PRAGMA ignore_check_constraints=ON; UPDATE records SET record_role='unknown'",
                "records.record_role",
            ),
            (
                "unknown-retrieval-disposition",
                "PRAGMA ignore_check_constraints=ON; UPDATE records SET retrieval_disposition='unknown'",
                "records.retrieval_disposition",
            ),
            (
                "unknown-retrieval-rationale",
                "PRAGMA ignore_check_constraints=ON; UPDATE records SET retrieval_rationale='unknown'",
                "records.retrieval_rationale",
            ),
            (
                "impossible-policy-tuple",
                "UPDATE records SET record_role='source',retrieval_disposition='direct_only',retrieval_rationale='source_record',is_default_visible=0; DELETE FROM records_fts",
                "records.retrieval_policy_tuple",
            ),
            (
                "derived-boolean-divergence",
                "UPDATE records SET is_default_visible=0",
                "records.retrieval_policy",
            ),
            (
                "visibility-policy-cross-contract",
                "UPDATE records SET visibility_reason='generated_canonical'",
                "records.visibility_role_coherence",
            ),
        ] {
            assert_data_mutation_rejected(&base_path, name, sql, expected_key)?;
        }

        let visibility_path = unique_temp_path("visibility-only-mutation.sqlite");
        fs::copy(&base_path, &visibility_path)?;
        let connection = Connection::open(&visibility_path)?;
        connection.execute("UPDATE records SET visibility_state='hidden'", [])?;
        drop(connection);
        let reader = crate::SqliteIndexReader::open_unpublished_read_only(&visibility_path)?;
        let record = reader
            .load_records_by_key(&[RecordKey::parse("actions:testAction00")?])?
            .pop()
            .expect("visibility-only fixture hydrates");
        assert!(!record.visibility.visible_by_default());
        let connection = Connection::open(&visibility_path)?;
        let policy: (String, String, String, i64) = connection.query_row(
            "SELECT record_role,retrieval_disposition,retrieval_rationale,is_default_visible FROM records",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;
        assert_eq!(
            policy,
            (
                "source".to_string(),
                "ordinary".to_string(),
                "source_record".to_string(),
                1,
            )
        );
        drop(connection);
        assert_eq!(reader.validate()?.status, ValidationStatus::Ok);

        let policy_path = unique_temp_path("policy-only-mutation.sqlite");
        fs::copy(&base_path, &policy_path)?;
        let connection = Connection::open(&policy_path)?;
        connection.execute_batch(
            "UPDATE records SET retrieval_disposition='inspection_only',retrieval_rationale='tooling_no_addressable_product_meaning',is_default_visible=0; DELETE FROM records_fts",
        )?;
        drop(connection);
        let reader = crate::SqliteIndexReader::open_unpublished_read_only(&policy_path)?;
        let record = reader
            .load_records_by_key(&[RecordKey::parse("actions:testAction00")?])?
            .pop()
            .expect("policy-only fixture hydrates");
        assert!(record.visibility.visible_by_default());
        assert_eq!(
            record.visibility.reason(),
            RecordVisibilityReason::SourceRecord
        );
        let connection = Connection::open(&policy_path)?;
        let policy: (String, String, String, i64) = connection.query_row(
            "SELECT record_role,retrieval_disposition,retrieval_rationale,is_default_visible FROM records",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;
        assert_eq!(
            policy,
            (
                "source".to_string(),
                "inspection_only".to_string(),
                "tooling_no_addressable_product_meaning".to_string(),
                0,
            )
        );
        drop(connection);
        let report = reader.validate()?;
        assert_eq!(report.status, ValidationStatus::Error, "{report:?}");
        assert_eq!(
            report.code,
            crate::ValidationCode::ArtifactContractViolation,
            "{report:?}"
        );
        for expected_key in [
            "metric_key_catalog.stale_keys",
            "filter_field_catalog.stale_rows",
        ] {
            assert!(
                report
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.key.as_deref() == Some(expected_key)),
                "expected catalog diagnostic `{expected_key}`, got {report:?}"
            );
        }
        let visibility_policy_keys = [
            "records.visibility_state",
            "records.visibility_reason",
            "records.record_role",
            "records.retrieval_disposition",
            "records.retrieval_rationale",
            "records.retrieval_policy",
            "records.retrieval_policy_tuple",
            "records.visibility_role_coherence",
        ];
        assert!(
            report.diagnostics.iter().all(|diagnostic| {
                diagnostic
                    .key
                    .as_deref()
                    .is_none_or(|key| !visibility_policy_keys.contains(&key))
            }),
            "policy-only mutation must not produce a visibility/policy diagnostic: {report:?}"
        );

        let _ = fs::remove_file(base_path);
        let _ = fs::remove_file(visibility_path);
        let _ = fs::remove_file(policy_path);
        Ok(())
    }

    #[test]
    fn failed_artifact_write_preserves_existing_target_and_cleans_temp()
    -> Result<(), Box<dyn std::error::Error>> {
        let target_path = unique_temp_path("failed-artifact-write.sqlite");
        fs::write(&target_path, b"existing artifact")?;
        let input = IndexBuildInput {
            source_signature: "fixture".to_string(),
            source_record_count: 1,
            packs: Vec::new(),
            records: Vec::new(),
            canonical_bodies: Vec::new(),
            canonical_spell_children: Vec::new(),
            consumable_occurrence_sets: Vec::new(),
            references: Vec::new(),
            aliases: Vec::new(),
            remaster_links: Vec::new(),
            pending_document_embeddings: Vec::new(),
            document_embeddings: Vec::new(),
        };

        let error = write_artifact(
            &target_path,
            &target_path,
            &input,
            EmbeddingModelId::BgeSmallEnV15,
        )
        .expect_err("invalid input should fail before publish");

        assert!(matches!(error, IndexWriteError::InvalidInput(_)));
        assert_eq!(fs::read(&target_path)?, b"existing artifact");
        let parent = target_path.parent().expect("temp file should have parent");
        let file_name = target_path
            .file_name()
            .expect("temp file should have file name")
            .to_string_lossy();
        let leftovers = fs::read_dir(parent)?
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(&format!("{file_name}.rebuild-"))
            })
            .collect::<Vec<_>>();
        assert!(leftovers.is_empty());

        let _ = fs::remove_file(&target_path);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn writer_rejects_non_utf8_artifact_paths() -> Result<(), Box<dyn std::error::Error>> {
        let target_path = non_utf8_temp_path("writer");
        let input = IndexBuildInput {
            source_signature: "foundry-pf2e:fixture".to_string(),
            source_record_count: 0,
            packs: Vec::new(),
            records: Vec::new(),
            canonical_bodies: Vec::new(),
            canonical_spell_children: Vec::new(),
            consumable_occurrence_sets: Vec::new(),
            references: Vec::new(),
            aliases: Vec::new(),
            remaster_links: Vec::new(),
            pending_document_embeddings: Vec::new(),
            document_embeddings: Vec::new(),
        };

        let error = write_artifact(
            &target_path,
            &target_path,
            &input,
            EmbeddingModelId::BgeSmallEnV15,
        )
        .expect_err("non-UTF-8 database path should be rejected");

        assert!(matches!(error, IndexWriteError::WriteFailed(_)));
        assert!(error.to_string().contains("not valid UTF-8"));
        let _ = remove_sqlite_files(&target_path);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn writer_rejects_non_utf8_pack_paths() -> Result<(), Box<dyn std::error::Error>> {
        let target_path = unique_temp_path("non-utf8-pack-path.sqlite");
        let pack_name = PackName::new("actions")?;
        let resolved_path = non_utf8_temp_path("pack");
        let input = IndexBuildInput {
            source_signature: "foundry-pf2e:fixture".to_string(),
            source_record_count: 0,
            packs: vec![IndexBuildPack {
                name: pack_name,
                label: "Actions".to_string(),
                document_type: "Item".to_string(),
                declared_path: "packs/actions".to_string(),
                resolved_path,
                record_count: 0,
            }],
            records: Vec::new(),
            canonical_bodies: Vec::new(),
            canonical_spell_children: Vec::new(),
            consumable_occurrence_sets: Vec::new(),
            references: Vec::new(),
            aliases: Vec::new(),
            remaster_links: Vec::new(),
            pending_document_embeddings: Vec::new(),
            document_embeddings: Vec::new(),
        };

        let error = write_artifact(
            &target_path,
            &target_path,
            &input,
            EmbeddingModelId::BgeSmallEnV15,
        )
        .expect_err("non-UTF-8 pack path should be rejected");

        assert!(matches!(error, IndexWriteError::WriteFailed(_)));
        assert!(error.to_string().contains("pack resolved path"));
        assert!(error.to_string().contains("not valid UTF-8"));
        let _ = remove_sqlite_files(&target_path);
        Ok(())
    }

    #[test]
    fn failed_backup_move_restores_partially_moved_target() -> Result<(), Box<dyn std::error::Error>>
    {
        let target_path = unique_temp_path("backup-source.sqlite");
        let backup_path = unique_temp_path("backup-target.sqlite");
        fs::write(&target_path, b"main")?;
        fs::write(sqlite_paths(&target_path)[1].as_path(), b"wal")?;
        fs::create_dir(sqlite_paths(&backup_path)[1].as_path())?;

        let error = move_existing_sqlite_files(&target_path, &backup_path)
            .expect_err("backup should fail after moving the main database");

        assert!(error.to_string().contains("Is a directory"));
        assert_eq!(fs::read(&target_path)?, b"main");
        assert_eq!(fs::read(sqlite_paths(&target_path)[1].as_path())?, b"wal");
        assert!(!backup_path.exists());

        let _ = fs::remove_file(&target_path);
        let _ = fs::remove_file(sqlite_paths(&target_path)[1].as_path());
        let _ = fs::remove_dir(sqlite_paths(&backup_path)[1].as_path());
        Ok(())
    }

    fn unique_temp_path(name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "atlas-index-{name}-{}-{timestamp}",
            std::process::id()
        ))
    }

    fn write_fixture_records(
        target_path: &Path,
        records: Vec<AtlasRecord>,
        remaster_links: Vec<RemasterLink>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        write_fixture_records_with_canonical_bodies(
            target_path,
            records,
            Vec::new(),
            remaster_links,
        )
    }

    fn write_fixture_records_with_canonical_bodies(
        target_path: &Path,
        records: Vec<AtlasRecord>,
        canonical_bodies: Vec<atlas_record::RecordBody>,
        remaster_links: Vec<RemasterLink>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut pack_counts = BTreeMap::<String, usize>::new();
        for record in &records {
            *pack_counts
                .entry(record.identity.pack().as_str().to_string())
                .or_default() += 1;
        }
        let packs = pack_counts
            .into_iter()
            .map(|(name, record_count)| {
                let pack_name = PackName::new(name.as_str()).expect("fixture pack name parses");
                let document_type = records
                    .iter()
                    .find(|record| record.identity.pack().as_str() == name)
                    .map(|record| record.foundry.document_type.as_str())
                    .unwrap_or("Item");
                IndexBuildPack {
                    name: pack_name,
                    label: name.clone(),
                    document_type: document_type.to_string(),
                    declared_path: format!("packs/{name}"),
                    resolved_path: Path::new("packs").join(&name),
                    record_count,
                }
            })
            .collect::<Vec<_>>();
        let input = IndexBuildInput {
            source_signature: "foundry-pf2e:focused-fixture".to_string(),
            source_record_count: records.len(),
            packs,
            records,
            canonical_bodies,
            canonical_spell_children: Vec::new(),
            consumable_occurrence_sets: Vec::new(),
            references: Vec::new(),
            aliases: Vec::new(),
            remaster_links,
            pending_document_embeddings: Vec::new(),
            document_embeddings: Vec::new(),
        };
        SqliteIndexWriter::new(target_path.to_path_buf())
            .write(&input, EmbeddingModelId::BgeSmallEnV15)?;
        Ok(())
    }

    fn fixture_creature_body(
        record: &AtlasRecord,
        armor_class: i64,
        hit_points: i64,
        perception: i64,
    ) -> atlas_record::RecordBody {
        use atlas_record::{CreatureFact, CreatureSourceField, FactValue};

        macro_rules! missing {
            ($field:expr) => {
                CreatureFact::source(FactValue::Missing, $field)
            };
        }

        atlas_record::RecordBody::Creature(atlas_record::CreatureRecord {
            identity: atlas_record::CreatureIdentity {
                record_key: record.identity.key.clone(),
                source_id: atlas_record::CreatureSourceId::new(record.identity.id().as_str())
                    .expect("fixture source ID"),
                name: record.identity.name.clone(),
                family: atlas_record::CreatureFamily::Npc,
            },
            level: missing!(CreatureSourceField::Level),
            rarity: missing!(CreatureSourceField::Rarity),
            traits: missing!(CreatureSourceField::Traits),
            size: missing!(CreatureSourceField::Size),
            publication: missing!(CreatureSourceField::Publication),
            adjustment: missing!(CreatureSourceField::Adjustment),
            source_alliance: missing!(CreatureSourceField::SourceAlliance),
            perception: CreatureFact::source(
                FactValue::Value(atlas_record::CreaturePerception {
                    modifier: FactValue::Value(perception),
                    details: FactValue::Missing,
                    has_vision: FactValue::Missing,
                    senses: FactValue::Value(Vec::new()),
                }),
                CreatureSourceField::Perception,
            ),
            initiative: missing!(CreatureSourceField::Initiative),
            languages: missing!(CreatureSourceField::Languages),
            skills: missing!(CreatureSourceField::Skills),
            legacy_abilities: missing!(CreatureSourceField::LegacyAbilities),
            defenses: CreatureFact::source(
                FactValue::Value(atlas_record::CreatureDefenses {
                    armor_class: FactValue::Value(atlas_record::CreatureArmorClass {
                        value: FactValue::Value(armor_class),
                        details: FactValue::Missing,
                    }),
                    hit_points: FactValue::Value(atlas_record::CreatureHitPoints {
                        value: FactValue::Value(atlas_record::CreatureNumber::Integer(hit_points)),
                        maximum: FactValue::Value(hit_points),
                        temporary: FactValue::Missing,
                        temporary_maximum: FactValue::Missing,
                        details: FactValue::Missing,
                    }),
                    hardness: FactValue::Missing,
                    shield: FactValue::Missing,
                    saves: FactValue::Missing,
                    all_saves_note: FactValue::Missing,
                    immunities: FactValue::Missing,
                    resistances: FactValue::Missing,
                    weaknesses: FactValue::Missing,
                }),
                CreatureSourceField::Defenses,
            ),
            movement: missing!(CreatureSourceField::Movement),
            resources: missing!(CreatureSourceField::Resources),
            embedded_entities: missing!(CreatureSourceField::EmbeddedEntities),
            content: atlas_record::OwnedRichContent::default(),
            provenance: atlas_record::CreatureProvenance {
                source_path: record.provenance.source_path.clone(),
                source_contract_version: "fixture".to_string(),
                source_system_version: "fixture".to_string(),
                source_upstream_commit: "fixture".to_string(),
            },
        })
    }

    fn assert_data_mutation_rejected(
        base_path: &Path,
        name: &str,
        sql: &str,
        expected_key: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = unique_temp_path(&format!("data-mutation-{name}.sqlite"));
        fs::copy(base_path, &path)?;
        let connection = Connection::open(&path)?;
        connection.execute_batch(sql)?;
        drop(connection);

        let report = crate::SqliteIndexReader::open_unpublished_read_only(&path)?.validate()?;
        assert_eq!(report.status, ValidationStatus::Error, "{name}: {report:?}");
        assert!(
            report.diagnostics.iter().any(|diagnostic| {
                diagnostic.family == crate::ArtifactValidationFamily::Data
                    && diagnostic.key.as_deref() == Some(expected_key)
                    && diagnostic.expected.is_some()
                    && diagnostic.actual.is_some()
            }),
            "{name}: expected typed diagnostic `{expected_key}`, got {report:?}"
        );
        let _ = fs::remove_file(path);
        Ok(())
    }

    #[cfg(unix)]
    fn non_utf8_temp_path(name: &str) -> PathBuf {
        use std::os::unix::ffi::OsStringExt;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let mut file_name =
            format!("atlas-index-{name}-{}-{timestamp}-", std::process::id()).into_bytes();
        file_name.push(0xff);
        file_name.extend_from_slice(b".sqlite");
        std::env::temp_dir().join(OsString::from_vec(file_name))
    }

    fn fixture_record(pack_name: &PackName, id: &str, name: &str) -> AtlasRecord {
        let record_id = RecordId::new(id).expect("record id parses");
        AtlasRecord {
            identity: RecordIdentity {
                key: RecordKey::new(pack_name.clone(), record_id),
                name: name.to_string(),
            },
            classification: RecordClassification {
                kind: RecordKind::Rule,
                level: Some(1),
                rarity: Some(Rarity::Rare),
                traits: vec!["test".to_string()],
                taxonomy: RecordTaxonomy::default(),
            },
            foundry: FoundryRecordInfo {
                pack_label: "Actions".to_string(),
                document_type: FoundryDocumentType::Item,
                record_type: FoundryRecordType::Action,
                folder_id: Some("folder-1".to_string()),
            },
            provenance: RecordProvenance {
                source_path: format!("packs/actions/{id}.json"),
                raw_json: Some(r#"{"fixture":true}"#.to_string()),
            },
            publication: RecordPublication {
                title: Some("Fixture Book".to_string()),
                remaster: true,
                category: PublicationCategory::Core,
            },
            requirements: RecordRequirements {
                prerequisites: vec!["trained in Arcana".to_string()],
            },
            timing: RecordTiming {
                activation: Some(RecordActivationTiming {
                    time: NormalizedTime {
                        kind: TimeKind::Actions,
                        actions: Some(2),
                        duration_value: None,
                        duration_unit: None,
                        text: "2 actions".to_string(),
                    },
                    source_field: ActivationTimeSourceField::TimeValue,
                }),
                duration: Some(RecordDurationTiming {
                    time: NormalizedTime {
                        kind: TimeKind::Duration,
                        actions: None,
                        duration_value: Some(1),
                        duration_unit: Some(TimeUnit::Minute),
                        text: "1 minute".to_string(),
                    },
                    source_field: DurationTimeSourceField::DurationValue,
                }),
            },
            mechanics: RecordMechanics {
                metrics: vec![
                    MetricRow {
                        domain: MetricDomain::Item,
                        key: "level.value".to_string(),
                        value: MetricValue::Number(1.0),
                    },
                    MetricRow {
                        domain: MetricDomain::Item,
                        key: "rank.value".to_string(),
                        value: MetricValue::Number(2.0),
                    },
                ],
                document: FoundryDocumentMechanics::Item(ItemMechanics {
                    category: Some("spell".to_string()),
                    base_item: Some("test-base".to_string()),
                    group: Some("test-group".to_string()),
                    usage: Some("held in 1 hand".to_string()),
                    price_json: Some(r#"{"gp":1}"#.to_string()),
                    price_cp: Some(100),
                    bulk_value: Some(1.0),
                    hands_requirement: Some("1".to_string()),
                    damage_types: vec!["mental".to_string()],
                }),
            },
            content: RecordContent {
                documents: vec![
                    RecordContentDocument {
                        source_kind: ContentSourceKind::Description,
                        label: None,
                        document: text_document("fixture description"),
                    },
                    RecordContentDocument {
                        source_kind: ContentSourceKind::Blurb,
                        label: None,
                        document: text_document("fixture blurb"),
                    },
                    RecordContentDocument {
                        source_kind: ContentSourceKind::PublicNotes,
                        label: Some("Note".to_string()),
                        document: text_document("fixture note"),
                    },
                ],
            },
            variant: Some(RecordVariantMembership {
                group_key: "actions:test".to_string(),
                base_name: "Test Action".to_string(),
                label: Some("Grade 1".to_string()),
                axes: vec!["grade".to_string()],
                confidence: Some(0.95),
                source: VariantSource::Parenthetical,
            }),
            visibility: RecordVisibility::visible(RecordVisibilityReason::SourceRecord),
        }
    }

    fn text_document(text: &str) -> RichDocument {
        RichDocument::new(vec![RichNode::HtmlElement {
            tag: "p".to_string(),
            attributes: BTreeMap::new(),
            children: vec![RichNode::Text {
                text: text.to_string(),
            }],
        }])
    }
}
mod canonical;
