//! Persisted trait vocabulary and bounded, exact authored associations.
use crate::{IndexError, SqliteIndexReader};
use atlas_domain::{RecordKey, SearchProfileId, SourceRecordSummary, TraitKey};
use atlas_record::source_content::OwnedContentLocator;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

pub const TRAIT_SCHEMA_SQL: &str = "
CREATE TABLE trait_definitions(key TEXT PRIMARY KEY, definition_json TEXT NOT NULL CHECK(json_valid(definition_json)),definition_hash TEXT NOT NULL) STRICT;
CREATE TABLE trait_vectors(trait_key TEXT NOT NULL REFERENCES trait_definitions(key) ON DELETE CASCADE,profile_id TEXT NOT NULL,input_hash TEXT NOT NULL,vector_json TEXT NOT NULL CHECK(json_valid(vector_json)),chunks_json TEXT NOT NULL CHECK(json_valid(chunks_json)),vector_hash TEXT NOT NULL,PRIMARY KEY(trait_key,profile_id)) WITHOUT ROWID,STRICT;
CREATE TABLE trait_memberships(trait_key TEXT NOT NULL,record_key TEXT NOT NULL REFERENCES records(key) DEFERRABLE INITIALLY DEFERRED,owners_json TEXT NOT NULL CHECK(json_valid(owners_json)),source_label TEXT,PRIMARY KEY(trait_key,record_key,owners_json)) WITHOUT ROWID,STRICT;
CREATE TABLE trait_references(trait_key TEXT NOT NULL REFERENCES trait_definitions(key) ON DELETE CASCADE,record_key TEXT NOT NULL REFERENCES records(key) DEFERRABLE INITIALLY DEFERRED,evidence TEXT NOT NULL,PRIMARY KEY(trait_key,record_key,evidence)) WITHOUT ROWID,STRICT;
";
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraitSourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraitDefinition {
    pub key: TraitKey,
    pub label: String,
    pub label_key: Option<String>,
    pub description_key: Option<String>,
    pub description_html: Option<String>,
    pub authored_markup: Option<String>,
    pub prepared: Option<atlas_record::source_content::PreparedTraitDefinition>,
    pub catalogs: Vec<String>,
    pub source_locations: Vec<TraitSourceLocation>,
    pub source_revision: Option<String>,
    pub indexing_locale: String,
}
#[derive(Debug, Clone)]
pub struct TraitVectorInput {
    pub key: TraitKey,
    pub profile_id: SearchProfileId,
    pub input_hash: String,
    pub vector: Vec<f32>,
    pub chunks: Vec<TraitSeedChunk>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitSeedChunk {
    pub input: String,
    pub input_hash: String,
    pub token_count: usize,
    pub address: atlas_domain::SourcePassageAddress,
    pub vector: Vec<f32>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitReferenceInput {
    pub source_label: Option<String>,
    pub owners: Vec<OwnedContentLocator>,
    pub field: Option<String>,
    pub direction: String,
    pub occurrence_path: String,
    pub key: TraitKey,
    pub record: RecordKey,
    pub evidence: String,
}
#[derive(Debug, Clone)]
pub struct TraitMembership {
    pub source_label: Option<String>,
    pub record: SourceRecordSummary,
    pub owners: Vec<OwnedContentLocator>,
}

pub(crate) fn write_traits(
    c: &Connection,
    input: &crate::IndexBuildInput,
) -> Result<(), IndexError> {
    c.execute("DELETE FROM trait_definitions", [])?;
    for definition in &input.traits {
        if definition.label.trim().is_empty() {
            return Err(IndexError::Invalid("empty trait label".into()));
        }
        validate_definition(definition)?;
        c.execute(
            "INSERT INTO trait_definitions VALUES(?1,?2,?3)",
            params![
                definition.key.to_string(),
                serde_json::to_string(definition)?,
                crate::codec::sha256(serde_json::to_string(definition)?.as_bytes())
            ],
        )?;
    }
    for unit in &input.trait_vectors {
        let profile = input.context.search_profile(&unit.profile_id)?;
        if unit.vector.len() != profile.dimensions
            || unit.vector.iter().any(|v| !v.is_finite())
            || !crate::codec::checked_hash(&unit.input_hash)
        {
            return Err(IndexError::Invalid("invalid trait vector".into()));
        }
        c.execute(
            "INSERT INTO trait_vectors VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                unit.key.to_string(),
                unit.profile_id.as_str(),
                unit.input_hash,
                serde_json::to_string(&unit.vector)?,
                serde_json::to_string(&unit.chunks)?,
                crate::codec::sha256(serde_json::to_string(&unit.vector)?.as_bytes())
            ],
        )?;
    }
    c.execute("DELETE FROM trait_memberships", [])?;
    for membership in project_memberships(&input.records, input.context.audience)? {
        c.execute(
            "INSERT INTO trait_memberships VALUES(?1,?2,?3,?4)",
            params![
                membership.key,
                membership.record.to_string(),
                serde_json::to_string(&membership.owners)?,
                membership.source_label
            ],
        )?;
    }
    for reference in &project_references(&input.records, &input.traits, input.context.audience) {
        c.execute(
            "INSERT INTO trait_references VALUES(?1,?2,?3)",
            params![
                reference.key.to_string(),
                reference.record.to_string(),
                serde_json::to_string(reference)?
            ],
        )?;
    }
    Ok(())
}
impl SqliteIndexReader {
    pub fn trait_definition(&self, key: &TraitKey) -> Result<Option<TraitDefinition>, IndexError> {
        let row: Option<(String, String)> = self
            .native()?
            .query_row(
                "SELECT definition_json,definition_hash FROM trait_definitions WHERE key=?1",
                [key.to_string()],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        row.map(|(json, hash)| {
            if crate::codec::sha256(json.as_bytes()) != hash {
                return Err(IndexError::Invalid(
                    "trait definition checksum mismatch".into(),
                ));
            }
            let definition = serde_json::from_str(&json)?;
            validate_definition(&definition)?;
            Ok(definition)
        })
        .transpose()
    }
    pub fn trait_memberships(
        &self,
        key: &TraitKey,
        limit: u32,
        offset: u32,
    ) -> Result<(Vec<TraitMembership>, u64), IndexError> {
        if limit == 0 || limit > 100 {
            return Err(IndexError::Invalid("trait page size must be 1..100".into()));
        }
        let sql = "SELECT record_key,owners_json AS owners,source_label FROM trait_memberships WHERE trait_key=?1";
        let total = self.native()?.query_row(
            &format!("SELECT count(*) FROM ({sql})"),
            [key.to_string()],
            |r| r.get(0),
        )?;
        let mut stmt = self.native()?.prepare(&format!(
            "SELECT record_key,owners,source_label FROM ({sql}) ORDER BY record_key,owners LIMIT ?2 OFFSET ?3"
        ))?;
        let rows = stmt
            .query_map(params![key.to_string(), limit, offset], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<String>>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let mut entries = Vec::new();
        for (record, owners, source_label) in rows {
            let record: RecordKey = record
                .parse()
                .map_err(|e| IndexError::Invalid(format!("invalid membership key: {e}")))?;
            if let Some(record) = self.read_summary(&record)? {
                let owners = if owners == "[]" {
                    Vec::new()
                } else {
                    serde_json::from_str(&owners)?
                };
                entries.push(TraitMembership {
                    record,
                    owners,
                    source_label,
                });
            }
        }
        Ok((entries, total))
    }
    pub fn trait_references(
        &self,
        key: &TraitKey,
        limit: u32,
        offset: u32,
    ) -> Result<(Vec<SourceRecordSummary>, u64), IndexError> {
        if limit == 0 || limit > 100 {
            return Err(IndexError::Invalid("trait page size must be 1..100".into()));
        }
        let total = self.native()?.query_row(
            "SELECT count(DISTINCT record_key) FROM trait_references WHERE trait_key=?1",
            [key.to_string()],
            |r| r.get(0),
        )?;
        let mut stmt=self.native()?.prepare("SELECT DISTINCT record_key FROM trait_references WHERE trait_key=?1 ORDER BY record_key LIMIT ?2 OFFSET ?3")?;
        let keys = stmt
            .query_map(params![key.to_string(), limit, offset], |r| {
                r.get::<_, String>(0)
            })?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|s| {
                s.parse()
                    .map_err(|e| IndexError::Invalid(format!("invalid reference key: {e}")))
            })
            .collect::<Result<Vec<RecordKey>, _>>()?;
        Ok((self.read_summaries(&keys)?.records, total))
    }
    pub fn trait_reference_entries(
        &self,
        key: &TraitKey,
        limit: u32,
        offset: u32,
    ) -> Result<(Vec<TraitReference>, u64), IndexError> {
        if limit == 0 || limit > 100 {
            return Err(IndexError::Invalid("trait page size must be 1..100".into()));
        }
        let total = self.native()?.query_row(
            "SELECT count(*) FROM trait_references WHERE trait_key=?1",
            [key.to_string()],
            |r| r.get(0),
        )?;
        let mut stmt=self.native()?.prepare("SELECT evidence FROM trait_references WHERE trait_key=?1 ORDER BY record_key,evidence LIMIT ?2 OFFSET ?3")?;
        let rows = stmt
            .query_map(params![key.to_string(), limit, offset], |r| {
                r.get::<_, String>(0)
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let mut entries = Vec::new();
        for json in rows {
            let source: TraitReferenceInput = serde_json::from_str(&json)?;
            if let Some(record) = self.read_summary(&source.record)? {
                entries.push(TraitReference {
                    record,
                    source_label: source.source_label,
                    owners: source.owners,
                    field: source.field,
                    direction: source.direction,
                    occurrence_path: source.occurrence_path,
                    evidence: source.evidence,
                });
            }
        }
        Ok((entries, total))
    }
    pub fn trait_vector(
        &self,
        key: &TraitKey,
        profile: &SearchProfileId,
    ) -> Result<Option<Vec<f32>>, IndexError> {
        let row: Option<(String,String)> = self
            .native()?
            .query_row(
                "SELECT vector_json,vector_hash FROM trait_vectors WHERE trait_key=?1 AND profile_id=?2",
                params![key.to_string(), profile.as_str()],
                |r| Ok((r.get(0)?,r.get(1)?)),
            )
            .optional()?;
        row.map(|(json, hash)| {
            if crate::codec::sha256(json.as_bytes()) != hash {
                return Err(IndexError::Invalid("trait vector checksum mismatch".into()));
            }
            let vector: Vec<f32> = serde_json::from_str(&json)?;
            let model = self.context().search_profile(profile)?;
            if vector.len() != model.dimensions || vector.iter().any(|v| !v.is_finite()) {
                return Err(IndexError::Invalid("trait vector identity mismatch".into()));
            }
            Ok(vector)
        })
        .transpose()
    }
}

fn validate_definition(definition: &TraitDefinition) -> Result<(), IndexError> {
    match (
        &definition.prepared,
        &definition.authored_markup,
        &definition.description_html,
    ) {
        (None, None, None) => {}
        (Some(prepared), Some(markup), Some(html)) => {
            if prepared.key != definition.key
                || &prepared.html != html
                || prepared.authored_markup_sha256 != crate::codec::sha256(markup.as_bytes())
                || prepared.interpretation_version
                    != atlas_record::source_content::CONTENT_INTERPRETATION_VERSION
            {
                return Err(IndexError::Invalid(
                    "trait prepared-content identity mismatch".into(),
                ));
            }
            atlas_record::source_content::validate_prepared_markers(html, &prepared.interactions)
                .map_err(|e| IndexError::Invalid(e.to_string()))?;
            atlas_record::source_content::validate_prepared_trait_reference_bindings(
                html,
                &definition.key,
                &prepared.authored_markup_sha256,
                &prepared.references,
            )
            .map_err(|e| IndexError::Invalid(e.to_string()))?;
        }
        _ => {
            return Err(IndexError::Invalid(
                "incomplete trait source/preparation snapshot".into(),
            ));
        }
    }
    Ok(())
}
pub(crate) fn validate_traits(
    c: &Connection,
    context: &crate::SourceArtifactBuildContext,
) -> Result<(), IndexError> {
    // Every published definition, including label-only definitions, supplies a
    // seed for every configured profile. An artifact with no profiles remains
    // valid metadata-only reference data.
    let missing:Option<(String,String)>=c.query_row("SELECT d.key,p.profile_id FROM trait_definitions d CROSS JOIN search_profiles p WHERE NOT EXISTS (SELECT 1 FROM trait_vectors v WHERE v.trait_key=d.key AND v.profile_id=p.profile_id) LIMIT 1",[],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
    if let Some((key, profile)) = missing {
        return Err(IndexError::Invalid(format!(
            "trait `{key}` has no seed vector for configured search profile `{profile}`"
        )));
    }
    let mut stmt = c.prepare("SELECT key,definition_json FROM trait_definitions ORDER BY key")?;
    for row in stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))? {
        let (key, json) = row?;
        let hash: String = c.query_row(
            "SELECT definition_hash FROM trait_definitions WHERE key=?1",
            [&key],
            |r| r.get(0),
        )?;
        if hash != crate::codec::sha256(json.as_bytes()) {
            return Err(IndexError::Invalid(
                "trait definition checksum mismatch".into(),
            ));
        }
        let definition: TraitDefinition = serde_json::from_str(&json)?;
        if definition.key.to_string() != key
            || definition.indexing_locale != context.indexing_locale
            || definition.source_revision != context.source_revision
        {
            return Err(IndexError::Invalid(
                "trait publication provenance mismatch".into(),
            ));
        }
        validate_definition(&definition)?;
    }
    let mut stmt=c.prepare("SELECT tv.profile_id,tv.input_hash,tv.vector_json,tv.chunks_json,d.definition_json,tv.vector_hash FROM trait_vectors tv JOIN trait_definitions d ON d.key=tv.trait_key")?;
    for row in stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, String>(5)?,
        ))
    })? {
        let (profile, hash, json, chunks, definition, vector_hash) = row?;
        if vector_hash != crate::codec::sha256(json.as_bytes()) {
            return Err(IndexError::Invalid("trait vector checksum mismatch".into()));
        }
        let model = context.search_profile(&SearchProfileId::new(profile))?;
        let definition: TraitDefinition = serde_json::from_str(&definition)?;
        let vector: Vec<f32> = serde_json::from_str(&json)?;
        let chunks: Vec<TraitSeedChunk> = serde_json::from_str(&chunks)?;
        validate_seed(&definition, model, &hash, &vector, &chunks)?;
        let identity = serde_json::to_string(model)?;
        for chunk in &chunks {
            let spec = atlas_embedding::search_profile(&model.profile_id)
                .map_err(|e| IndexError::Unsupported(e.to_string()))?
                .model_spec();
            let reuse_key = spec.document_reuse_key(&chunk.input);
            let checkpoint: Option<(Vec<u8>, String)> = c
                .query_row(
                    TRAIT_CHECKPOINT_SELECT,
                    params![
                        reuse_key,
                        identity,
                        chunk.input_hash,
                        chunk.token_count,
                        model.dimensions
                    ],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()?;
            let Some((bytes, checksum)) = checkpoint else {
                return Err(IndexError::Invalid("trait chunk checkpoint missing".into()));
            };
            if crate::codec::sha256(&bytes) != checksum
                || crate::read_units::decode_vector(&bytes)? != chunk.vector
            {
                return Err(IndexError::Invalid(
                    "trait chunk checkpoint identity mismatch".into(),
                ));
            }
        }
    }
    let records = crate::read_persisted_inputs(c, context)?;
    let definitions = c
        .prepare("SELECT definition_json FROM trait_definitions")?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|j| serde_json::from_str::<TraitDefinition>(&j).map_err(IndexError::from))
        .collect::<Result<Vec<_>, _>>()?;
    let expected = project_references(&records, &definitions, context.audience)
        .into_iter()
        .map(|r| {
            Ok((
                r.key.to_string(),
                r.record.to_string(),
                serde_json::to_string(&r)?,
            ))
        })
        .collect::<Result<std::collections::BTreeSet<_>, IndexError>>()?;
    let actual = c
        .prepare("SELECT trait_key,record_key,evidence FROM trait_references")?
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?
        .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
    if actual != expected {
        return Err(IndexError::Invalid(
            "trait reference evidence differs from checked source".into(),
        ));
    }
    let expected = project_memberships(&records, context.audience)?
        .into_iter()
        .map(|membership| {
            Ok((
                membership.key,
                membership.record.to_string(),
                serde_json::to_string(&membership.owners)?,
                membership.source_label,
            ))
        })
        .collect::<Result<std::collections::BTreeSet<_>, IndexError>>()?;
    let actual = c
        .prepare("SELECT trait_key,record_key,owners_json,source_label FROM trait_memberships")?
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
            ))
        })?
        .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
    if actual != expected {
        return Err(IndexError::Invalid(
            "trait memberships differ from checked source".into(),
        ));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    fn refresh_fixture(input: &mut crate::IndexBuildInput) {
        input.lexical_units.clear();
        let hash = input.context.preparation_sha256().unwrap();
        for source in &mut input.records {
            source.preparation_context_hash = hash.clone();
            let label = &input
                .packs
                .iter()
                .find(|p| p.pack_id == source.record.key().pack().as_str())
                .unwrap()
                .label;
            let selection = atlas_record::source_record::select_record_search(
                &source.record,
                &source.content,
                input.context.audience,
                label,
                &input.context.used_trait_labels,
            )
            .unwrap();
            input
                .lexical_units
                .extend(crate::project_lexical_units(&selection, &[]));
        }
    }
    fn definition(input: &crate::IndexBuildInput, id: &str) -> TraitDefinition {
        TraitDefinition {
            key: TraitKey::new(id).unwrap(),
            label: id.into(),
            label_key: None,
            description_key: None,
            description_html: None,
            authored_markup: None,
            prepared: None,
            catalogs: vec![],
            source_locations: vec![],
            source_revision: input.context.source_revision.clone(),
            indexing_locale: input.context.indexing_locale.clone(),
        }
    }
    #[test]
    fn trait_seed_rejects_changed_vector_definition_and_incomplete_coverage() {
        let input = crate::foundation_tests::fixture(true);
        let definition = definition(&input, "agile");
        let text = "agile\n";
        let mut vector = vec![0.; 768];
        vector[0] = 1.;
        let chunk = TraitSeedChunk {
            input: text.into(),
            input_hash: crate::codec::sha256(text.as_bytes()),
            token_count: 2,
            address: atlas_domain::SourcePassageAddress::PlainSection {
                source_text_sha256: crate::codec::sha256(text.as_bytes()),
                selection_version: "trait-definition-centroid/v1".into(),
                section_ordinal: 0,
                label: Some("agile".into()),
                chunk_bytes: atlas_domain::SourceByteRange {
                    start: 0,
                    end: text.len(),
                },
            },
            vector: vector.clone(),
        };
        let hash = crate::codec::sha256(
            format!("trait-definition-centroid/v1:{}", chunk.input_hash).as_bytes(),
        );
        let model = &input.context.search_profiles[0];
        assert!(
            validate_seed(
                &definition,
                model,
                &hash,
                &vector,
                std::slice::from_ref(&chunk)
            )
            .is_ok()
        );
        let mut changed = vector.clone();
        changed[0] = 0.;
        changed[1] = 1.;
        assert!(
            validate_seed(
                &definition,
                model,
                &hash,
                &changed,
                std::slice::from_ref(&chunk)
            )
            .is_err()
        );
        let mut changed = definition.clone();
        changed.label = "different".into();
        assert!(
            validate_seed(
                &changed,
                model,
                &hash,
                &vector,
                std::slice::from_ref(&chunk)
            )
            .is_err()
        );
        let mut short = chunk;
        let atlas_domain::SourcePassageAddress::PlainSection { chunk_bytes, .. } =
            &mut short.address
        else {
            unreachable!()
        };
        chunk_bytes.end -= 1;
        assert!(validate_seed(&definition, model, &hash, &vector, &[short]).is_err());
    }
    #[test]
    fn trait_vector_publication_requires_exact_native_checkpoints() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("workspace.sqlite");
        let mut input = crate::foundation_tests::fixture(true);
        let definition = definition(&input, "agile");
        let key = definition.key.clone();
        input.traits.push(definition);
        let text = "agile\n";
        let mut vector = vec![0.; 768];
        vector[0] = 1.;
        let hash = crate::codec::sha256(text.as_bytes());
        let chunk = TraitSeedChunk {
            input: text.into(),
            input_hash: hash.clone(),
            token_count: 2,
            address: atlas_domain::SourcePassageAddress::PlainSection {
                source_text_sha256: hash.clone(),
                selection_version: "trait-definition-centroid/v1".into(),
                section_ordinal: 0,
                label: Some("agile".into()),
                chunk_bytes: atlas_domain::SourceByteRange {
                    start: 0,
                    end: text.len(),
                },
            },
            vector: vector.clone(),
        };
        input.trait_vectors.push(TraitVectorInput {
            key,
            profile_id: SearchProfileId::new("bge-base"),
            input_hash: crate::codec::sha256(
                format!("trait-definition-centroid/v1:{hash}").as_bytes(),
            ),
            vector: vector.clone(),
            chunks: vec![chunk],
        });
        // Multiple traits may share exact native inputs. Pruning must decode the
        // vocabulary once, even with many unrelated retained checkpoint rows.
        for ordinal in 0..128 {
            let key = TraitKey::new(format!("fixture-{ordinal}")).unwrap();
            let mut definition = input.traits[0].clone();
            definition.key = key.clone();
            input.traits.push(definition);
            let mut seed = input.trait_vectors[0].clone();
            seed.key = key;
            input.trait_vectors.push(seed);
        }
        crate::test_support::publish_input(&path, &input).unwrap();
        let workspace = crate::test_support::initialize_workspace(&path).unwrap();
        {
            let owner = workspace.update_owner().unwrap();
            let write = workspace
                .write(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap();
            write.connection().execute_batch("WITH RECURSIVE obsolete(n) AS (VALUES(1) UNION ALL SELECT n+1 FROM obsolete WHERE n<10000)
                INSERT INTO embedding_checkpoints SELECT 'obsolete-'||n,model_identity,printf('%064x',n),token_count,dimensions,vector,vector_hash,touched_job FROM obsolete CROSS JOIN (SELECT * FROM embedding_checkpoints LIMIT 1);").unwrap();
            assert_eq!(
                write
                    .connection()
                    .query_row("SELECT count(*) FROM embedding_checkpoints", [], |r| r
                        .get::<_, usize>(0))
                    .unwrap(),
                10001
            );
            let target = input.context.sha256().unwrap();
            let job = crate::start_reference_job(&write, &target).unwrap();
            write.commit().unwrap();
            let maintenance = owner
                .maintenance(
                    crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION,
                    "trait checkpoint pruning test",
                )
                .unwrap();
            crate::SqliteIndexWriter::new(&maintenance)
                .write_job(&input, &job, &target)
                .unwrap();
        }

        {
            let write = workspace
                .write(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap();
            write
                .connection()
                .execute(
                    "DELETE FROM trait_vectors WHERE trait_key='trait:agile'",
                    [],
                )
                .unwrap();
            let error = validate_traits(write.connection(), &input.context).unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("has no seed vector for configured search profile"),
                "{error}"
            );
        }
        {
            let write = workspace
                .write(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap();
            assert!(validate_traits(write.connection(), &input.context).is_ok());
            let plan = write
                .connection()
                .prepare(&format!("EXPLAIN QUERY PLAN {TRAIT_CHECKPOINT_SELECT}"))
                .unwrap()
                .query_map(params!["key", "model", "hash", 2, 768], |r| {
                    r.get::<_, String>(3)
                })
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            assert!(
                plan.iter()
                    .any(|step| step
                        .contains("sqlite_autoindex_embedding_checkpoints_1 (reuse_key=?)")),
                "{plan:?}"
            );

            assert_eq!(
                write
                    .connection()
                    .query_row("SELECT count(*) FROM embedding_checkpoints", [], |r| r
                        .get::<_, usize>(0))
                    .unwrap(),
                1
            );

            write
                .connection()
                .execute("DELETE FROM embedding_checkpoints", [])
                .unwrap();
            assert!(validate_traits(write.connection(), &input.context).is_err());
        }
        {
            let write = workspace
                .write(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap();
            write
                .connection()
                .execute("UPDATE embedding_checkpoints SET vector_hash='wrong'", [])
                .unwrap();
            assert!(validate_traits(write.connection(), &input.context).is_err());
        }
        {
            let write = workspace
                .write(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap();
            write
                .connection()
                .execute("UPDATE embedding_checkpoints SET model_identity='{}'", [])
                .unwrap();
            assert!(validate_traits(write.connection(), &input.context).is_err());
        }
        {
            let write = workspace
                .write(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap();
            vector[0] = 0.;
            vector[1] = 1.;
            let json = serde_json::to_string(&vector).unwrap();
            write
                .connection()
                .execute(
                    "UPDATE trait_vectors SET vector_json=?1,vector_hash=?2",
                    params![json, crate::codec::sha256(json.as_bytes())],
                )
                .unwrap();
            assert!(validate_traits(write.connection(), &input.context).is_err());
        }
    }
    #[test]
    fn trait_seed_profile_coverage_failure_retains_previous_generation() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("workspace.sqlite");
        let mut input = crate::profile_tests::both();
        let definition = definition(&input, "agile");
        let key = definition.key.clone();
        input.traits.push(definition);
        let text = "agile\n";
        let hash = crate::codec::sha256(text.as_bytes());
        let mut vector = vec![0.; 768];
        vector[0] = 1.;
        let chunk = TraitSeedChunk {
            input: text.into(),
            input_hash: hash.clone(),
            token_count: 2,
            address: atlas_domain::SourcePassageAddress::PlainSection {
                source_text_sha256: hash.clone(),
                selection_version: "trait-definition-centroid/v1".into(),
                section_ordinal: 0,
                label: Some("agile".into()),
                chunk_bytes: atlas_domain::SourceByteRange {
                    start: 0,
                    end: text.len(),
                },
            },
            vector: vector.clone(),
        };
        for model in &input.context.search_profiles {
            input.trait_vectors.push(TraitVectorInput {
                key: key.clone(),
                profile_id: model.profile_id.clone(),
                input_hash: crate::codec::sha256(
                    format!("trait-definition-centroid/v1:{hash}").as_bytes(),
                ),
                vector: vector.clone(),
                chunks: vec![chunk.clone()],
            });
        }
        crate::test_support::publish_input(&path, &input).unwrap();
        let before = crate::validate_artifact(&path).unwrap();
        let generation = || {
            let connection = rusqlite::Connection::open(&path).unwrap();
            connection
                .query_row(
                    "SELECT workspace_key,reference_revision FROM workspace WHERE id=1",
                    [],
                    |r| Ok((r.get::<_, String>(0)?, r.get::<_, u64>(1)?)),
                )
                .unwrap()
        };
        let before_generation = generation();
        input
            .trait_vectors
            .retain(|seed| seed.profile_id.as_str() == "bge-base");
        let error = crate::test_support::publish_input(&path, &input).unwrap_err();
        assert!(error.to_string().contains("trait `trait:agile` has no seed vector for configured search profile `embeddinggemma`"),"{error}");
        assert_eq!(generation(), before_generation);
        assert_eq!(crate::validate_artifact(&path).unwrap(), before);
        assert_eq!(
            rusqlite::Connection::open(&path)
                .unwrap()
                .query_row("SELECT count(*) FROM trait_vectors", [], |r| r
                    .get::<_, usize>(0))
                .unwrap(),
            2
        );
    }
    #[test]
    fn trait_seed_coverage_requires_each_configured_profile_but_allows_metadata_only() {
        let directory = tempfile::tempdir().unwrap();
        let mut configured = crate::foundation_tests::fixture(true);
        configured.traits.push(definition(&configured, "agile"));
        let error = crate::test_support::publish_input(
            directory.path().join("incomplete.sqlite"),
            &configured,
        )
        .unwrap_err();
        assert!(
            error.to_string().contains(
                "trait `trait:agile` has no seed vector for configured search profile `bge-base`"
            ),
            "{error}"
        );
        let mut metadata = crate::foundation_tests::fixture(false);
        metadata.traits.push(definition(&metadata, "agile"));
        let path = directory.path().join("metadata.sqlite");
        crate::test_support::publish_input(&path, &metadata).unwrap();
        let reader = crate::test_support::open_reader(&path).unwrap();
        assert!(
            reader
                .trait_definition(&TraitKey::new("agile").unwrap())
                .unwrap()
                .is_some()
        );
    }
    #[test]
    fn trait_reference_validation_rejects_missing_and_extra_occurrences() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("workspace.sqlite");
        let mut input = crate::foundation_tests::fixture(false);
        input.records[1] = crate::foundation_tests::record(
            "items",
            "Item",
            json!({"_id":"bbbbbbbbbbbbbbbb","name":"Dragon Staff","type":"weapon","system":{"description":{"value":"@Localize[PF2E.TraitAgile]"}}}),
        );
        input.traits.push(definition(&input, "agile"));
        refresh_fixture(&mut input);
        crate::test_support::publish_input(&path, &input).unwrap();
        let workspace = crate::test_support::initialize_workspace(&path).unwrap();
        {
            let write = workspace
                .write(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap();
            assert_eq!(
                write
                    .connection()
                    .execute("DELETE FROM trait_references", [])
                    .unwrap(),
                1
            );
            assert!(validate_traits(write.connection(), &input.context).is_err());
        }
        {
            let write = workspace
                .write(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap();
            write.connection().execute("INSERT INTO trait_references VALUES('trait:agile','items:bbbbbbbbbbbbbbbb','invented')",[]).unwrap();
            assert!(validate_traits(write.connection(), &input.context).is_err());
        }
    }
    #[test]
    fn trait_unknown_policy_cannot_be_relabelled_during_projection_upgrade() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("workspace.sqlite");
        let mut input = crate::foundation_tests::fixture(false);
        crate::test_support::publish_input(&path, &input).unwrap();
        input.context.trait_policy = "trait-definition-centroid/v99".into();
        input.context.query_projection_version = "authored-query/v2".into();
        let json = serde_json::to_string(&input.context).unwrap();
        let workspace = crate::test_support::initialize_workspace(&path).unwrap();
        let write = workspace
            .write(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap();
        write
            .connection()
            .execute(
                "UPDATE artifact_context SET context_json=?1,context_hash=?2",
                params![json, crate::codec::sha256(json.as_bytes())],
            )
            .unwrap();
        assert!(crate::classify_reference_upgrade(write.connection()).is_err());
    }
    #[test]
    fn trait_memberships_preserve_nameless_spell_and_nested_subitem_owners() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("workspace.sqlite");
        let mut input = crate::foundation_tests::fixture(false);
        input.records[1] = crate::foundation_tests::record(
            "items",
            "Item",
            json!({"_id":"bbbbbbbbbbbbbbbb","name":"Dragon Staff","type":"weapon","system":{"traits":{"value":[]},"subitems":[{"_id":"eeeeeeeeeeeeeeee","name":"Attached Tool","type":"equipment","system":{"traits":{"value":["custom-tool"]},"subitems":[{"_id":"ffffffffffffffff","type":"consumable","system":{"spell":{"_id":"gggggggggggggggg","type":"spell","system":{"traits":{"value":["custom-spell"]}}}}}]}}]}}),
        );
        refresh_fixture(&mut input);
        crate::test_support::publish_input(&path, &input).unwrap();
        let reader = crate::test_support::open_reader(&path).unwrap();
        let (members, total) = reader
            .trait_memberships(&TraitKey::new("custom-tool").unwrap(), 20, 0)
            .unwrap();
        assert_eq!(total, 1);
        assert_eq!(members[0].owners.len(), 1);
        assert_eq!(members[0].source_label.as_deref(), Some("Attached Tool"));
        let (members, total) = reader
            .trait_memberships(&TraitKey::new("custom-spell").unwrap(), 20, 0)
            .unwrap();
        assert_eq!(total, 1);
        assert_eq!(members[0].owners.len(), 3);
        assert!(members[0].source_label.is_none());
    }

    use super::*;
    use serde_json::json;
    #[test]
    fn source_free_definitions_and_exact_root_embedded_memberships() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("workspace.sqlite");
        let mut input = crate::foundation_tests::fixture(false);
        let key = TraitKey::new("dragon").unwrap();
        input.traits.push(TraitDefinition {
            key: key.clone(),
            label: "Dragon".into(),
            label_key: None,
            description_key: None,
            description_html: None,
            authored_markup: None,
            prepared: None,
            catalogs: vec!["creatureTraits".into()],
            source_locations: vec![],
            source_revision: input.context.source_revision.clone(),
            indexing_locale: input.context.indexing_locale.clone(),
        });
        input.records[0] = crate::foundation_tests::record(
            "actors",
            "Actor",
            json!({"_id":"aaaaaaaaaaaaaaaa","name":"Dragon Guard","type":"npc","system":{"details":{"level":{"value":9}},"traits":{"value":["dragon"]}},"items":[{"_id":"eeeeeeeeeeeeeeee","name":"Unknown Breath","type":"action","system":{"traits":{"value":["custom-unknown"]},"description":{"value":"Breath"}}}]}),
        );
        input.records[0].preparation_context_hash = input.context.preparation_sha256().unwrap();
        input.lexical_units.clear();
        for source in &input.records {
            let label = &input
                .packs
                .iter()
                .find(|p| p.pack_id == source.record.key().pack().as_str())
                .unwrap()
                .label;
            let selection = atlas_record::source_record::select_record_search(
                &source.record,
                &source.content,
                input.context.audience,
                label,
                &input.context.used_trait_labels,
            )
            .unwrap();
            input
                .lexical_units
                .extend(crate::project_lexical_units(&selection, &[]));
        }
        crate::test_support::publish_input(&path, &input).unwrap();
        let reader = crate::test_support::open_reader(&path).unwrap();
        assert_eq!(
            reader.trait_definition(&key).unwrap().unwrap().label,
            "Dragon"
        );
        let (root, total) = reader.trait_memberships(&key, 1, 0).unwrap();
        assert_eq!(total, 1);
        assert!(root[0].owners.is_empty());
        let unknown = TraitKey::new("custom-unknown").unwrap();
        assert!(reader.trait_definition(&unknown).unwrap().is_none());
        let (embedded, total) = reader.trait_memberships(&unknown, 1, 0).unwrap();
        assert_eq!(total, 1);
        assert_eq!(embedded[0].owners.len(), 1);
        assert_eq!(embedded[0].record.key, root[0].record.key);
        assert!(
            reader
                .trait_memberships(&unknown, 1, 1)
                .unwrap()
                .0
                .is_empty()
        );
        assert!(reader.trait_references(&key, 20, 0).unwrap().0.is_empty());
    }
    #[test]
    fn legacy_context_without_trait_policy_requires_source_preparation() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("workspace.sqlite");
        let input = crate::foundation_tests::fixture(false);
        crate::test_support::publish_input(&path, &input).unwrap();
        let workspace = crate::test_support::initialize_workspace(&path).unwrap();
        let write = workspace
            .write(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap();
        let mut json = serde_json::to_value(&input.context).unwrap();
        json.as_object_mut().unwrap().remove("trait_policy");
        let json = serde_json::to_string(&json).unwrap();
        write
            .connection()
            .execute(
                "UPDATE artifact_context SET context_json=?1,context_hash=?2",
                params![json, crate::codec::sha256(json.as_bytes())],
            )
            .unwrap();
        assert_eq!(
            crate::classify_reference_upgrade(write.connection()).unwrap(),
            crate::ReferenceUpgrade::SourcePreparation
        );
    }
}

#[derive(Debug, Clone)]
pub struct TraitReference {
    pub source_label: Option<String>,
    pub record: SourceRecordSummary,
    pub owners: Vec<OwnedContentLocator>,
    pub field: Option<String>,
    pub direction: String,
    pub occurrence_path: String,
    pub evidence: String,
}

struct TraitMembershipRow {
    key: String,
    record: RecordKey,
    owners: Vec<OwnedContentLocator>,
    source_label: Option<String>,
}
fn project_memberships(
    records: &[crate::SourceArtifactRecordInput],
    audience: atlas_record::source_content::ContentAudience,
) -> Result<Vec<TraitMembershipRow>, IndexError> {
    let mut result = std::collections::BTreeSet::new();
    for source in records {
        for membership in
            atlas_record::source_record::trait_record_memberships(&source.record, audience)
        {
            result.insert((
                membership.trait_key.to_string(),
                source.record.key().clone(),
                serde_json::to_string(&membership.owners)?,
                if membership.owners.is_empty() {
                    None
                } else {
                    membership.source_label
                },
            ));
        }
    }
    result
        .into_iter()
        .map(|(key, record, owners, label)| {
            Ok(TraitMembershipRow {
                key,
                record,
                owners: serde_json::from_str(&owners)?,
                source_label: label,
            })
        })
        .collect()
}
fn project_references(
    records: &[crate::SourceArtifactRecordInput],
    definitions: &[TraitDefinition],
    audience: atlas_record::source_content::ContentAudience,
) -> Vec<TraitReferenceInput> {
    use atlas_record::source_content::{ContentReferenceResolution, ContentReferenceTarget};
    let mut references = Vec::new();
    for record in records {
        for occurrence in atlas_record::source_record::trait_record_localization_references(
            &record.record,
            audience,
        ) {
            if occurrence.visible {
                references.push(TraitReferenceInput {
                    source_label: if occurrence.locator.owners.is_empty() {
                        None
                    } else {
                        record
                            .record
                            .node_at(&occurrence.locator.owners)
                            .and_then(|node| node.name().value().cloned())
                    },
                    key: occurrence.trait_key,
                    record: occurrence.locator.record,
                    owners: occurrence.locator.owners,
                    field: Some(occurrence.locator.field),
                    direction: "incoming".into(),
                    occurrence_path: occurrence.path,
                    evidence: occurrence.localization_key,
                });
            }
        }
    }
    for definition in definitions {
        if let Some(prepared) = &definition.prepared {
            for occurrence in &prepared.references {
                if !occurrence.visible {
                    continue;
                }
                let ContentReferenceResolution::Resolved(target) = &occurrence.resolution else {
                    continue;
                };
                let (record, owners) = match target {
                    ContentReferenceTarget::Record { key } => (key.clone(), vec![]),
                    ContentReferenceTarget::OwnedNode { key, owners } => {
                        (key.clone(), owners.clone())
                    }
                    ContentReferenceTarget::Url { .. } => continue,
                };
                references.push(TraitReferenceInput {
                    source_label: if owners.is_empty() {
                        None
                    } else {
                        occurrence.label.target_name.clone()
                    },
                    key: definition.key.clone(),
                    record,
                    owners,
                    field: None,
                    direction: "outgoing".into(),
                    occurrence_path: occurrence.path.clone(),
                    evidence: format!(
                        "{}:{}",
                        definition.description_key.as_deref().unwrap_or(""),
                        occurrence.authored_target
                    ),
                });
            }
        }
    }
    references
}

const TRAIT_CHECKPOINT_SELECT: &str = "SELECT vector,vector_hash FROM embedding_checkpoints WHERE reuse_key=?1 AND model_identity=?2 AND input_hash=?3 AND token_count=?4 AND dimensions=?5";

fn validate_seed(
    definition: &TraitDefinition,
    model: &crate::SourceSearchProfileIdentity,
    hash: &str,
    vector: &[f32],
    chunks: &[TraitSeedChunk],
) -> Result<(), IndexError> {
    let text = format!(
        "{}\n{}",
        definition.label,
        definition.prepared.as_ref().map_or("", |p| p.text.as_str())
    );
    let mut covered = 0;
    for chunk in chunks {
        let atlas_domain::SourcePassageAddress::PlainSection {
            source_text_sha256,
            selection_version,
            section_ordinal,
            label,
            chunk_bytes,
        } = &chunk.address
        else {
            return Err(IndexError::Invalid("trait seed address kind".into()));
        };
        let body = chunk_bytes
            .extract(&text)
            .map_err(|e| IndexError::Invalid(e.to_string()))?;
        // Context may be shortened under the shared budget, but the full body must survive.
        if source_text_sha256 != &crate::codec::sha256(text.as_bytes())
            || selection_version != "trait-definition-centroid/v1"
            || *section_ordinal != 0
            || label.as_deref() != Some(definition.label.as_str())
            || chunk_bytes.start > covered
            || chunk_bytes.end <= covered
            || !chunk.input.ends_with(body)
            || chunk.input_hash != crate::codec::sha256(chunk.input.as_bytes())
            || chunk.token_count == 0
            || chunk.token_count > model.token_limit
            || chunk.vector.len() != model.dimensions
        {
            return Err(IndexError::Invalid(
                "trait seed input or coverage mismatch".into(),
            ));
        }
        covered = chunk_bytes.end;
    }
    let expected_hash = crate::codec::sha256(
        format!(
            "trait-definition-centroid/v1:{}",
            chunks
                .iter()
                .map(|c| c.input_hash.as_str())
                .collect::<Vec<_>>()
                .join(":")
        )
        .as_bytes(),
    );
    crate::validation::check_vector(vector, model.dimensions)?;
    for chunk in chunks {
        crate::validation::check_vector(&chunk.vector, model.dimensions)?;
    }
    let expected = atlas_embedding::normalized_centroid(
        &chunks
            .iter()
            .map(|c| c.vector.as_slice())
            .collect::<Vec<_>>(),
    )
    .map_err(|e| IndexError::Invalid(e.to_string()))?;
    if covered != text.len() || expected_hash != hash || expected != vector {
        return Err(IndexError::Invalid(
            "trait seed aggregation or complete coverage mismatch".into(),
        ));
    }
    Ok(())
}
