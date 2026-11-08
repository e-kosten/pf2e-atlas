//! Persisted trait vocabulary and bounded, exact authored associations.
use crate::{IndexError, SqliteIndexReader};
use atlas_domain::OwnedContentLocator;
use atlas_domain::{RecordKey, SearchProfileId, SourceRecordSummary, TraitKey};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const TRAIT_SCHEMA_SQL: &str = "
CREATE TABLE trait_definitions(key TEXT PRIMARY KEY, definition_json TEXT NOT NULL CHECK(json_valid(definition_json)),definition_hash TEXT NOT NULL) STRICT;
CREATE TABLE trait_vectors(trait_key TEXT NOT NULL REFERENCES trait_definitions(key) ON DELETE CASCADE,profile_id TEXT NOT NULL,input_hash TEXT NOT NULL,vector_json TEXT NOT NULL CHECK(json_valid(vector_json)),chunks_json TEXT NOT NULL CHECK(json_valid(chunks_json)),vector_hash TEXT NOT NULL,PRIMARY KEY(trait_key,profile_id)) WITHOUT ROWID,STRICT;
CREATE TABLE trait_memberships(trait_key TEXT NOT NULL,record_key TEXT NOT NULL REFERENCES records(key) DEFERRABLE INITIALLY DEFERRED,owners_json TEXT NOT NULL CHECK(json_valid(owners_json)),source_label TEXT,PRIMARY KEY(trait_key,record_key,owners_json)) WITHOUT ROWID,STRICT;
CREATE TABLE trait_references(trait_key TEXT NOT NULL REFERENCES trait_definitions(key) ON DELETE CASCADE,record_key TEXT NOT NULL REFERENCES records(key) DEFERRABLE INITIALLY DEFERRED,evidence TEXT NOT NULL,PRIMARY KEY(trait_key,record_key,evidence)) WITHOUT ROWID,STRICT;
";
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraitDefinition {
    pub key: TraitKey,
    pub label: String,
    pub label_key: Option<String>,
    pub description_key: Option<String>,
    pub description_html: Option<String>,
    pub authored_markup: Option<String>,
    pub prepared: Option<atlas_record::source_content::PreparedReferenceDefinition>,
    pub catalogs: Vec<String>,
    pub source_locations: Vec<crate::ReferenceSourceLocation>,
    pub source_revision: Option<String>,
    pub indexing_locale: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_locale: Option<String>,
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

fn write_trait_definition(
    c: &Connection,
    definition: &TraitDefinition,
    kept: &mut BTreeSet<String>,
) -> Result<(), IndexError> {
    let key = definition.key.to_string();
    if !kept.insert(key.clone()) {
        return Err(IndexError::Invalid("duplicate trait definition".into()));
    }
    let entry = crate::ReferenceEntry::from_trait(definition)?;
    crate::references::write_reference_entry(c, &entry, None)?;
    let evidence = TraitEvidence::from_definitions(std::slice::from_ref(definition))?;
    let mut old=c.prepare("SELECT record_key,evidence FROM trait_references WHERE trait_key=? AND json_extract(evidence,'$.direction')='outgoing'")?.query_map([&key],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?)))?.collect::<Result<std::collections::BTreeSet<_>,_>>()?;
    for (trait_key, record, reference) in evidence.references {
        if !old.remove(&(record.clone(), reference.clone())) {
            c.execute(
                "INSERT INTO trait_references VALUES(?,?,?)",
                params![trait_key, record, reference],
            )?;
        }
    }
    for (record, reference) in old {
        c.execute(
            "DELETE FROM trait_references WHERE trait_key=? AND record_key=? AND evidence=?",
            params![key, record, reference],
        )?;
    }
    Ok(())
}
pub(crate) fn write_record_trait_relations(
    c: &Connection,
    record: &RecordKey,
    evidence: TraitEvidence,
) -> Result<(), IndexError> {
    let key = record.to_string();
    let current = c
        .prepare_cached("SELECT value,record_key,owners_json,lookup_member,source_label FROM trait_members WHERE record_key=?")?
        .query_map([&key], read_member)?
        .collect::<Result<BTreeSet<_>, _>>()?;
    for (value, record, owners, lookup_member, label) in evidence.memberships.difference(&current) {
        c.execute(
            "INSERT INTO trait_members VALUES(?,?,?,?,?)
             ON CONFLICT(value,record_key,owners_json) DO UPDATE SET
                 lookup_member=excluded.lookup_member,source_label=excluded.source_label",
            params![value, record, owners, lookup_member, label],
        )?;
    }
    for (value, record, owners, lookup_member, label) in current.difference(&evidence.memberships) {
        c.execute(
            "DELETE FROM trait_members WHERE value=? AND record_key=? AND owners_json=?
             AND lookup_member=? AND source_label IS ?",
            params![value, record, owners, lookup_member, label],
        )?;
    }
    write_record_trait_references(c, &key, evidence.references)
}

