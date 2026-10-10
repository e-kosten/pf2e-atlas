//! Read-only artifact handles. Summary queries never decode source bodies.
use crate::{
    IndexError, SourceArtifactBuildContext,
    codec::gunzip,
    numeric::{SourceNumber, SourceNumberSql},
    schema,
};
use atlas_domain::{
    RecordKey, RecordKind, SourceLevelBasis, SourceQueryFact, SourceRecordSummary,
    query::QueryFieldState,
};
use atlas_foundry_model::{SNAPSHOT_VERSION, decode_snapshot};
use atlas_record::source_record::SourceBackedRecord;
use diesel::{
    Connection, ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, RunQueryDsl,
    SqliteConnection,
    sql_types::{BigInt, Nullable, Text},
};
use rusqlite::{Connection as NativeConnection, OpenFlags};
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
};

pub struct SqliteIndexReader {
    path: PathBuf,
    diesel: RefCell<SqliteConnection>,
    context: SourceArtifactBuildContext,
    native: NativeConnection,
    vector_error: Option<String>,
    #[cfg(any(test, feature = "test-support"))]
    read_metrics: std::cell::Cell<SourceReadMetrics>,
}
/// Test-only counts of executed read statements and checked source decodes.
#[cfg(any(test, feature = "test-support"))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SourceReadMetrics {
    pub sql_statements: usize,
    pub source_body_decodes: usize,
    pub prepared_content_batches: usize,
    pub summary_batches: usize,
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct SourceReadCapabilities {
    pub lexical: bool,
    pub semantic: bool,
    pub vector_unavailable_reason: Option<String>,
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct SourceArtifactStatistics {
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
#[derive(Debug, Clone)]
pub struct SourceSummaryBatch {
    pub records: Vec<SourceRecordSummary>,
    pub missing: Vec<RecordKey>,
}

impl SqliteIndexReader {
    pub(crate) fn count_source_decode(&self) {
        #[cfg(any(test, feature = "test-support"))]
        {
            let mut m = self.read_metrics.get();
            m.source_body_decodes += 1;
            self.read_metrics.set(m);
        }
    }
    pub(crate) fn count_reuse_content_batch(&self) {
        #[cfg(any(test, feature = "test-support"))]
        self.count_content_batch();
    }
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
    pub fn open_read_only(path: impl AsRef<Path>) -> Result<Self, IndexError> {
        let path = path.as_ref().to_path_buf();
        let before = same_file::Handle::from_path(&path)
            .map_err(|e| IndexError::Unavailable(e.to_string()))?;
        let vector_error = atlas_sqlite_vec::register_sqlite_vec_auto_extension()
            .err()
            .map(|e| e.to_string());
        let c = NativeConnection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| IndexError::Unavailable(e.to_string()))?;
        let context = crate::validation::validate_open(&c)?;
        let url = read_only_uri(&path)?;
        let diesel = SqliteConnection::establish(&url)
            .map_err(|e| IndexError::Unavailable(e.to_string()))?;
        let after = same_file::Handle::from_path(&path)
            .map_err(|e| IndexError::Unavailable(e.to_string()))?;
        if before != after {
            return Err(IndexError::Unavailable(
                "artifact replaced while reader opened; retry".into(),
            ));
        }
        Ok(Self {
            path,
            diesel: RefCell::new(diesel),
            context,
            native: c,
            vector_error,
            #[cfg(any(test, feature = "test-support"))]
            read_metrics: std::cell::Cell::new(SourceReadMetrics::default()),
        })
    }
    pub fn open_read_only_with_vectors(path: impl AsRef<Path>) -> Result<Self, IndexError> {
        atlas_sqlite_vec::register_sqlite_vec_auto_extension()
            .map_err(|e| IndexError::Unavailable(e.to_string()))?;
        let reader = Self::open_read_only(path)?;
        if reader.context.semantic_model.is_none() {
            return Err(IndexError::Unavailable(
                "artifact has no semantic vectors; rebuild with embeddings".into(),
            ));
        }
        Ok(reader)
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn context(&self) -> &SourceArtifactBuildContext {
        &self.context
    }
    pub fn capabilities(&self) -> SourceReadCapabilities {
        let reason = if self.context.semantic_model.is_none() {
            Some("artifact has no semantic vectors".into())
        } else {
            self.vector_error.clone()
        };
        SourceReadCapabilities {
            lexical: true,
            semantic: reason.is_none(),
            vector_unavailable_reason: reason,
        }
    }
    pub(crate) fn require_vectors(&self) -> Result<(), IndexError> {
        let capabilities = self.capabilities();
        if let Some(reason) = capabilities.vector_unavailable_reason {
            return Err(IndexError::Unavailable(reason));
        }
        Ok(())
    }
    pub fn statistics(&self) -> Result<SourceArtifactStatistics, IndexError> {
        let c = self.native()?;
        self.count_statement();
        let mut stats=c.query_row("SELECT (SELECT count(*) FROM records),(SELECT count(*) FROM records WHERE document_kind<>'Macro'),(SELECT count(*) FROM packs),(SELECT count(*) FROM prepared_content),(SELECT count(*) FROM relationship_occurrences),(SELECT count(*) FROM lexical_units),(SELECT count(*) FROM semantic_units),(SELECT coalesce(sum(length(snapshot)),0) FROM record_bodies),(SELECT coalesce(sum(length(html_gzip)),0) FROM prepared_content)",[],|r|Ok(SourceArtifactStatistics{records:r.get(0)?,product_records:r.get(1)?,packs:r.get(2)?,prepared_fields:r.get(3)?,relationships:r.get(4)?,lexical_units:r.get(5)?,semantic_units:r.get(6)?,snapshot_bytes:r.get(7)?,prepared_html_bytes:r.get(8)?,artifact_bytes:0}))?;
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
        Ok(&self.native)
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
        let mut query = diesel::sql_query(sql).into_boxed::<diesel::sqlite::Sqlite>();
        for key in keys {
            query = query.bind::<Text, _>(key.to_string());
        }
        let rows = query.load::<SummaryRow>(&mut *self.diesel.borrow_mut())?;
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
        let row = schema::records::table
            .inner_join(
                schema::record_bodies::table
                    .on(schema::record_bodies::record_id.eq(schema::records::record_id)),
            )
            .filter(schema::records::key.eq(key.to_string()))
            .filter(schema::records::document_kind.ne(if developer { "" } else { "Macro" }))
            .select((
                schema::records::pack_id,
                schema::records::key,
                schema::record_bodies::codec_version,
                schema::record_bodies::encoding,
                schema::record_bodies::snapshot,
            ))
            .first::<(String, String, i64, String, Vec<u8>)>(&mut *self.diesel.borrow_mut())
            .optional()?;
        row.map(|(pack, stored, version, encoding, bytes)| {
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
        })
        .transpose()
    }
}
fn read_only_uri(path: &Path) -> Result<String, IndexError> {
    let path = path
        .to_str()
        .ok_or_else(|| IndexError::Unavailable("artifact path is not UTF-8".into()))?;
    Ok(format!(
        "file:{}?mode=ro",
        path.replace('%', "%25")
            .replace('?', "%3f")
            .replace('#', "%23")
    ))
}

const SUMMARY_SELECT: &str = "SELECT r.key,p.label AS pack_label,r.document_kind,r.source_path,r.content_hash,r.name_state,r.name,r.source_type_state,r.source_type,r.record_kind_state,r.record_kind,r.rarity_state,r.rarity,r.traits_state,(SELECT json_group_array(value) FROM (SELECT value FROM record_traits rt WHERE rt.record_id=r.record_id ORDER BY value)) AS traits_json,CASE WHEN r.source_type='spell' THEN sp.rank_state ELSE r.level_state END AS level_state,CASE WHEN r.source_type='spell' THEN sp.rank ELSE r.level END AS level,r.size_state,r.size,r.publication_title_state,r.publication_title,r.publication_remaster_state,r.publication_remaster FROM records r JOIN packs p ON p.pack_id=r.pack_id LEFT JOIN spell_projection sp ON sp.root_record_id=r.record_id";
#[derive(diesel::QueryableByName)]
struct SummaryRow {
    #[diesel(sql_type=Text)]
    key: String,
    #[diesel(sql_type=Text)]
    pack_label: String,
    #[diesel(sql_type=Text)]
    document_kind: String,
    #[diesel(sql_type=Text)]
    source_path: String,
    #[diesel(sql_type=Text)]
    content_hash: String,
    #[diesel(sql_type=Text)]
    name_state: String,
    #[diesel(sql_type=Nullable<Text>)]
    name: Option<String>,
    #[diesel(sql_type=Text)]
    source_type_state: String,
    #[diesel(sql_type=Nullable<Text>)]
    source_type: Option<String>,
    #[diesel(sql_type=Text)]
    record_kind_state: String,
    #[diesel(sql_type=Nullable<Text>)]
    record_kind: Option<String>,
    #[diesel(sql_type=Text)]
    rarity_state: String,
    #[diesel(sql_type=Nullable<Text>)]
    rarity: Option<String>,
    #[diesel(sql_type=Text)]
    traits_state: String,
    #[diesel(sql_type=Text)]
    traits_json: String,
    #[diesel(sql_type=Text)]
    level_state: String,
    #[diesel(sql_type=Nullable<SourceNumberSql>)]
    level: Option<SourceNumber>,
    #[diesel(sql_type=Text)]
    size_state: String,
    #[diesel(sql_type=Nullable<Text>)]
    size: Option<String>,
    #[diesel(sql_type=Text)]
    publication_title_state: String,
    #[diesel(sql_type=Nullable<Text>)]
    publication_title: Option<String>,
    #[diesel(sql_type=Text)]
    publication_remaster_state: String,
    #[diesel(sql_type=Nullable<BigInt>)]
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
