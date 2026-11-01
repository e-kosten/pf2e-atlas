//! Persisted trait vocabulary and bounded, exact authored associations.
use crate::{IndexError, SqliteIndexReader};
use atlas_domain::OwnedContentLocator;
use atlas_domain::{RecordKey, SearchProfileId, SourceRecordSummary, TraitKey};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

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

pub(crate) fn write_traits_stream(
    c: &Connection,
    input: &dyn crate::PublicationSource,
    values: &dyn crate::NativeValueSource,
) -> Result<(), IndexError> {
    let mut kept = std::collections::BTreeSet::new();
    input.traits(&mut |definition|{
        if definition.label.trim().is_empty(){return Err(IndexError::Invalid("empty trait label".into()));}
        validate_definition(definition)?;
        let key=definition.key.to_string();if !kept.insert(key.clone()){return Err(IndexError::Invalid("duplicate trait definition".into()));}
        let json=serde_json::to_string(definition)?;let hash=crate::codec::sha256(json.as_bytes());
        c.execute("INSERT INTO trait_definitions VALUES(?,?,?) ON CONFLICT(key) DO UPDATE SET definition_json=excluded.definition_json,definition_hash=excluded.definition_hash WHERE definition_hash IS NOT excluded.definition_hash",params![key,json,hash])?;
        let evidence=TraitEvidence::from_definitions(std::slice::from_ref(definition))?;
        let mut old=c.prepare("SELECT record_key,evidence FROM trait_references WHERE trait_key=? AND json_extract(evidence,'$.direction')='outgoing'")?.query_map([&key],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?)))?.collect::<Result<std::collections::BTreeSet<_>,_>>()?;
        for (trait_key,record,reference) in evidence.references {if !old.remove(&(record.clone(),reference.clone())){c.execute("INSERT INTO trait_references VALUES(?,?,?)",params![trait_key,record,reference])?;}}
        for (record,reference) in old{c.execute("DELETE FROM trait_references WHERE trait_key=? AND record_key=? AND evidence=?",params![key,record,reference])?;}
        Ok(())
    })?;
    input.records(&mut |frame|{
        let key=frame.source.record.key().to_string();
        let mut evidence=TraitEvidence::default();evidence.add_record(&frame.source.record,input.context().audience)?;
        let mut memberships=c.prepare("SELECT trait_key,record_key,owners_json,source_label FROM trait_memberships WHERE record_key=?")?.query_map([&key],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,Option<String>>(3)?)))?.collect::<Result<std::collections::BTreeSet<_>,_>>()?;
        for row in evidence.memberships{
            let prior=memberships.iter().find(|old|old.0==row.0 && old.1==row.1 && old.2==row.2).cloned();
            if let Some(old)=&prior{memberships.remove(old);}
            if prior.as_ref()!=Some(&row){c.execute("INSERT INTO trait_memberships VALUES(?,?,?,?) ON CONFLICT(trait_key,record_key,owners_json) DO UPDATE SET source_label=excluded.source_label WHERE source_label IS NOT excluded.source_label",params![row.0,row.1,row.2,row.3])?;}
        }
        for row in memberships{c.execute("DELETE FROM trait_memberships WHERE trait_key=? AND record_key=? AND owners_json=?",params![row.0,row.1,row.2])?;}
        let mut references=c.prepare("SELECT trait_key,record_key,evidence FROM trait_references WHERE record_key=? AND json_extract(evidence,'$.direction')='incoming'")?.query_map([&key],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?)))?.collect::<Result<std::collections::BTreeSet<_>,_>>()?;
        for row in evidence.references{if !references.remove(&row){c.execute("INSERT INTO trait_references VALUES(?,?,?)",params![row.0,row.1,row.2])?;}}
        for row in references{c.execute("DELETE FROM trait_references WHERE trait_key=? AND record_key=? AND evidence=?",params![row.0,row.1,row.2])?;}
        Ok(())
    })?;
    let mut seeds = std::collections::BTreeSet::new();
    input.trait_seeds(&mut |seed| {
        let model = input.context().search_profile(&seed.profile_id)?;
        if !seeds.insert((seed.key.to_string(), seed.profile_id.to_string())) {
            return Err(IndexError::Invalid("duplicate trait seed".into()));
        }
        let chunks = seed
            .chunks
            .iter()
            .map(|chunk| {
                Ok(TraitSeedChunk {
                    input: chunk.input.clone(),
                    input_hash: chunk.descriptor.input_sha256.clone(),
                    token_count: chunk.descriptor.token_count,
                    address: chunk.address.clone(),
                    vector: values.value(model, &chunk.descriptor)?.into_owned(),
                })
            })
            .collect::<Result<Vec<_>, IndexError>>()?;
        let vector = atlas_embedding::normalized_centroid(
            &chunks
                .iter()
                .map(|c| c.vector.as_slice())
                .collect::<Vec<_>>(),
        )
        .map_err(|e| IndexError::Invalid(e.to_string()))?;
        write_seed(
            c,
            &TraitVectorInput {
                key: seed.key.clone(),
                profile_id: seed.profile_id.clone(),
                input_hash: seed.input_hash.clone(),
                vector,
                chunks,
            },
            model,
        )
    })?;
    let old = c
        .prepare("SELECT trait_key,profile_id FROM trait_vectors")?
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    for row in old {
        if !seeds.contains(&row) {
            c.execute(
                "DELETE FROM trait_vectors WHERE trait_key=? AND profile_id=?",
                params![row.0, row.1],
            )?;
        }
    }
    let old = c
        .prepare("SELECT key FROM trait_definitions")?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for key in old {
        if !kept.contains(&key) {
            c.execute("DELETE FROM trait_definitions WHERE key=?", [key])?;
        }
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
    /// Complete distinct root eligibility, independent of occurrence preview limits.
    pub fn trait_reference_record_keys(
        &self,
        key: &TraitKey,
    ) -> Result<Vec<RecordKey>, IndexError> {
        let mut statement = self.native()?.prepare(
            "SELECT DISTINCT record_key FROM trait_references WHERE trait_key=?1 ORDER BY record_key",
        )?;
        let keys = statement
            .query_map([key.to_string()], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        keys.into_iter()
            .map(|key| {
                key.parse().map_err(|error| {
                    IndexError::Invalid(format!("invalid trait reference key: {error}"))
                })
            })
            .collect()
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
        let row: Option<(Vec<u8>, Vec<u8>)> = self
            .native()?
            .query_row(
                "SELECT vector,vector_hash FROM trait_vectors WHERE trait_key=? AND profile_id=?",
                params![key.to_string(), profile.as_str()],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        row.map(|(bytes, hash)| {
            if crate::embedding_values::hash_bytes(&bytes) != hash {
                return Err(IndexError::Invalid("trait vector checksum mismatch".into()));
            }
            let vector = crate::read_units::decode_vector(&bytes)?;
            crate::validation::check_vector(
                &vector,
                self.context().search_profile(profile)?.dimensions,
            )?;
            Ok(vector)
        })
        .transpose()
    }
}

pub(crate) fn write_seed(
    c: &Connection,
    unit: &TraitVectorInput,
    model: &crate::SourceSearchProfileIdentity,
) -> Result<(), IndexError> {
    crate::validation::check_vector(&unit.vector, model.dimensions)?;
    if !crate::codec::checked_hash(&unit.input_hash) {
        return Err(IndexError::Invalid("invalid trait input hash".into()));
    }
    let bytes = crate::read_units::vector_bytes(&unit.vector);
    c.execute(
        "INSERT INTO trait_vectors VALUES(?,?,?,?,?) ON CONFLICT(trait_key,profile_id) DO UPDATE SET input_hash=excluded.input_hash,vector=excluded.vector,vector_hash=excluded.vector_hash WHERE input_hash IS NOT excluded.input_hash OR vector_hash IS NOT excluded.vector_hash",
        params![
            unit.key.to_string(),
            unit.profile_id.as_str(),
            unit.input_hash,
            bytes,
            crate::embedding_values::hash_bytes(&bytes)
        ],
    )?;
    for (ordinal, chunk) in unit.chunks.iter().enumerate() {
        let id = crate::embedding_values::publish_input(
            c,
            model,
            &chunk.input_hash,
            chunk.token_count,
            &chunk.vector,
        )?;
        c.execute(
            "INSERT INTO trait_seed_chunks VALUES(?,?,?,?,?,?) ON CONFLICT(trait_key,profile_id,ordinal) DO UPDATE SET input_id=excluded.input_id,input=excluded.input,address_json=excluded.address_json WHERE input_id IS NOT excluded.input_id OR input IS NOT excluded.input OR address_json IS NOT excluded.address_json",
            params![
                unit.key.to_string(),
                unit.profile_id.as_str(),
                ordinal,
                id,
                chunk.input,
                serde_json::to_string(&chunk.address)?
            ],
        )?;
        // A document occurrence is canonical when present. Otherwise keep one
        // native binary value, regardless of how many traits share this input.
        let document: bool = c.query_row(
            "SELECT EXISTS(SELECT 1 FROM semantic_units WHERE input_id=?)",
            [id],
            |r| r.get(0),
        )?;
        if !document {
            let bytes = crate::read_units::vector_bytes(&chunk.vector);
            c.execute("INSERT INTO trait_native_values VALUES(?,?) ON CONFLICT(input_id) DO UPDATE SET vector=excluded.vector WHERE vector IS NOT excluded.vector",params![id,bytes])?;
        }
    }
    c.execute(
        "DELETE FROM trait_seed_chunks WHERE trait_key=? AND profile_id=? AND ordinal>=?",
        params![
            unit.key.to_string(),
            unit.profile_id.as_str(),
            unit.chunks.len()
        ],
    )?;
    Ok(())
}
pub(crate) fn read_chunks(
    c: &Connection,
    key: &str,
    model: &crate::SourceSearchProfileIdentity,
) -> Result<Vec<TraitSeedChunk>, IndexError> {
    let mut s=c.prepare("SELECT t.ordinal,t.input_id,t.input,t.address_json,lower(hex(i.input_hash)),i.token_count,m.identity_json FROM trait_seed_chunks t JOIN embedding_inputs i USING(input_id) JOIN embedding_models m USING(model_id) WHERE trait_key=? AND profile_id=? ORDER BY ordinal")?;
    let mut rows = s.query(params![key, model.profile_id.as_str()])?;
    let mut chunks = Vec::new();
    while let Some(row) = rows.next()? {
        let ordinal: usize = row.get(0)?;
        let stored: String = row.get(6)?;
        if ordinal != chunks.len() || stored != serde_json::to_string(model)? {
            return Err(IndexError::Invalid(
                "trait chunk model or ordinal mismatch".into(),
            ));
        }
        chunks.push(TraitSeedChunk {
            input: row.get(2)?,
            input_hash: row.get(4)?,
            token_count: row.get(5)?,
            address: serde_json::from_str(&row.get::<_, String>(3)?)?,
            vector: crate::embedding_values::native_value(c, row.get(1)?, model)?,
        });
    }
    Ok(chunks)
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
pub(crate) fn validate_trait_definitions(
    c: &Connection,
    context: &crate::SourceArtifactBuildContext,
) -> Result<TraitEvidence, IndexError> {
    // Every published definition, including label-only definitions, supplies a
    // seed for every configured profile. An artifact with no profiles remains
    // valid metadata-only reference data.
    let missing:Option<(String,String)>=c.query_row("SELECT d.key,p.profile_id FROM trait_definitions d CROSS JOIN search_profiles p WHERE NOT EXISTS (SELECT 1 FROM trait_vectors v WHERE v.trait_key=d.key AND v.profile_id=p.profile_id) LIMIT 1",[],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
    if let Some((key, profile)) = missing {
        return Err(IndexError::Invalid(format!(
            "trait `{key}` has no seed vector for configured search profile `{profile}`"
        )));
    }
    let mut evidence = TraitEvidence::default();
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
        evidence.add_definition(&definition)?;
    }
    let mut stmt=c.prepare("SELECT tv.trait_key,tv.profile_id,tv.input_hash,tv.vector,tv.vector_hash,d.definition_json FROM trait_vectors tv JOIN trait_definitions d ON d.key=tv.trait_key")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let key: String = row.get(0)?;
        let profile: String = row.get(1)?;
        let hash: String = row.get(2)?;
        let bytes: Vec<u8> = row.get(3)?;
        let checksum: Vec<u8> = row.get(4)?;
        if crate::embedding_values::hash_bytes(&bytes) != checksum {
            return Err(IndexError::Invalid("trait vector checksum mismatch".into()));
        }
        let model = context.search_profile(&SearchProfileId::new(profile))?;
        let definition: TraitDefinition = serde_json::from_str(&row.get::<_, String>(5)?)?;
        let vector = crate::read_units::decode_vector(&bytes)?;
        let chunks = read_chunks(c, &key, model)?;
        validate_seed(&definition, model, &hash, &vector, &chunks)?;
    }
    Ok(evidence)
}

#[derive(Default, Serialize)]
pub(crate) struct TraitEvidence {
    memberships: BTreeSet<(String, String, String, Option<String>)>,
    references: Vec<(String, String, String)>,
}

impl TraitEvidence {
    fn from_definitions(definitions: &[TraitDefinition]) -> Result<Self, IndexError> {
        let mut evidence = Self::default();
        for definition in definitions {
            evidence.add_definition(definition)?;
        }
        Ok(evidence)
    }
    fn add_definition(&mut self, definition: &TraitDefinition) -> Result<(), IndexError> {
        use atlas_record::source_content::{ContentReferenceResolution, ContentReferenceTarget};
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
                self.add_reference(TraitReferenceInput {
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
                })?;
            }
        }
        Ok(())
    }

    pub(crate) fn add_record(
        &mut self,
        record: &atlas_record::source_record::SourceBackedRecord,
        audience: atlas_record::source_content::ContentAudience,
    ) -> Result<(), IndexError> {
        for membership in atlas_record::source_record::trait_record_memberships(record, audience) {
            self.memberships.insert((
                membership.trait_key.to_string(),
                record.key().to_string(),
                serde_json::to_string(&membership.owners)?,
                if membership.owners.is_empty() {
                    None
                } else {
                    membership.source_label
                },
            ));
        }
        for occurrence in
            atlas_record::source_record::trait_record_localization_references(record, audience)
        {
            if occurrence.visible {
                self.add_reference(TraitReferenceInput {
                    source_label: if occurrence.locator.owners.is_empty() {
                        None
                    } else {
                        record
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
                })?;
            }
        }
        Ok(())
    }

    fn add_reference(&mut self, reference: TraitReferenceInput) -> Result<(), IndexError> {
        self.references.push((
            reference.key.to_string(),
            reference.record.to_string(),
            serde_json::to_string(&reference)?,
        ));
        Ok(())
    }
    pub(crate) fn validate(self, c: &Connection) -> Result<(), IndexError> {
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
        if actual != self.references.into_iter().collect::<BTreeSet<_>>() {
            return Err(IndexError::Invalid(
                "trait reference evidence differs from checked source".into(),
            ));
        }
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
        if actual != self.memberships {
            return Err(IndexError::Invalid(
                "trait memberships differ from checked source".into(),
            ));
        }
        Ok(())
    }
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
    fn removed_roots_prune_trait_edges_and_reappear_without_touching_live_associations() {
        use atlas_record::source_content::prepare_trait_definition;
        use atlas_record::source_record::SourceReferenceIndex;
        fn input(keep_removed: bool) -> crate::IndexBuildInput {
            let mut input = crate::foundation_tests::fixture(false);
            for (index, id, name) in [
                (1, "bbbbbbbbbbbbbbbb", "Removed staff"),
                (2, "cccccccccccccccc", "Kept spell"),
            ] {
                input.records[index] = crate::foundation_tests::record(
                    "items",
                    "Item",
                    json!({"_id":id,"name":name,"type":"spell","system":{"traits":{"value":["fire"]},"description":{"value":"<p>@Localize[PF2E.TraitDragon]</p>"}}}),
                );
            }
            if !keep_removed {
                input.records.retain(|r| {
                    !matches!(
                        r.record.key().to_string().as_str(),
                        "actors:aaaaaaaaaaaaaaaa" | "items:bbbbbbbbbbbbbbbb"
                    )
                });
            }
            let mut resolver = SourceReferenceIndex::default();
            for source in &input.records {
                resolver.insert_source(source.record.key(), source.record.source());
            }
            let mut dragon = definition(&input, "dragon");
            let resolve = |kind: &atlas_record::source_content::ContentReferenceKind,
                           target: &str| {
                resolver.resolve_global_content_reference(kind, target)
            };
            let markup = "<p>@UUID[Compendium.pf2e.actors.Actor.aaaaaaaaaaaaaaaa]{Owner}</p>";
            let prepared = prepare_trait_definition(
                dragon.key.clone(),
                markup,
                input.context.audience,
                None,
                Some(&resolve),
            )
            .unwrap();
            dragon.authored_markup = Some(markup.into());
            dragon.description_html = Some(prepared.html.clone());
            dragon.description_key = Some("fixture.dragon.description".into());
            dragon.prepared = Some(prepared);
            input.traits.push(dragon);
            refresh_fixture(&mut input);
            input
        }
        let initial = input(true);
        let (_directory, path) = crate::foundation_tests::build(&initial);
        let c = Connection::open(&path).unwrap();
        c.execute_batch("INSERT INTO saved_lists(id,list_key,slug,name,created_at,updated_at) VALUES(1,'kept','kept','Saved trait owner','now','now'); INSERT INTO saved_list_items(list_id,record_key,position,note,record_title_snapshot,added_at,updated_at) VALUES(1,'actors:aaaaaaaaaaaaaaaa',0,'Keep removed-owner note','Dragon Guard','now','now'); CREATE TABLE live_trait_write_audit(component TEXT NOT NULL);").unwrap();
        for table in ["trait_memberships", "trait_references"] {
            for (event, row) in [("INSERT", "new"), ("UPDATE", "new"), ("DELETE", "old")] {
                c.execute_batch(&format!("CREATE TRIGGER live_{table}_{event} AFTER {event} ON {table} WHEN {row}.record_key='items:cccccccccccccccc' BEGIN INSERT INTO live_trait_write_audit VALUES('{table}'); END;")).unwrap();
            }
        }
        let identity: i64 = c
            .query_row(
                "SELECT id FROM record_identity WHERE key='actors:aaaaaaaaaaaaaaaa'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let revision: u64 = c
            .query_row("SELECT reference_revision FROM workspace", [], |r| r.get(0))
            .unwrap();
        assert_eq!(c.query_row("SELECT count(*) FROM trait_references WHERE record_key IN ('actors:aaaaaaaaaaaaaaaa','items:bbbbbbbbbbbbbbbb')",[],|r|r.get::<_,usize>(0)).unwrap(),2,"fixture needs outgoing and incoming removed-root edges");
        let reduced = input(false);
        crate::test_support::publish_input(&path, &reduced).unwrap();
        for table in ["trait_memberships", "trait_references"] {
            assert_eq!(c.query_row(&format!("SELECT count(*) FROM {table} WHERE record_key IN ('actors:aaaaaaaaaaaaaaaa','items:bbbbbbbbbbbbbbbb')"),[],|r|r.get::<_,usize>(0)).unwrap(),0,"{table}");
        }
        assert_eq!(
            c.query_row(
                "SELECT availability FROM record_identity WHERE id=?",
                [identity],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
            "unavailable"
        );
        assert_eq!(
            c.query_row(
                "SELECT count(*) FROM trait_references WHERE record_key='items:cccccccccccccccc'",
                [],
                |r| r.get::<_, usize>(0)
            )
            .unwrap(),
            1
        );
        assert_eq!(
            c.query_row(
                "SELECT count(*) FROM trait_memberships WHERE record_key='items:cccccccccccccccc'",
                [],
                |r| r.get::<_, usize>(0)
            )
            .unwrap(),
            1
        );
        assert_eq!(
            c.query_row(
                "SELECT count(*) FROM trait_definitions WHERE key='trait:dragon'",
                [],
                |r| r.get::<_, usize>(0)
            )
            .unwrap(),
            1
        );
        crate::test_support::publish_input(&path, &initial).unwrap();
        assert_eq!(
            c.query_row(
                "SELECT id FROM record_identity WHERE key='actors:aaaaaaaaaaaaaaaa'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            identity
        );
        assert_eq!(
            c.query_row(
                "SELECT availability FROM record_identity WHERE id=?",
                [identity],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
            "available"
        );
        assert_eq!(c.query_row("SELECT count(*) FROM trait_references WHERE record_key IN ('actors:aaaaaaaaaaaaaaaa','items:bbbbbbbbbbbbbbbb')",[],|r|r.get::<_,usize>(0)).unwrap(),2);
        assert_eq!(
            c.query_row(
                "SELECT note FROM saved_list_items WHERE list_id=1",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
            "Keep removed-owner note"
        );
        assert_eq!(
            c.query_row("SELECT reference_revision FROM workspace", [], |r| r
                .get::<_, u64>(0))
                .unwrap(),
            revision + 2
        );
        assert_eq!(
            c.query_row("SELECT count(*) FROM live_trait_write_audit", [], |r| r
                .get::<_, usize>(
                0
            ))
            .unwrap(),
            0,
            "live associations were rewritten"
        );
        for sql in [
            "SELECT trait_key,record_key,owners_json,source_label FROM trait_memberships WHERE record_key=?",
            "SELECT trait_key,record_key,evidence FROM trait_references WHERE record_key=? AND json_extract(evidence,'$.direction')='incoming'",
        ] {
            let plan = c
                .prepare(&format!("EXPLAIN QUERY PLAN {sql}"))
                .unwrap()
                .query_map(["items:cccccccccccccccc"], |r| r.get::<_, String>(3))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            assert!(
                plan.iter()
                    .any(|step| step.contains("SEARCH") && step.contains("record_key=?")),
                "root-key lookup must use a useful index: {plan:?}"
            );
        }
        crate::validate_artifact(&path).unwrap();
    }
    #[test]
    fn duplicate_trait_definition_is_rejected_without_selecting_a_winner() {
        let mut input = crate::foundation_tests::fixture(false);
        let definition = definition(&input, "agile");
        input.traits.push(definition.clone());
        input.traits.push(definition);
        let directory = tempfile::tempdir().unwrap();
        let error =
            crate::test_support::publish_input(directory.path().join("duplicate.sqlite"), &input)
                .unwrap_err();
        assert!(
            error.to_string().contains("duplicate trait definition"),
            "{error}"
        );
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
    fn trait_vector_publication_shares_native_values_and_validates_corruption() {
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
        // Multiple traits share one exact native value, without a duplicate cache.
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
            assert_eq!(
                write
                    .connection()
                    .query_row("SELECT count(*) FROM pending_embedding_values", [], |r| {
                        r.get::<_, usize>(0)
                    })
                    .unwrap(),
                0
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
                .write_job(&input, &input, &job, &target)
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
            let error = crate::validate_reference(write.connection()).unwrap_err();
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
            assert!(crate::validate_artifact(&path).is_ok());
            assert_eq!(
                write
                    .connection()
                    .query_row("SELECT count(*) FROM trait_native_values", [], |r| r
                        .get::<_, usize>(0))
                    .unwrap(),
                1
            );
            write
                .connection()
                .execute("DELETE FROM trait_native_values", [])
                .unwrap();
            assert!(crate::validate_reference(write.connection()).is_err());
        }
        {
            let write = workspace
                .write(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap();
            write
                .connection()
                .execute(
                    "UPDATE embedding_inputs SET published_hash=zeroblob(32)",
                    [],
                )
                .unwrap();
            assert!(crate::validate_reference(write.connection()).is_err());
        }
        {
            let write = workspace
                .write(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap();
            write
                .connection()
                .execute("UPDATE embedding_models SET identity_json='{}'", [])
                .unwrap();
            assert!(crate::validate_reference(write.connection()).is_err());
        }
        {
            let write = workspace
                .write(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap();
            vector[0] = 0.;
            vector[1] = 1.;
            let bytes = crate::read_units::vector_bytes(&vector);
            write
                .connection()
                .execute(
                    "UPDATE trait_vectors SET vector=?1,vector_hash=?2",
                    params![bytes, crate::embedding_values::hash_bytes(&bytes)],
                )
                .unwrap();
            assert!(crate::validate_reference(write.connection()).is_err());
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
            assert!(crate::validate_reference(write.connection()).is_err());
        }
        {
            let write = workspace
                .write(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap();
            write.connection().execute("INSERT INTO trait_references VALUES('trait:agile','items:bbbbbbbbbbbbbbbb','invented')",[]).unwrap();
            assert!(crate::validate_reference(write.connection()).is_err());
        }
        {
            let write = workspace
                .write(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap();
            assert_eq!(
                write
                    .connection()
                    .execute("UPDATE trait_references SET evidence=' ' || evidence", [],)
                    .unwrap(),
                1
            );
            let error = crate::validate_reference(write.connection()).unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("trait reference evidence differs"),
                "{error}"
            );
        }
    }
    #[test]
    fn definition_origin_references_are_validated_with_record_evidence() {
        use atlas_record::source_content::{
            ContentReferenceTarget, ResolvedContentReference, prepare_trait_definition,
        };
        let mut input = crate::foundation_tests::fixture(false);
        let mut definition = definition(&input, "agile");
        let target = input.records[1].record.key().clone();
        let resolve = |_: &atlas_record::source_content::ContentReferenceKind, _: &str| {
            Some(ResolvedContentReference {
                target: ContentReferenceTarget::Record {
                    key: target.clone(),
                },
                display_name: Some("Dragon Staff".into()),
                item_family: Some("weapon".into()),
            })
        };
        let markup = "<p>@UUID[Compendium.pf2e.items.Item.bbbbbbbbbbbbbbbb]{Dragon Staff}</p>";
        let prepared = prepare_trait_definition(
            definition.key.clone(),
            markup,
            input.context.audience,
            None,
            Some(&resolve),
        )
        .unwrap();
        definition.authored_markup = Some(markup.into());
        definition.description_html = Some(prepared.html.clone());
        definition.prepared = Some(prepared);
        input.traits.push(definition);
        let (_directory, path) = crate::foundation_tests::build(&input);
        let connection = Connection::open(&path).unwrap();
        let (record, direction): (String, String) = connection
            .query_row(
                "SELECT record_key,json_extract(evidence,'$.direction') FROM trait_references",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(record, "items:bbbbbbbbbbbbbbbb");
        assert_eq!(direction, "outgoing");
        crate::validate_artifact(&path).unwrap();
        connection
            .execute("DELETE FROM trait_references", [])
            .unwrap();
        let error = crate::validate_reference(&connection).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("trait reference evidence differs"),
            "{error}"
        );
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

pub(crate) fn validate_seed(
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