fn write_record_trait_references(
    c: &Connection,
    key: &str,
    desired: Vec<(String, String, String)>,
) -> Result<(), IndexError> {
    let mut references=c.prepare("SELECT trait_key,record_key,evidence FROM trait_references WHERE record_key=? AND json_extract(evidence,'$.direction')='incoming'")?.query_map([key],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?)))?.collect::<Result<std::collections::BTreeSet<_>,_>>()?;
    for row in desired {
        if !references.remove(&row) {
            c.execute(
                "INSERT INTO trait_references VALUES(?,?,?)",
                params![row.0, row.1, row.2],
            )?;
        }
    }
    for row in references {
        c.execute(
            "DELETE FROM trait_references WHERE trait_key=? AND record_key=? AND evidence=?",
            params![row.0, row.1, row.2],
        )?;
    }
    Ok(())
}
pub(crate) fn write_trait_definitions(
    c: &Connection,
    input: &crate::PublicationSpool,
) -> Result<BTreeSet<String>, IndexError> {
    let mut kept = BTreeSet::new();
    input.traits(&mut |definition| write_trait_definition(c, definition, &mut kept))?;
    Ok(kept)
}

pub(crate) fn prune_trait_definitions(
    c: &Connection,
    kept: &BTreeSet<String>,
) -> Result<(), IndexError> {
    let old = c
        .prepare("SELECT key FROM reference_entries WHERE key LIKE 'trait:%'")?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for key in old {
        if !kept.contains(&key) {
            c.execute("DELETE FROM reference_entries WHERE key=?", [key])?;
        }
    }
    Ok(())
}
/// Populate the genuine released trait layout for published-prefix fixtures.
/// The migration10 write_seed/validate_seed closure below is unchanged.
#[cfg(any(test, feature = "test-support"))]
pub(crate) fn write_historical_traits(
    c: &Connection,
    input: &crate::IndexBuildInput,
) -> Result<(), IndexError> {
    crate::test_support::fixture_native_values(input)?;
    let mut kept = BTreeSet::new();
    for definition in &input.traits {
        validate_definition(definition)?;
        let key = definition.key.to_string();
        if !kept.insert(key.clone()) {
            return Err(IndexError::Invalid(
                "duplicate historical trait definition".into(),
            ));
        }
        let json = serde_json::to_string(definition)?;
        c.execute("INSERT INTO trait_definitions VALUES(?,?,?) ON CONFLICT(key) DO UPDATE SET definition_json=excluded.definition_json,definition_hash=excluded.definition_hash",params![key,json,crate::codec::sha256(json.as_bytes())])?;
    }
    let evidence = TraitEvidence::from_definitions(&input.traits)?;
    for (key, record, reference) in evidence.references {
        c.execute(
            "INSERT INTO trait_references VALUES(?,?,?)",
            params![key, record, reference],
        )?;
    }
    for source in &input.records {
        let mut evidence = TraitEvidence::default();
        evidence.add_record(&source.record, input.context.audience)?;
        write_historical_record_traits(c, source.record.key(), evidence)?;
    }
    c.execute_batch("DELETE FROM trait_seed_chunks; DELETE FROM trait_vectors; DELETE FROM trait_native_values;")?;
    for seed in &input.trait_vectors {
        let model = input.context.search_profile(&seed.profile_id)?;
        let definition = input
            .traits
            .iter()
            .find(|d| d.key == seed.key)
            .ok_or_else(|| IndexError::Invalid("historical trait seed has no definition".into()))?;
        validate_seed(
            definition,
            model,
            &seed.input_hash,
            &seed.vector,
            &seed.chunks,
        )?;
        write_seed(c, seed, model)?;
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

#[cfg(any(test, feature = "test-support"))]
pub(crate) fn write_historical_record_traits(
    c: &Connection,
    record_key: &RecordKey,
    evidence: TraitEvidence,
) -> Result<(), IndexError> {
    for (value, record, owners, admitted, label) in evidence.memberships {
        if admitted {
            c.execute(
                "INSERT OR REPLACE INTO trait_memberships VALUES(?,?,?,?)",
                params![format!("trait:{value}"), record, owners, label],
            )?;
        }
    }
    write_record_trait_references(c, &record_key.to_string(), evidence.references)
}

impl SqliteIndexReader {
    pub fn trait_definition(&self, key: &TraitKey) -> Result<Option<TraitDefinition>, IndexError> {
        crate::references::read_entry(
            self.native()?,
            &atlas_domain::DefinitionKey::from(key.clone()),
        )?
        .map(|entry| entry.trait_view())
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
        let sql = "SELECT record_key,owners_json AS owners,source_label FROM trait_members WHERE value=?1 AND lookup_member=1";
        let total = self.native()?.query_row(
            &format!("SELECT count(*) FROM ({sql})"),
            [key.identifier()],
            |r| r.get(0),
        )?;
        let mut stmt = self.native()?.prepare(&format!(
            "SELECT record_key,owners,source_label FROM ({sql}) ORDER BY record_key,owners LIMIT ?2 OFFSET ?3"
        ))?;
        let rows = stmt
            .query_map(params![key.identifier(), limit, offset], |r| {
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
        "INSERT INTO trait_vectors VALUES(?,?,?,?,?)",
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
            "INSERT INTO trait_seed_chunks VALUES(?,?,?,?,?,?)",
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
            c.execute("INSERT INTO trait_native_values VALUES(?,?) ON CONFLICT(input_id) DO UPDATE SET vector=excluded.vector",params![id,bytes])?;
        }
    }
    Ok(())
}

#[cfg(any(test, feature = "test-support"))]
fn validate_definition(definition: &TraitDefinition) -> Result<(), IndexError> {
    match (
        &definition.prepared,
        &definition.authored_markup,
        &definition.description_html,
    ) {
        (None, None, None) => {}
        (Some(prepared), Some(markup), Some(html)) => {
            if prepared.key != atlas_domain::DefinitionKey::from(definition.key.clone())
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
            atlas_record::source_content::validate_prepared_definition_reference_bindings(
                html,
                &prepared.key,
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
type Member = (String, String, String, bool, Option<String>);

fn read_member(row: &rusqlite::Row<'_>) -> rusqlite::Result<Member> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
    ))
}

#[derive(Default, Serialize)]
pub(crate) struct TraitEvidence {
    memberships: BTreeSet<Member>,
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
    pub(crate) fn add_published_definition(
        &mut self,
        entry: &crate::ReferenceEntry,
        context: &crate::SourceArtifactBuildContext,
    ) -> Result<(), IndexError> {
        let definition = entry.trait_view()?;
        if definition.indexing_locale != context.indexing_locale
            || definition.source_revision != context.source_revision
        {
            return Err(IndexError::Invalid(
                "trait publication provenance mismatch".into(),
            ));
        }
        self.add_definition(&definition)
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
                membership.identifier,
                record.key().to_string(),
                serde_json::to_string(&membership.owners)?,
                membership.lookup_member,
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
            .prepare(
                "SELECT value,record_key,owners_json,lookup_member,source_label FROM trait_members",
            )?
            .query_map([], read_member)?
            .collect::<Result<BTreeSet<_>, _>>()?;
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
            content_locale: None,
        }
    }
    #[test]
    fn removed_roots_prune_trait_edges_and_reappear_without_touching_live_associations() {
        use atlas_record::source_content::prepare_reference_definition;
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
            let prepared = prepare_reference_definition(
                dragon.key.clone().into(),
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
        for table in ["trait_members", "trait_references"] {
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
        for table in ["trait_members", "trait_references"] {
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
                "SELECT count(*) FROM trait_members WHERE record_key='items:cccccccccccccccc'",
                [],
                |r| r.get::<_, usize>(0)
            )
            .unwrap(),
            1
        );
        assert_eq!(
            c.query_row(
                "SELECT count(*) FROM reference_entries WHERE key='trait:dragon'",
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
            "SELECT value,record_key,owners_json,source_label FROM trait_members WHERE record_key=?",
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
            ContentReferenceTarget, ResolvedContentReference, prepare_reference_definition,
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
        let prepared = prepare_reference_definition(
            definition.key.clone().into(),
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
            content_locale: None,
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
