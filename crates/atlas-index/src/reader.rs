//! Read-only artifact handles. Summary queries never decode source bodies.
use crate::{
    ArtifactInstance, IndexError, SourceArtifactBuildContext, codec::gunzip, numeric::SourceNumber,
};
use atlas_domain::{
    RecordKey, RecordKind, SourceLevelBasis, SourceQueryFact, SourceRecordSummary,
    query::QueryFieldState,
};
use atlas_foundry_model::{SNAPSHOT_VERSION, decode_snapshot};
use atlas_record::source_record::SourceBackedRecord;
use rusqlite::{Connection as NativeConnection, OptionalExtension};
use std::path::Path;

pub struct SqliteIndexReader {
    context: SourceArtifactBuildContext,
    context_hash: String,
    instance: ArtifactInstance,
    session: atlas_storage::ReadSession,
    vector_error: Option<String>,
    #[cfg(any(test, feature = "test-support"))]
    read_metrics: std::cell::Cell<SourceReadMetrics>,
}
/// Test-only counts of executed read statements and checked source decodes.
#[cfg(any(test, feature = "test-support"))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SourceReadMetrics {
    pub knn_queries: usize,
    pub sql_statements: usize,
    pub source_body_decodes: usize,
    pub prepared_content_batches: usize,
    pub summary_batches: usize,
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct SourceReadCapabilities {
    pub search_profiles: Vec<atlas_domain::SearchProfileId>,
    pub lexical: bool,
    pub semantic: bool,
    pub vector_unavailable_reason: Option<String>,
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct SourceArtifactStatistics {
    pub search_profiles: Vec<SourceProfileStatistics>,
    pub records: usize,
    pub product_records: usize,
    pub packs: usize,
    pub prepared_fields: usize,
    pub relationships: usize,
    pub lexical_units: usize,
    pub semantic_units: usize,
    pub snapshot_bytes: u64,
    pub prepared_html_bytes: u64,
    pub artifact_bytes: u64,
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct SourceProfileStatistics {
    pub profile_id: atlas_domain::SearchProfileId,
    pub semantic_units: usize,
    pub dimensions: usize,
}
#[derive(Debug, Clone)]
pub struct SourceSummaryBatch {
    pub records: Vec<SourceRecordSummary>,
    pub missing: Vec<RecordKey>,
}

impl SqliteIndexReader {
    #[cfg(any(test, feature = "test-support"))]
    pub fn read_metrics(&self) -> SourceReadMetrics {
        self.read_metrics.get()
    }
    #[cfg(any(test, feature = "test-support"))]
    pub fn reset_read_metrics(&self) {
        self.read_metrics.set(SourceReadMetrics::default());
    }
    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn count_content_batch(&self) {
        let mut m = self.read_metrics.get();
        m.prepared_content_batches += 1;
        self.read_metrics.set(m);
    }
    pub(crate) fn count_statement(&self) {
        #[cfg(any(test, feature = "test-support"))]
        {
            let mut m = self.read_metrics.get();
            m.sql_statements += 1;
            self.read_metrics.set(m);
        }
    }
    pub(crate) fn count_knn(&self) {
        #[cfg(any(test, feature = "test-support"))]
        {
            let mut m = self.read_metrics.get();
            m.knn_queries += 1;
            self.read_metrics.set(m);
        }
    }
    pub fn from_session(session: atlas_storage::ReadSession) -> Result<Self, IndexError> {
        let vector_error = atlas_sqlite_vec::register_sqlite_vec_auto_extension()
            .err()
            .map(|e| e.to_string());
        let (context, context_hash) = crate::validation::validate_open(session.connection())?;
        let instance =
            ArtifactInstance::new(session.workspace_key()?, session.reference_revision()?);
        Ok(Self {
            context,
            context_hash,
            instance,
            session,
            vector_error,
            #[cfg(any(test, feature = "test-support"))]
            read_metrics: std::cell::Cell::new(SourceReadMetrics::default()),
        })
    }
    /// Bounded canonical product keys for shell completion; never decodes bodies.
    pub fn complete_record_keys(&self, prefix: &str) -> Result<Vec<String>, IndexError> {
        let mut statement = self.session.connection().prepare(
            "SELECT key FROM records WHERE document_kind <> 'Macro' AND key >= ?1 AND key < ?1 || char(1114111) ORDER BY key LIMIT 100",
        )?;
        Ok(statement
            .query_map([prefix], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?)
    }

    pub fn workspace_session(&self) -> &atlas_storage::ReadSession {
        &self.session
    }
    pub fn path(&self) -> &Path {
        self.session.path()
    }
    pub fn context(&self) -> &SourceArtifactBuildContext {
        &self.context
    }
    /// Complete context digest verified from the exact admitted artifact bytes.
    pub fn context_sha256(&self) -> &str {
        &self.context_hash
    }
    /// Identity of the workspace revision visible to this bounded operation.
    pub fn artifact_instance(&self) -> &ArtifactInstance {
        &self.instance
    }
    pub fn capabilities(&self) -> SourceReadCapabilities {
        let reason = if self.context.search_profiles.is_empty() {
            Some("artifact has no semantic vectors".into())
        } else {
            self.vector_error.clone()
        };
        SourceReadCapabilities {
            search_profiles: self
                .context
                .search_profiles
                .iter()
                .map(|p| p.profile_id.clone())
                .collect(),
            lexical: true,
            semantic: reason.is_none(),
            vector_unavailable_reason: reason,
        }
    }
    pub fn require_vectors(&self) -> Result<(), IndexError> {
        let capabilities = self.capabilities();
        if let Some(reason) = capabilities.vector_unavailable_reason {
            return Err(IndexError::Unavailable(reason));
        }
        Ok(())
    }
    pub fn statistics(&self) -> Result<SourceArtifactStatistics, IndexError> {
        let c = self.native()?;
        self.count_statement();
        let mut stats=c.query_row("SELECT (SELECT count(*) FROM records),(SELECT count(*) FROM records WHERE document_kind<>'Macro'),(SELECT count(*) FROM packs),(SELECT count(*) FROM prepared_content),(SELECT count(*) FROM relationship_occurrences),(SELECT count(*) FROM lexical_units),(SELECT count(*) FROM semantic_units),(SELECT coalesce(sum(length(snapshot)),0) FROM record_bodies),(SELECT coalesce(sum(length(html_gzip)),0) FROM prepared_content)",[],|r|Ok(SourceArtifactStatistics{search_profiles:vec![],records:r.get(0)?,product_records:r.get(1)?,packs:r.get(2)?,prepared_fields:r.get(3)?,relationships:r.get(4)?,lexical_units:r.get(5)?,semantic_units:r.get(6)?,snapshot_bytes:r.get(7)?,prepared_html_bytes:r.get(8)?,artifact_bytes:0}))?;
        for profile in &self.context.search_profiles {
            self.count_statement();
            let count: usize = c.query_row(
                "SELECT count(*) FROM semantic_units WHERE profile_id=?",
                [profile.profile_id.as_str()],
                |row| row.get(0),
            )?;
            stats.search_profiles.push(SourceProfileStatistics {
                profile_id: profile.profile_id.clone(),
                semantic_units: count,
                dimensions: profile.dimensions,
            });
        }
        self.count_statement();
        let pages: u64 = c.query_row("PRAGMA page_count", [], |r| r.get(0))?;
        self.count_statement();
        let page_size: u64 = c.query_row("PRAGMA page_size", [], |r| r.get(0))?;
        stats.artifact_bytes = pages
            .checked_mul(page_size)
            .ok_or_else(|| IndexError::Invalid("artifact size overflow".into()))?;
        Ok(stats)
    }
    pub(crate) fn native(&self) -> Result<&NativeConnection, IndexError> {
        Ok(self.session.connection())
    }
    pub fn read_summary(&self, key: &RecordKey) -> Result<Option<SourceRecordSummary>, IndexError> {
        Ok(self
            .read_summaries(std::slice::from_ref(key))?
            .records
            .into_iter()
            .next())
    }
    pub fn read_summaries(&self, keys: &[RecordKey]) -> Result<SourceSummaryBatch, IndexError> {
        self.read_summaries_selected(keys, false)
    }
    /// Developer provenance inspection includes excluded tooling documents.
    pub fn read_summary_for_inspection(
        &self,
        key: &RecordKey,
    ) -> Result<Option<SourceRecordSummary>, IndexError> {
        Ok(self
            .read_summaries_selected(std::slice::from_ref(key), true)?
            .records
            .into_iter()
            .next())
    }
    fn read_summaries_selected(
        &self,
        keys: &[RecordKey],
        developer: bool,
    ) -> Result<SourceSummaryBatch, IndexError> {
        if keys.len() > 1024 {
            return Err(IndexError::Invalid(
                "summary batch exceeds 1024 keys".into(),
            ));
        }
        if keys.is_empty() {
            return Ok(SourceSummaryBatch {
                records: Vec::new(),
                missing: Vec::new(),
            });
        }
        let sql = format!(
            "{SUMMARY_SELECT} WHERE {} r.key IN ({})",
            if developer {
                ""
            } else {
                "r.document_kind<>'Macro' AND"
            },
            vec!["?"; keys.len()].join(",")
        );
        let parameters = keys.iter().map(ToString::to_string).collect::<Vec<_>>();
        let rows = self
            .session
            .connection()
            .prepare(&sql)?
            .query_map(rusqlite::params_from_iter(&parameters), SummaryRow::read)?
            .collect::<Result<Vec<_>, _>>()?;
        self.count_statement();
        #[cfg(any(test, feature = "test-support"))]
        {
            let mut m = self.read_metrics.get();
            m.summary_batches += 1;
            self.read_metrics.set(m);
        }
        let values = rows
            .into_iter()
            .map(SummaryRow::summary)
            .collect::<Result<Vec<_>, _>>()?;
        let map = values
            .into_iter()
            .map(|s| (s.key.clone(), s))
            .collect::<std::collections::BTreeMap<_, _>>();
        let mut records = Vec::new();
        let mut missing = Vec::new();
        for key in keys {
            if let Some(s) = map.get(key) {
                records.push(s.clone());
            } else {
                missing.push(key.clone());
            }
        }
        Ok(SourceSummaryBatch { records, missing })
    }
    pub fn read_source_record(
        &self,
        key: &RecordKey,
    ) -> Result<Option<SourceBackedRecord>, IndexError> {
        self.read_body(key, false)
    }
    /// Read at most 256 unique product snapshots in one SQL statement.
    pub fn read_source_records(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<SourceBackedRecord>, IndexError> {
        if keys.len() > 256 {
            return Err(IndexError::Limit(
                "source snapshot batch exceeds 256 records".into(),
            ));
        }
        if keys.is_empty() {
            return Ok(vec![]);
        }
        let placeholders = vec!["?"; keys.len()].join(",");
        let sql = format!(
            "SELECT r.pack_id,r.key,b.codec_version,b.encoding,b.snapshot FROM records r JOIN record_bodies b ON b.record_id=r.record_id WHERE r.key IN ({placeholders}) AND r.document_kind<>'Macro' ORDER BY r.key"
        );
        let parameters = keys.iter().map(ToString::to_string).collect::<Vec<_>>();
        self.count_statement();
        let mut statement = self.session.connection().prepare(&sql)?;
        let rows = statement.query_map(rusqlite::params_from_iter(parameters), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Vec<u8>>(4)?,
            ))
        })?;
        rows.map(|row| {
            let (pack, stored, version, encoding, bytes) = row?;
            let key = stored
                .parse()
                .map_err(|error| IndexError::Invalid(format!("source row identity: {error}")))?;
            self.decode_body(&key, pack, stored, version, encoding, bytes)
        })
        .collect()
    }
    /// Exact checked snapshot inspection remains a developer operation, including Macro sources.
    pub fn read_source_record_for_inspection(
        &self,
        key: &RecordKey,
    ) -> Result<Option<SourceBackedRecord>, IndexError> {
        self.read_body(key, true)
    }
    fn read_body(
        &self,
        key: &RecordKey,
        developer: bool,
    ) -> Result<Option<SourceBackedRecord>, IndexError> {
        self.count_statement();
        let row = self.session.connection().query_row(
            "SELECT r.pack_id,r.key,b.codec_version,b.encoding,b.snapshot FROM records r JOIN record_bodies b ON b.record_id=r.record_id WHERE r.key=? AND (? OR r.document_kind<>'Macro')",
            rusqlite::params![key.to_string(), developer],
            |r| Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)?,r.get::<_,String>(3)?,r.get::<_,Vec<u8>>(4)?)),
        ).optional()?;
        row.map(|(pack, stored, version, encoding, bytes)| {
            self.decode_body(key, pack, stored, version, encoding, bytes)
        })
        .transpose()
    }
    fn decode_body(
        &self,
        key: &RecordKey,
        pack: String,
        stored: String,
        version: i64,
        encoding: String,
        bytes: Vec<u8>,
    ) -> Result<SourceBackedRecord, IndexError> {
        if version != i64::from(SNAPSHOT_VERSION) || encoding != "gzip6" {
            return Err(IndexError::Unsupported(
                "source snapshot codec identity".into(),
            ));
        }
        let source = decode_snapshot(&gunzip(&bytes)?)?;
        #[cfg(any(test, feature = "test-support"))]
        {
            let mut m = self.read_metrics.get();
            m.source_body_decodes += 1;
            self.read_metrics.set(m);
        }
        let record = SourceBackedRecord::new(&pack, source)
            .map_err(|e| IndexError::Invalid(format!("source identity {:?}", e.reason)))?;
        if record.key() != key || stored != key.to_string() {
            return Err(IndexError::Invalid(
                "snapshot identity disagrees with record row".into(),
            ));
        }
        Ok(record)
    }
}
const SUMMARY_SELECT: &str = "SELECT r.key,p.label AS pack_label,r.document_kind,r.source_path,r.content_hash,r.name_state,r.name,r.source_type_state,r.source_type,r.record_kind_state,r.record_kind,r.rarity_state,r.rarity,r.traits_state,(SELECT json_group_array(value) FROM (SELECT value FROM record_traits rt WHERE rt.record_id=r.record_id ORDER BY value)) AS traits_json,CASE WHEN r.source_type='spell' THEN sp.rank_state ELSE r.level_state END AS level_state,CASE WHEN r.source_type='spell' THEN sp.rank ELSE r.level END AS level,CASE WHEN r.source_type='spell' THEN sp.ritual_state ELSE 'not_applicable' END AS ritual_state,sp.ritual,r.size_state,r.size,r.publication_title_state,r.publication_title,r.publication_remaster_state,r.publication_remaster FROM records r JOIN packs p ON p.pack_id=r.pack_id LEFT JOIN spell_projection sp ON sp.root_record_id=r.record_id";
struct SummaryRow {
    key: String,
    pack_label: String,
    document_kind: String,
    source_path: String,
    content_hash: String,
    name_state: String,
    name: Option<String>,
    source_type_state: String,
    source_type: Option<String>,
    record_kind_state: String,
    record_kind: Option<String>,
    rarity_state: String,
    rarity: Option<String>,
    traits_state: String,
    traits_json: String,
    level_state: String,
    level: Option<SourceNumber>,
    ritual_state: String,
    ritual: Option<i64>,
    size_state: String,
    size: Option<String>,
    publication_title_state: String,
    publication_title: Option<String>,
    publication_remaster_state: String,
    publication_remaster: Option<i64>,
}
pub(crate) fn field_state(value: &str) -> Result<QueryFieldState, IndexError> {
    match value {
        "value" => Ok(QueryFieldState::Value),
        "missing" => Ok(QueryFieldState::Missing),
        "null" => Ok(QueryFieldState::Null),
        "invalid" => Ok(QueryFieldState::Invalid),
        "not_applicable" => Ok(QueryFieldState::NotApplicable),
        _ => Err(IndexError::Invalid(format!("unknown field state {value}"))),
    }
}
fn fact<T>(state: &str, value: Option<T>) -> Result<SourceQueryFact<T>, IndexError> {
    let fact = SourceQueryFact {
        state: field_state(state)?,
        value,
    };
    if !fact.is_valid() {
        return Err(IndexError::Invalid(
            "summary field availability/value mismatch".into(),
        ));
    }
    Ok(fact)
}
impl SummaryRow {
    fn read(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            key: row.get("key")?,
            pack_label: row.get("pack_label")?,
            document_kind: row.get("document_kind")?,
            source_path: row.get("source_path")?,
            content_hash: row.get("content_hash")?,
            name_state: row.get("name_state")?,
            name: row.get("name")?,
            source_type_state: row.get("source_type_state")?,
            source_type: row.get("source_type")?,
            record_kind_state: row.get("record_kind_state")?,
            record_kind: row.get("record_kind")?,
            rarity_state: row.get("rarity_state")?,
            rarity: row.get("rarity")?,
            traits_state: row.get("traits_state")?,
            traits_json: row.get("traits_json")?,
            level_state: row.get("level_state")?,
            level: row.get("level")?,
            ritual_state: row.get("ritual_state")?,
            ritual: row.get("ritual")?,
            size_state: row.get("size_state")?,
            size: row.get("size")?,
            publication_title_state: row.get("publication_title_state")?,
            publication_title: row.get("publication_title")?,
            publication_remaster_state: row.get("publication_remaster_state")?,
            publication_remaster: row.get("publication_remaster")?,
        })
    }
    fn summary(self) -> Result<SourceRecordSummary, IndexError> {
        let level_basis = if self.source_type.as_deref() == Some("spell") {
            Some(SourceLevelBasis::SpellRank)
        } else if self.level_state != "not_applicable" {
            Some(if self.document_kind == "Actor" {
                SourceLevelBasis::ActorLevel
            } else {
                SourceLevelBasis::ItemLevel
            })
        } else {
            None
        };
        let kind = self
            .record_kind
            .map(|s| {
                s.parse::<RecordKind>()
                    .map_err(|e| IndexError::Invalid(e.to_string()))
            })
            .transpose()?;
        let traits = if self.traits_state == "value" {
            Some(serde_json::from_str(&self.traits_json)?)
        } else {
            None
        };
        Ok(SourceRecordSummary {
            key: RecordKey::parse(&self.key).map_err(|e| IndexError::Invalid(e.to_string()))?,
            pack_label: self.pack_label,
            document_kind: self.document_kind,
            name: fact(&self.name_state, self.name)?,
            source_type: fact(&self.source_type_state, self.source_type)?,
            record_kind: fact(&self.record_kind_state, kind)?,
            level: fact(&self.level_state, self.level.map(|n| n.0))?,
            level_basis,
            spell_ritual: fact(&self.ritual_state, self.ritual.map(|n| n != 0))?,
            rarity: fact(&self.rarity_state, self.rarity)?,
            traits: fact(&self.traits_state, traits)?,
            size: fact(&self.size_state, self.size)?,
            publication_title: fact(&self.publication_title_state, self.publication_title)?,
            publication_remaster: fact(
                &self.publication_remaster_state,
                self.publication_remaster.map(|n| n != 0),
            )?,
            source_path: self.source_path,
            content_hash: self.content_hash,
        })
    }
}
