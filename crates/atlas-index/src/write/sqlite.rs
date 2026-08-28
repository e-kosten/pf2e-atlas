use std::path::{Path, PathBuf};

use crate::{IndexArtifactWriter, IndexBuildInput};
use atlas_embedding::EmbeddingModelId;
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
}

impl SqliteIndexWriter {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
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
    ) -> Result<(), IndexWriteError> {
        write_artifact(&self.path, input, embedding_model)
    }
}

fn write_artifact(
    path: &Path,
    input: &IndexBuildInput,
    embedding_model: EmbeddingModelId,
) -> Result<(), IndexWriteError> {
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
    connection.transaction::<_, IndexWriteError, _>(|connection| {
        let canonical_record_keys = input
            .canonical_bodies
            .iter()
            .map(|body| match body {
                atlas_record::RecordBody::Creature(creature) => {
                    creature.identity.record_key.to_string()
                }
            })
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
            &canonical_record_keys,
        )?;
        artifact_progress("artifact_write", "Writing canonical record artifact");
        write_canonical_records(connection, &input.canonical_bodies)?;
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
        write_reference_occurrences(connection, &input.records, &canonical_record_keys)?;
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

    artifact_progress("artifact_write", "Validating complete candidate artifact");
    let validation_connection = rusqlite::Connection::open(output.temp_path())
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    let report = crate::validate_index_connection(
        output.temp_path().display().to_string(),
        &validation_connection,
    )
    .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    if report.status != crate::ValidationStatus::Ok {
        let details = report
            .diagnostics
            .iter()
            .take(10)
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>()
            .join("; ");
        return Err(IndexWriteError::WriteFailed(format!(
            "candidate artifact failed deep validation before publication: {details}"
        )));
    }
    drop(validation_connection);

    artifact_progress("artifact_write", "Publishing artifact");
    info!("publishing artifact");
    output.commit()
}

fn artifact_progress(phase: &'static str, message: &'static str) {
    info!(target: "atlas_progress", phase, "{message}");
}

#[cfg(test)]
mod tests {
    use super::*;
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
        ActivationTimeSourceField, ActivityRoll, ActivityRollAbility, ActivityRollSurface,
        AliasSource, AtlasRecord, ContentSourceKind, ContentVisibility, DamageEffectKind,
        DamageExpression, DurationTimeSourceField, FoundryDocumentMechanics, FoundryDocumentType,
        FoundryRecordInfo, FoundryRecordType, ItemMechanics, ItemTypeMechanics, MechanicActivity,
        MechanicActivityKind, MechanicActivityMode, MechanicActivityUsage, MetricRow, MetricValue,
        NormalizedTime, RecordActivationTiming, RecordAlias, RecordClassification, RecordContent,
        RecordContentDocument, RecordDurationTiming, RecordIdentity, RecordMechanics,
        RecordProvenance, RecordPublication, RecordRequirements, RecordTaxonomy, RecordTiming,
        RecordVariantMembership, RecordVisibility, RecordVisibilityReason, ReferenceEdge,
        RemasterLink, RichDocument, RichNode, SpellArea, SpellDefense, SpellMechanics, SpellRange,
        SpellTarget, SpellcastingEntryMechanics, SpellcastingPreparation, VariantSource,
    };
    use rusqlite::Connection;

    use crate::{IndexBuildPack, ValidationStatus};
    use output::{move_existing_sqlite_files, remove_sqlite_files, sqlite_paths};

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
        assert_eq!(
            loaded
                .mechanics
                .spellcasting_entries
                .iter()
                .map(|entry| entry.entry_id.as_str())
                .collect::<Vec<_>>(),
            vec!["casting-1", "casting-2"]
        );
        assert_eq!(
            loaded
                .mechanics
                .activities
                .iter()
                .map(|activity| activity.activity_id.as_str())
                .collect::<Vec<_>>(),
            vec!["activity-1", "activity-1"]
        );
        assert_eq!(
            loaded
                .mechanics
                .activities
                .iter()
                .map(|activity| activity.label.as_str())
                .collect::<Vec<_>>(),
            vec!["Arcane Claw", "Empty Activity"]
        );
        assert_eq!(
            loaded.mechanics.activities[0].traits,
            vec!["attack", "magical"]
        );
        assert_eq!(loaded.mechanics.activities[0].rolls[0].base_value, 17);
        assert_eq!(loaded.mechanics.activities[0].damage[0].formula, "2d6+4");
        assert_eq!(
            loaded.mechanics.activities[0].modes[0].damage[0].formula,
            "3d6+4"
        );
        assert!(loaded.mechanics.activities[1].traits.is_empty());
        assert!(loaded.mechanics.activities[1].rolls.is_empty());
        assert!(loaded.mechanics.activities[1].damage.is_empty());
        assert!(loaded.mechanics.activities[1].modes.is_empty());
        assert_eq!(
            loaded.mechanics.spellcasting_entries[1].preparation,
            SpellcastingPreparation::Other("ritual".to_string())
        );
        assert_eq!(loaded.mechanics.spellcasting_entries[1].spell_attack, None);
        assert_eq!(loaded.mechanics.spellcasting_entries[1].spell_dc, None);
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
        let spell = loaded
            .mechanics
            .spell()
            .expect("spell mechanics should round trip");
        assert_eq!(spell.traditions, vec!["arcane"]);
        assert_eq!(spell.kinds, vec!["spell"]);
        assert_eq!(
            spell.range.as_ref().map(|range| range.text.as_str()),
            Some("30 feet")
        );
        assert_eq!(
            spell.target.as_ref().map(|target| target.text.as_str()),
            Some("1 creature")
        );
        assert_eq!(
            spell.area.as_ref().and_then(|area| area.kind.as_deref()),
            Some("burst")
        );
        assert!(spell.defense.as_ref().is_some_and(|defense| defense.basic));

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
        let duplicate_activity_ordinal = connection.execute(
            "INSERT INTO record_activities (record_key,activity_id,ordinal,payload_json)
             SELECT record_key,'duplicate-activity',ordinal,payload_json
             FROM record_activities
             WHERE record_key='actions:testAction00' AND ordinal=0",
            [],
        );
        assert!(
            duplicate_activity_ordinal
                .expect_err("duplicate activity ordinal must be rejected")
                .to_string()
                .contains("record_activities.record_key, record_activities.ordinal")
        );
        let duplicate_spellcasting_id = connection.execute(
            "INSERT INTO record_spellcasting_entries (record_key,entry_id,ordinal,payload_json)
             SELECT record_key,entry_id,99,payload_json
             FROM record_spellcasting_entries
             WHERE record_key='actions:testAction00' AND ordinal=0",
            [],
        );
        assert!(
            duplicate_spellcasting_id
                .expect_err("duplicate spellcasting child ID must be rejected")
                .to_string()
                .contains(
                    "record_spellcasting_entries.record_key, record_spellcasting_entries.entry_id"
                )
        );
        let first_activity_payload: String = connection.query_row(
            "SELECT payload_json FROM record_activities
             WHERE record_key='actions:testAction00' AND ordinal=0",
            [],
            |row| row.get(0),
        )?;
        let second_activity_payload: String = connection.query_row(
            "SELECT payload_json FROM record_activities
             WHERE record_key='actions:testAction00' AND ordinal=1",
            [],
            |row| row.get(0),
        )?;
        connection.execute(
            "UPDATE record_activities SET payload_json=?1
             WHERE record_key='actions:testAction00' AND ordinal=0",
            [&second_activity_payload],
        )?;
        connection.execute(
            "UPDATE record_activities SET payload_json=?1
             WHERE record_key='actions:testAction00' AND ordinal=1",
            [&first_activity_payload],
        )?;
        let reordered =
            crate::SqliteIndexReader::open_unpublished_read_only(&target_path)?.validate()?;
        assert_eq!(reordered.status, ValidationStatus::Error, "{reordered:?}");
        assert!(reordered.diagnostics.iter().any(|diagnostic| {
            diagnostic.key.as_deref()
                == Some("record_activities[actions:testAction00].order_sha256")
        }));
        connection.execute(
            "UPDATE record_activities SET payload_json=?1
             WHERE record_key='actions:testAction00' AND ordinal=0",
            [&first_activity_payload],
        )?;
        connection.execute(
            "UPDATE record_activities SET payload_json=?1
             WHERE record_key='actions:testAction00' AND ordinal=1",
            [&second_activity_payload],
        )?;
        let restored =
            crate::SqliteIndexReader::open_unpublished_read_only(&target_path)?.validate()?;
        assert_eq!(restored.status, ValidationStatus::Ok, "{restored:?}");
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
    fn shobhad_duplicate_activity_ids_round_trip_by_parent_ordinal()
    -> Result<(), Box<dyn std::error::Error>> {
        const HUNTER_KEY: &str = "pfs-season-3-bestiary:EB00f6ADElWInuix";
        const HUNTER_ACTIVITY_ID: &str = "AMqdiX2GpuYvsQOp";
        const SNIPER_KEY: &str = "strength-of-thousands-bestiary:RJKVH3fxPEiTCwt5";
        const SNIPER_ACTIVITY_ID: &str = "lLXZFku1wFZoAPdz";

        let target_path = unique_temp_path("shobhad-activity-identities.sqlite");
        let hunter_pack = PackName::new("pfs-season-3-bestiary")?;
        let sniper_pack = PackName::new("strength-of-thousands-bestiary")?;
        let mut hunter = fixture_record(&hunter_pack, "EB00f6ADElWInuix", "Shobhad Hunter");
        hunter.mechanics.activities = shobhad_activities(6, HUNTER_ACTIVITY_ID);
        let mut sniper = fixture_record(&sniper_pack, "RJKVH3fxPEiTCwt5", "Shobhad Sniper");
        sniper.mechanics.activities = shobhad_activities(2, SNIPER_ACTIVITY_ID);
        let expected = vec![hunter, sniper];

        write_fixture_records(&target_path, expected.clone(), Vec::new())?;
        let reader = crate::SqliteIndexReader::open_unpublished_read_only(&target_path)?;
        let full = reader.load_record_set()?.records;
        let requested = vec![RecordKey::parse(SNIPER_KEY)?, RecordKey::parse(HUNTER_KEY)?];
        let by_key = reader.load_records_by_key(&requested)?;

        for (records, path) in [(&full, "full"), (&by_key, "by_key")] {
            let hunter = records
                .iter()
                .find(|record| record.identity.key.to_string() == HUNTER_KEY)
                .expect("Shobhad Hunter must hydrate");
            assert_shobhad_duplicate_pair(
                &hunter.mechanics.activities,
                6,
                HUNTER_ACTIVITY_ID,
                path,
            );
            let sniper = records
                .iter()
                .find(|record| record.identity.key.to_string() == SNIPER_KEY)
                .expect("Shobhad Sniper must hydrate");
            assert_shobhad_duplicate_pair(
                &sniper.mechanics.activities,
                2,
                SNIPER_ACTIVITY_ID,
                path,
            );
        }
        assert_eq!(full, expected);
        assert_eq!(by_key.len(), 2);

        let _ = fs::remove_file(&target_path);
        Ok(())
    }

    #[test]
    fn interleaved_multi_parent_ordered_mechanics_round_trip_full_and_by_key()
    -> Result<(), Box<dyn std::error::Error>> {
        const FIRST_KEY: &str = "actions:testAction00";
        const SECOND_KEY: &str = "actions:testAction99";

        let target_path = unique_temp_path("interleaved-ordered-mechanics.sqlite");
        let pack_name = PackName::new("actions")?;
        let mut first = fixture_record(&pack_name, "testAction00", "Test Action 00");
        first.mechanics.metrics[0].value = MetricValue::Number(11.0);
        first.mechanics.metrics[1].value = MetricValue::Number(12.0);
        first.mechanics.activities[0].activity_id = "first-duplicate".to_string();
        first.mechanics.activities[1].activity_id = "first-duplicate".to_string();
        first.mechanics.spellcasting_entries[0].label = "First Prepared".to_string();
        first.mechanics.spellcasting_entries[1].label = "First Ritual".to_string();

        let mut second = fixture_record(&pack_name, "testAction99", "Test Action 99");
        second.mechanics.metrics[0].value = MetricValue::Number(91.0);
        second.mechanics.metrics[1].value = MetricValue::Number(92.0);
        second.mechanics.activities[0].activity_id = "second-duplicate".to_string();
        second.mechanics.activities[0].label = "Second Arcane Claw".to_string();
        second.mechanics.activities[1].activity_id = "second-duplicate".to_string();
        second.mechanics.activities[1].label = "Second Empty Activity".to_string();
        second.mechanics.spellcasting_entries[0].entry_id = "second-casting-1".to_string();
        second.mechanics.spellcasting_entries[0].label = "Second Prepared".to_string();
        second.mechanics.spellcasting_entries[1].entry_id = "second-casting-2".to_string();
        second.mechanics.spellcasting_entries[1].label = "Second Ritual".to_string();

        let expected = vec![first, second];
        write_fixture_records(&target_path, expected.clone(), Vec::new())?;

        let connection = Connection::open(&target_path)?;
        reinsert_ordered_mechanics_interleaved(&connection)?;
        for table_name in [
            "record_metrics",
            "record_activities",
            "record_spellcasting_entries",
        ] {
            assert_interleaved_storage_order(&connection, table_name, FIRST_KEY, SECOND_KEY)?;
        }
        drop(connection);

        let reader = crate::SqliteIndexReader::open_unpublished_read_only(&target_path)?;
        let full = reader.load_record_set()?.records;
        let reversed_keys = vec![RecordKey::parse(SECOND_KEY)?, RecordKey::parse(FIRST_KEY)?];
        let by_key = reader.load_records_by_key(&reversed_keys)?;

        for (hydration_path, records) in [("full", &full), ("by_key", &by_key)] {
            assert_eq!(
                records, &expected,
                "{hydration_path} hydration must preserve complete typed records"
            );
            for expected_record in &expected {
                let loaded = records
                    .iter()
                    .find(|record| record.identity.key == expected_record.identity.key)
                    .expect("requested parent must hydrate");
                assert_eq!(
                    loaded.mechanics.metrics, expected_record.mechanics.metrics,
                    "{hydration_path} metrics must retain parent-local ordinal order"
                );
                assert_eq!(
                    loaded.mechanics.activities, expected_record.mechanics.activities,
                    "{hydration_path} activities must retain parent-local ordinal order"
                );
                assert_eq!(
                    loaded.mechanics.spellcasting_entries,
                    expected_record.mechanics.spellcasting_entries,
                    "{hydration_path} spellcasting must retain parent-local ordinal order"
                );
                assert_eq!(
                    loaded.mechanics.activities[0].activity_id,
                    loaded.mechanics.activities[1].activity_id,
                    "{hydration_path} must preserve duplicate parent-local activity_id payloads"
                );
                assert_ne!(
                    loaded.mechanics.activities[0], loaded.mechanics.activities[1],
                    "{hydration_path} must preserve distinct Activity bodies at distinct ordinals"
                );
            }
        }

        let validation = reader.validate()?;
        assert_eq!(validation.status, ValidationStatus::Ok, "{validation:?}");

        let _ = fs::remove_file(&target_path);
        Ok(())
    }

    #[test]
    fn ordered_mechanics_reject_complete_negative_mutation_matrix()
    -> Result<(), Box<dyn std::error::Error>> {
        let base_path = unique_temp_path("ordered-mechanics-mutation-base.sqlite");
        let pack_name = PackName::new("actions")?;
        let source = fixture_record(&pack_name, "testAction00", "Test Action 00");
        let mut target = fixture_record(&pack_name, "testAction99", "Test Action 99");
        target.mechanics.metrics.clear();
        target.mechanics.activities.clear();
        target.mechanics.spellcasting_entries.clear();
        write_fixture_records(&base_path, vec![source, target], Vec::new())?;

        for (name, sql, expected_key) in [
            (
                "metric-missing",
                "UPDATE records SET metric_count=3 WHERE record_key='actions:testAction00'",
                "record_metrics[actions:testAction00].count",
            ),
            (
                "activity-missing",
                "UPDATE records SET activity_count=3 WHERE record_key='actions:testAction00'",
                "record_activities[actions:testAction00].count",
            ),
            (
                "spellcasting-missing",
                "UPDATE records SET spellcasting_entry_count=3 WHERE record_key='actions:testAction00'",
                "record_spellcasting_entries[actions:testAction00].count",
            ),
            (
                "metric-extra",
                "UPDATE records SET metric_count=1 WHERE record_key='actions:testAction00'",
                "record_metrics[actions:testAction00].count",
            ),
            (
                "activity-extra",
                "UPDATE records SET activity_count=1 WHERE record_key='actions:testAction00'",
                "record_activities[actions:testAction00].count",
            ),
            (
                "spellcasting-extra",
                "UPDATE records SET spellcasting_entry_count=1 WHERE record_key='actions:testAction00'",
                "record_spellcasting_entries[actions:testAction00].count",
            ),
            (
                "metric-deleted",
                "DELETE FROM record_metrics WHERE record_key='actions:testAction00' AND ordinal=1",
                "record_metrics[actions:testAction00].count",
            ),
            (
                "activity-deleted",
                "DELETE FROM record_activities WHERE record_key='actions:testAction00' AND ordinal=1",
                "record_activities[actions:testAction00].count",
            ),
            (
                "spellcasting-deleted",
                "DELETE FROM record_spellcasting_entries WHERE record_key='actions:testAction00' AND ordinal=1",
                "record_spellcasting_entries[actions:testAction00].count",
            ),
            (
                "metric-duplicated",
                "INSERT INTO record_metrics (record_key,ordinal,metric_domain,metric_key,value_type,number_value) VALUES ('actions:testAction00',2,'item','duplicate.metric','number',1)",
                "record_metrics[actions:testAction00].count",
            ),
            (
                "activity-duplicated",
                "INSERT INTO record_activities (record_key,activity_id,ordinal,payload_json) SELECT record_key,activity_id,2,payload_json FROM record_activities WHERE record_key='actions:testAction00' AND ordinal=1",
                "record_activities[actions:testAction00].count",
            ),
            (
                "spellcasting-duplicated",
                "INSERT INTO record_spellcasting_entries (record_key,entry_id,ordinal,payload_json) SELECT record_key,'casting-3',2,replace(payload_json,'casting-2','casting-3') FROM record_spellcasting_entries WHERE record_key='actions:testAction00' AND ordinal=1",
                "record_spellcasting_entries[actions:testAction00].count",
            ),
            (
                "metric-reordered",
                "UPDATE record_metrics SET ordinal=99 WHERE record_key='actions:testAction00' AND ordinal=0; UPDATE record_metrics SET ordinal=0 WHERE record_key='actions:testAction00' AND ordinal=1; UPDATE record_metrics SET ordinal=1 WHERE record_key='actions:testAction00' AND ordinal=99",
                "record_metrics[actions:testAction00].order_sha256",
            ),
            (
                "activity-reordered",
                "UPDATE record_activities SET ordinal=99 WHERE record_key='actions:testAction00' AND ordinal=0; UPDATE record_activities SET ordinal=0 WHERE record_key='actions:testAction00' AND ordinal=1; UPDATE record_activities SET ordinal=1 WHERE record_key='actions:testAction00' AND ordinal=99",
                "record_activities[actions:testAction00].order_sha256",
            ),
            (
                "spellcasting-reordered",
                "UPDATE record_spellcasting_entries SET ordinal=99 WHERE record_key='actions:testAction00' AND ordinal=0; UPDATE record_spellcasting_entries SET ordinal=0 WHERE record_key='actions:testAction00' AND ordinal=1; UPDATE record_spellcasting_entries SET ordinal=1 WHERE record_key='actions:testAction00' AND ordinal=99",
                "record_spellcasting_entries[actions:testAction00].order_sha256",
            ),
            (
                "metric-reparented",
                "UPDATE record_metrics SET record_key='actions:testAction99',ordinal=0 WHERE record_key='actions:testAction00' AND ordinal=1",
                "record_metrics[actions:testAction00].count",
            ),
            (
                "activity-reparented",
                "UPDATE record_activities SET record_key='actions:testAction99',ordinal=0 WHERE record_key='actions:testAction00' AND ordinal=1",
                "record_activities[actions:testAction00].count",
            ),
            (
                "spellcasting-reparented",
                "UPDATE record_spellcasting_entries SET record_key='actions:testAction99',ordinal=0 WHERE record_key='actions:testAction00' AND ordinal=1",
                "record_spellcasting_entries[actions:testAction00].count",
            ),
            (
                "activity-malformed",
                "UPDATE record_activities SET payload_json='{\"bad\":true}' WHERE record_key='actions:testAction00' AND ordinal=0",
                "record_activities[actions:testAction00:0].payload_json",
            ),
            (
                "spellcasting-malformed",
                "UPDATE record_spellcasting_entries SET payload_json='{\"bad\":true}' WHERE record_key='actions:testAction00' AND ordinal=0",
                "record_spellcasting_entries[actions:testAction00:casting-1].payload_json",
            ),
            (
                "activity-payload-id-divergence",
                "UPDATE record_activities SET activity_id='relational-only' WHERE record_key='actions:testAction00' AND ordinal=0",
                "record_activities[actions:testAction00:0].payload_json",
            ),
            (
                "spellcasting-payload-id-divergence",
                "UPDATE record_spellcasting_entries SET entry_id='relational-only' WHERE record_key='actions:testAction00' AND ordinal=0",
                "record_spellcasting_entries[actions:testAction00:relational-only].payload_json",
            ),
        ] {
            assert_mechanics_mutation_rejected(&base_path, name, sql, expected_key)?;
        }
        assert_malformed_metric_mutation_rejected(&base_path)?;

        let connection = Connection::open(&base_path)?;
        for (name, sql, constraint) in [
            (
                "metric-duplicate-ordinal",
                "INSERT INTO record_metrics (record_key,ordinal,metric_domain,metric_key,value_type,number_value) VALUES ('actions:testAction00',0,'item','duplicate.metric','number',1)",
                "record_metrics.record_key, record_metrics.ordinal",
            ),
            (
                "activity-duplicate-ordinal",
                "INSERT INTO record_activities (record_key,activity_id,ordinal,payload_json) SELECT record_key,'duplicate',ordinal,payload_json FROM record_activities WHERE record_key='actions:testAction00' AND ordinal=0",
                "record_activities.record_key, record_activities.ordinal",
            ),
            (
                "spellcasting-duplicate-ordinal",
                "INSERT INTO record_spellcasting_entries (record_key,entry_id,ordinal,payload_json) SELECT record_key,'casting-3',ordinal,replace(payload_json,'casting-1','casting-3') FROM record_spellcasting_entries WHERE record_key='actions:testAction00' AND ordinal=0",
                "record_spellcasting_entries.record_key, record_spellcasting_entries.ordinal",
            ),
            (
                "spellcasting-duplicate-id",
                "INSERT INTO record_spellcasting_entries (record_key,entry_id,ordinal,payload_json) SELECT record_key,entry_id,99,payload_json FROM record_spellcasting_entries WHERE record_key='actions:testAction00' AND ordinal=0",
                "record_spellcasting_entries.record_key, record_spellcasting_entries.entry_id",
            ),
        ] {
            let error = connection
                .execute_batch(sql)
                .expect_err("duplicate ordered child identity must be rejected");
            assert!(error.to_string().contains(constraint), "{name}: {error}");
        }

        let _ = fs::remove_file(&base_path);
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
            references: Vec::new(),
            aliases: Vec::new(),
            remaster_links: Vec::new(),
            pending_document_embeddings: Vec::new(),
            document_embeddings: Vec::new(),
        };

        let error = write_artifact(&target_path, &input, EmbeddingModelId::BgeSmallEnV15)
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
            references: Vec::new(),
            aliases: Vec::new(),
            remaster_links: Vec::new(),
            pending_document_embeddings: Vec::new(),
            document_embeddings: Vec::new(),
        };

        let error = write_artifact(&target_path, &input, EmbeddingModelId::BgeSmallEnV15)
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
            references: Vec::new(),
            aliases: Vec::new(),
            remaster_links: Vec::new(),
            pending_document_embeddings: Vec::new(),
            document_embeddings: Vec::new(),
        };

        let error = write_artifact(&target_path, &input, EmbeddingModelId::BgeSmallEnV15)
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
                IndexBuildPack {
                    name: pack_name,
                    label: name.clone(),
                    document_type: "Item".to_string(),
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
            canonical_bodies: Vec::new(),
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

    fn shobhad_activities(prefix_count: usize, duplicate_id: &str) -> Vec<MechanicActivity> {
        let mut activities = (0..prefix_count)
            .map(|ordinal| MechanicActivity {
                activity_id: format!("fixture-prefix-{ordinal}"),
                label: format!("Prefix {ordinal}"),
                kind: MechanicActivityKind::Other,
                traits: Vec::new(),
                compendium_source: None,
                usage: MechanicActivityUsage::Unlimited,
                rolls: Vec::new(),
                damage: Vec::new(),
                modes: Vec::new(),
            })
            .collect::<Vec<_>>();
        activities.extend([0, 1].map(|duplicate_ordinal| MechanicActivity {
            activity_id: duplicate_id.to_string(),
            label: "Four-Armed".to_string(),
            kind: MechanicActivityKind::Other,
            traits: Vec::new(),
            compendium_source: None,
            usage: if duplicate_ordinal == 0 {
                MechanicActivityUsage::Limited
            } else {
                MechanicActivityUsage::Unlimited
            },
            rolls: Vec::new(),
            damage: Vec::new(),
            modes: Vec::new(),
        }));
        activities
    }

    fn assert_shobhad_duplicate_pair(
        activities: &[MechanicActivity],
        first_ordinal: usize,
        activity_id: &str,
        hydration_path: &str,
    ) {
        let pair = &activities[first_ordinal..first_ordinal + 2];
        assert_eq!(
            pair.iter()
                .map(|activity| activity.activity_id.as_str())
                .collect::<Vec<_>>(),
            vec![activity_id, activity_id],
            "{hydration_path} hydration must preserve duplicate activity_id payloads"
        );
        assert!(pair.iter().all(|activity| activity.label == "Four-Armed"));
        assert!(
            pair.iter()
                .all(|activity| activity.kind == MechanicActivityKind::Other)
        );
        assert_eq!(pair[0].usage, MechanicActivityUsage::Limited);
        assert_eq!(pair[1].usage, MechanicActivityUsage::Unlimited);
    }

    fn reinsert_ordered_mechanics_interleaved(
        connection: &Connection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        connection.execute_batch(
            "CREATE TEMP TABLE interleaved_metrics AS SELECT * FROM record_metrics;
             CREATE TEMP TABLE interleaved_activities AS SELECT * FROM record_activities;
             CREATE TEMP TABLE interleaved_spellcasting AS
                 SELECT * FROM record_spellcasting_entries;

             DELETE FROM record_metrics;
             INSERT INTO record_metrics (
                 record_key,ordinal,metric_domain,metric_key,value_type,
                 number_value,text_value,bool_value
             )
             SELECT record_key,ordinal,metric_domain,metric_key,value_type,
                    number_value,text_value,bool_value
             FROM interleaved_metrics
             ORDER BY ordinal DESC,record_key DESC;

             DELETE FROM record_activities;
             INSERT INTO record_activities (record_key,activity_id,ordinal,payload_json)
             SELECT record_key,activity_id,ordinal,payload_json
             FROM interleaved_activities
             ORDER BY ordinal DESC,record_key DESC;

             DELETE FROM record_spellcasting_entries;
             INSERT INTO record_spellcasting_entries (record_key,entry_id,ordinal,payload_json)
             SELECT record_key,entry_id,ordinal,payload_json
             FROM interleaved_spellcasting
             ORDER BY ordinal DESC,record_key DESC;

             DROP TABLE interleaved_metrics;
             DROP TABLE interleaved_activities;
             DROP TABLE interleaved_spellcasting;",
        )?;
        Ok(())
    }

    fn assert_interleaved_storage_order(
        connection: &Connection,
        table_name: &str,
        first_key: &str,
        second_key: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut statement = connection.prepare(&format!(
            "SELECT record_key,ordinal FROM {table_name} ORDER BY rowid"
        ))?;
        let actual = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(
            actual,
            vec![
                (second_key.to_string(), 1),
                (first_key.to_string(), 1),
                (second_key.to_string(), 0),
                (first_key.to_string(), 0),
            ],
            "{table_name} rows must be physically interleaved across parents with non-sorted parent-local ordinals"
        );
        Ok(())
    }

    fn assert_mechanics_mutation_rejected(
        base_path: &Path,
        name: &str,
        sql: &str,
        expected_key: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = unique_temp_path(&format!("ordered-mechanics-{name}.sqlite"));
        fs::copy(base_path, &path)?;
        let connection = Connection::open(&path)?;
        connection.execute_batch(sql)?;
        drop(connection);

        let report = crate::SqliteIndexReader::open_unpublished_read_only(&path)?.validate()?;
        assert_eq!(report.status, ValidationStatus::Error, "{name}: {report:?}");
        assert_eq!(
            report.code,
            crate::ValidationCode::ArtifactContractViolation,
            "{name}: {report:?}"
        );
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

    fn assert_malformed_metric_mutation_rejected(
        base_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = unique_temp_path("ordered-mechanics-metric-malformed.sqlite");
        fs::copy(base_path, &path)?;
        let connection = Connection::open(&path)?;
        connection.execute_batch(
            "PRAGMA ignore_check_constraints=ON; UPDATE record_metrics SET value_type='text',number_value=NULL,text_value=NULL WHERE record_key='actions:testAction00' AND ordinal=0",
        )?;
        drop(connection);

        let report = crate::SqliteIndexReader::open_unpublished_read_only(&path)?.validate()?;
        assert_eq!(report.status, ValidationStatus::Error, "{report:?}");
        assert_eq!(
            report.code,
            crate::ValidationCode::ArtifactContractViolation,
            "{report:?}"
        );
        let diagnostic = report
            .diagnostics
            .first()
            .expect("malformed metric produces a typed diagnostic");
        assert_eq!(diagnostic.family, crate::ArtifactValidationFamily::Data);
        assert_eq!(diagnostic.key.as_deref(), Some("record_metrics:text_value"));
        assert_eq!(
            diagnostic.message,
            "metric value shape `record_metrics:text_value` is inconsistent with value_type"
        );
        assert_eq!(
            diagnostic.expected.as_deref(),
            Some("exactly one matching value column")
        );
        assert_eq!(diagnostic.actual.as_deref(), Some("1 invalid rows"));
        let _ = fs::remove_file(path);
        Ok(())
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
                    foundry_type: Some(ItemTypeMechanics::Spell(SpellMechanics {
                        traditions: vec!["arcane".to_string()],
                        kinds: vec!["spell".to_string()],
                        range: Some(SpellRange {
                            text: "30 feet".to_string(),
                            distance: Some(30.0),
                        }),
                        target: Some(SpellTarget {
                            text: "1 creature".to_string(),
                        }),
                        area: Some(SpellArea {
                            kind: Some("burst".to_string()),
                            value: Some(10.0),
                        }),
                        defense: Some(SpellDefense {
                            save: Some("will".to_string()),
                            basic: true,
                        }),
                        sustained: true,
                        damage_types: vec!["mental".to_string()],
                    })),
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
                spellcasting_entries: vec![
                    SpellcastingEntryMechanics {
                        entry_id: "casting-1".to_string(),
                        label: "Arcane Prepared Spells".to_string(),
                        preparation: SpellcastingPreparation::Prepared,
                        spell_attack: Some(17),
                        spell_dc: Some(27),
                    },
                    SpellcastingEntryMechanics {
                        entry_id: "casting-2".to_string(),
                        label: "Rituals".to_string(),
                        preparation: SpellcastingPreparation::Other("ritual".to_string()),
                        spell_attack: None,
                        spell_dc: None,
                    },
                ],
                activities: vec![
                    MechanicActivity {
                        activity_id: "activity-1".to_string(),
                        label: "Arcane Claw".to_string(),
                        kind: MechanicActivityKind::Strike,
                        traits: vec!["attack".to_string(), "magical".to_string()],
                        compendium_source: Some(
                            "Compendium.pf2e.actionspf2e.Item.test".to_string(),
                        ),
                        usage: MechanicActivityUsage::Limited,
                        rolls: vec![ActivityRoll {
                            roll_id: "attack".to_string(),
                            label: "Attack".to_string(),
                            base_value: 17,
                            surface: ActivityRollSurface::AttackRoll,
                            ability: Some(ActivityRollAbility::Strength),
                        }],
                        damage: vec![DamageExpression {
                            damage_id: "base".to_string(),
                            label: Some("slashing".to_string()),
                            formula: "2d6+4".to_string(),
                            damage_type: Some("slashing".to_string()),
                            effect_kind: DamageEffectKind::Damage,
                            ability: Some(ActivityRollAbility::Strength),
                        }],
                        modes: vec![MechanicActivityMode {
                            mode_id: "two-action".to_string(),
                            label: "Two Actions".to_string(),
                            sort: 1,
                            target: Some("one creature".to_string()),
                            range: Some("reach 10 feet".to_string()),
                            time: Some("2 actions".to_string()),
                            damage: vec![DamageExpression {
                                damage_id: "mode".to_string(),
                                label: None,
                                formula: "3d6+4".to_string(),
                                damage_type: Some("force".to_string()),
                                effect_kind: DamageEffectKind::Damage,
                                ability: None,
                            }],
                        }],
                    },
                    MechanicActivity {
                        activity_id: "activity-1".to_string(),
                        label: "Empty Activity".to_string(),
                        kind: MechanicActivityKind::Other,
                        traits: Vec::new(),
                        compendium_source: None,
                        usage: MechanicActivityUsage::Unlimited,
                        rolls: Vec::new(),
                        damage: Vec::new(),
                        modes: Vec::new(),
                    },
                ],
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
