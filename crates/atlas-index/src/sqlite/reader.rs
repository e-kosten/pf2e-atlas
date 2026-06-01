use std::cell::RefCell;
use std::path::{Path, PathBuf};

use atlas_domain::{FilterFieldDiscovery, FilterValueDiscovery, RecordKey, SearchFilterNode};
use atlas_record::{PersistedRecord, PersistedRecordSet};
use diesel::connection::SimpleConnection;
use diesel::{Connection as DieselConnection, RunQueryDsl, SqliteConnection};
use rusqlite::{Connection, OpenFlags};

use crate::discovery::{self, DiscoveryError, FilterValueRequest};
use crate::filters::{
    FilterCompileError, SqliteEligibleRecordKeyset, SqliteFilterSqlQuery,
    SqliteFilteredRecordSort as SqliteKeysetRecordSort,
};
use crate::fts;
use crate::relationship_edges::{GraphReferenceEdge, read_reference_edges_for_seed};
use crate::sqlite::raw_sql::{CountRow, RecordKeyRow, bind_sql_query};
use crate::vector::register_sqlite_vec_extension;
use crate::{
    ArtifactValidationReport, IndexInspectionReport, IndexValidationError, RecordIdentityMatch,
    RecordLoadError, SearchCandidateRecord, ValidationTarget, VectorQueryError, VectorSearchHit,
    check_index_connection, inspect, records, validate_index_connection, vector,
};

pub struct SqliteIndexReader {
    path: PathBuf,
    diesel_connection: RefCell<SqliteConnection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceEdgeDirection {
    Outgoing,
    Backlink,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FtsColumnWeights {
    pub title: f64,
    pub aliases: f64,
    pub traits: f64,
    pub taxonomy_terms: f64,
    pub constraint_terms: f64,
    pub mechanic_terms: f64,
    pub source_terms: f64,
    pub metric_terms: f64,
    pub headings: f64,
    pub body: f64,
    pub facts: f64,
    pub reference_terms: f64,
    pub embedded_content: f64,
}

impl Default for FtsColumnWeights {
    fn default() -> Self {
        Self {
            title: 8.0,
            aliases: 8.0,
            traits: 4.0,
            taxonomy_terms: 2.5,
            constraint_terms: 2.0,
            mechanic_terms: 1.5,
            source_terms: 0.5,
            metric_terms: 1.0,
            headings: 4.0,
            facts: 2.0,
            body: 1.0,
            reference_terms: 0.5,
            embedded_content: 0.5,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FtsSearchHit {
    pub record_key: RecordKey,
    pub rank: f64,
    pub lane: FtsSearchLane,
    pub lane_rank: u32,
    pub title_alias_texts: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FtsSearchLane {
    Mixed,
    TitleAlias,
    Facet,
}

impl FtsSearchLane {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mixed => "mixed",
            Self::TitleAlias => "title-alias",
            Self::Facet => "facet",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FtsQuery {
    pub(crate) tokens: Vec<String>,
}

impl FtsQuery {
    pub fn from_tokens(tokens: Vec<String>) -> Option<Self> {
        let tokens = tokens
            .into_iter()
            .filter(|token| is_safe_fts_token(token))
            .map(|token| token.to_lowercase())
            .collect::<Vec<_>>();
        (!tokens.is_empty()).then_some(Self { tokens })
    }

    pub fn as_match_query(&self) -> String {
        self.as_disjunction_match_query()
    }

    pub(crate) fn as_conjunction_match_query(&self) -> String {
        self.tokens
            .iter()
            .filter(|token| !fts::is_primary_type_intent_token(token))
            .map(|token| format!("\"{token}\""))
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub(crate) fn as_disjunction_match_query(&self) -> String {
        self.tokens
            .iter()
            .map(|token| format!("\"{token}\""))
            .collect::<Vec<_>>()
            .join(" OR ")
    }
}

fn is_safe_fts_token(token: &str) -> bool {
    !token.is_empty() && token.chars().all(char::is_alphanumeric)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilteredRecordSort {
    RecordKey,
    Alphabetical,
    LevelAsc,
    LevelDesc,
    PriceAsc,
    PriceDesc,
    Random { seed: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilteredRecordKeyPage {
    pub record_keys: Vec<RecordKey>,
    pub total: u64,
}

impl SqliteIndexReader {
    pub fn open_read_only(path: impl AsRef<Path>) -> Result<Self, IndexValidationError> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() {
            return Err(IndexValidationError::Unavailable(format!(
                "unable to open database file: {}",
                path.display()
            )));
        }
        let database_url = read_only_sqlite_uri(&path)?;
        let mut diesel_connection = SqliteConnection::establish(&database_url)
            .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
        diesel_connection
            .batch_execute("PRAGMA query_only = ON")
            .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
        Ok(Self {
            path,
            diesel_connection: RefCell::new(diesel_connection),
        })
    }

    pub fn open_read_only_with_vectors(
        path: impl AsRef<Path>,
    ) -> Result<Self, IndexValidationError> {
        register_sqlite_vec_extension().map_err(IndexValidationError::Unavailable)?;
        Self::open_read_only(path)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn validation_connection(&self) -> Result<Connection, IndexValidationError> {
        Connection::open_with_flags(&self.path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|error| IndexValidationError::Unavailable(error.to_string()))
    }

    pub(crate) fn with_diesel_connection<T>(
        &self,
        f: impl FnOnce(&mut SqliteConnection) -> T,
    ) -> T {
        f(&mut self.diesel_connection.borrow_mut())
    }

    pub fn validate(&self) -> Result<ArtifactValidationReport, IndexValidationError> {
        validate_index_connection(
            self.path.display().to_string(),
            &self.validation_connection()?,
        )
    }

    pub fn validate_report(&self) -> ArtifactValidationReport {
        match self.validate() {
            Ok(report) => report,
            Err(error) => crate::validation_report_from_error(&self.path, error),
        }
    }

    pub fn check(&self) -> Result<ArtifactValidationReport, IndexValidationError> {
        check_index_connection(
            self.path.display().to_string(),
            &self.validation_connection()?,
        )
    }

    pub fn check_report(&self) -> ArtifactValidationReport {
        match self.check() {
            Ok(report) => report,
            Err(error) => crate::validation_report_from_error(&self.path, error),
        }
    }

    pub fn check_embedding_readiness_report(&self) -> ArtifactValidationReport {
        match self.check() {
            Ok(report) => {
                let connection = match self.validation_connection() {
                    Ok(connection) => connection,
                    Err(error) => return crate::validation_report_from_error(&self.path, error),
                };
                match vector::check_embedding_readiness_connection(
                    self.path.display().to_string(),
                    report,
                    &connection,
                ) {
                    Ok(report) => report,
                    Err(error) => crate::validation_report_from_error(&self.path, error),
                }
            }
            Err(error) => crate::validation_report_from_error(&self.path, error),
        }
    }

    pub fn validate_target(
        &self,
        target: ValidationTarget,
    ) -> Result<ArtifactValidationReport, IndexValidationError> {
        match target {
            ValidationTarget::BaseOnly => self.validate(),
            ValidationTarget::Full => self.validate_vector_index(),
            ValidationTarget::EmbeddingsOnly => self.validate_embedding_readiness(),
        }
    }

    pub fn validate_embedding_readiness(
        &self,
    ) -> Result<ArtifactValidationReport, IndexValidationError> {
        vector::validate_embedding_readiness_connection(
            self.path.display().to_string(),
            &self.validation_connection()?,
        )
    }

    pub fn vector_extension_unavailable_report(
        &self,
        target: ValidationTarget,
        message: String,
    ) -> ArtifactValidationReport {
        let base_report = match target {
            ValidationTarget::EmbeddingsOnly => match self.validation_connection() {
                Ok(connection) => {
                    match crate::validate_index_metadata_connection(
                        self.path.display().to_string(),
                        &connection,
                    ) {
                        Ok(report) => report,
                        Err(error) => crate::validation_report_from_error(&self.path, error),
                    }
                }
                Err(error) => crate::validation_report_from_error(&self.path, error),
            },
            ValidationTarget::BaseOnly | ValidationTarget::Full => self.validate_report(),
        };
        vector::vector_extension_unavailable_report_from_base(
            self.path.display().to_string(),
            base_report,
            message,
        )
    }

    pub fn validate_target_report(&self, target: ValidationTarget) -> ArtifactValidationReport {
        match self.validate_target(target) {
            Ok(report) => report,
            Err(error) => crate::validation_report_from_error(&self.path, error),
        }
    }

    pub fn inspect(&self) -> Result<IndexInspectionReport, IndexValidationError> {
        let validation = self.validate()?;
        inspect::inspect_index_connection(
            self.path.display().to_string(),
            validation,
            &self.validation_connection()?,
        )
    }

    pub fn load_records(&self) -> Result<Vec<PersistedRecord>, RecordLoadError> {
        records::load_persisted_records_from_diesel_connection(
            &mut self.diesel_connection.borrow_mut(),
        )
    }

    pub fn load_record_set(&self) -> Result<PersistedRecordSet, RecordLoadError> {
        records::load_persisted_record_set_from_diesel_connection(
            &mut self.diesel_connection.borrow_mut(),
        )
    }

    pub fn load_records_by_key(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<PersistedRecord>, RecordLoadError> {
        records::load_persisted_records_by_key_from_diesel_connection(
            &mut self.diesel_connection.borrow_mut(),
            keys,
        )
    }

    pub fn load_search_candidate_records(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<SearchCandidateRecord>, RecordLoadError> {
        records::load_search_candidate_records_from_diesel_connection(
            &mut self.diesel_connection.borrow_mut(),
            keys,
        )
    }

    pub fn resolve_record_identity_matches(
        &self,
        query: &str,
        normalized_query: &str,
        filter: Option<&SearchFilterNode>,
    ) -> Result<Vec<RecordIdentityMatch>, FilterCompileError> {
        records::resolve_record_identity_matches_from_diesel_connection(
            &mut self.diesel_connection.borrow_mut(),
            query,
            normalized_query,
            filter,
        )
    }

    pub fn reference_edges_for_seed(
        &self,
        seed: &RecordKey,
        direction: ReferenceEdgeDirection,
    ) -> Result<Vec<GraphReferenceEdge>, RecordLoadError> {
        read_reference_edges_for_seed(&mut self.diesel_connection.borrow_mut(), seed, direction)
    }

    pub fn list_filter_fields(
        &self,
        filter: Option<&SearchFilterNode>,
        filter_json: Option<serde_json::Value>,
    ) -> Result<FilterFieldDiscovery, DiscoveryError> {
        discovery::list_filter_fields(
            &mut self.diesel_connection.borrow_mut(),
            filter,
            filter_json,
        )
    }

    pub fn list_filter_values(
        &self,
        filter: Option<&SearchFilterNode>,
        request: FilterValueRequest,
    ) -> Result<FilterValueDiscovery, DiscoveryError> {
        discovery::list_filter_values(&mut self.diesel_connection.borrow_mut(), filter, request)
    }

    pub fn resolve_metric_filters(
        &self,
        filter: Option<&SearchFilterNode>,
    ) -> Result<Option<SearchFilterNode>, FilterCompileError> {
        discovery::resolve_filter_metrics(&mut self.diesel_connection.borrow_mut(), filter)
            .map_err(|error| FilterCompileError::InvalidValue(error.to_string()))
    }

    pub fn list_filtered_record_keys(
        &self,
        filter: Option<&SearchFilterNode>,
        sort: FilteredRecordSort,
        limit: u32,
        offset: u32,
    ) -> Result<FilteredRecordKeyPage, FilterCompileError> {
        match sort {
            FilteredRecordSort::Random { seed } => {
                let query = SqliteEligibleRecordKeyset::new(filter)
                    .compile()?
                    .into_record_keys_query(SqliteKeysetRecordSort::RecordKeyAsc, None, None);
                let mut record_keys =
                    read_record_keys(&mut self.diesel_connection.borrow_mut(), &query)?;
                record_keys.sort_by_key(|key| seeded_key_hash(seed, &key.to_string()));
                let total = record_keys.len() as u64;
                let record_keys = record_keys
                    .into_iter()
                    .skip(offset as usize)
                    .take(limit as usize)
                    .collect();
                Ok(FilteredRecordKeyPage { record_keys, total })
            }
            sort => {
                let total =
                    count_filtered_records(&mut self.diesel_connection.borrow_mut(), filter)?;
                let query = SqliteEligibleRecordKeyset::new(filter)
                    .compile()?
                    .into_record_keys_query(sql_sort(sort), Some(limit), Some(offset));
                Ok(FilteredRecordKeyPage {
                    record_keys: read_record_keys(
                        &mut self.diesel_connection.borrow_mut(),
                        &query,
                    )?,
                    total,
                })
            }
        }
    }

    pub fn validate_vector_index(&self) -> Result<ArtifactValidationReport, IndexValidationError> {
        vector::validate_vector_index_connection(
            self.path.display().to_string(),
            self.validate()?,
            &self.validation_connection()?,
        )
    }

    pub fn vector_validation_report(&self) -> ArtifactValidationReport {
        match self.validate_vector_index() {
            Ok(report) => report,
            Err(error) => crate::validation_report_from_error(&self.path, error),
        }
    }

    pub fn query_vector_index(
        &self,
        query_vector: &[f32],
        filter: Option<&SearchFilterNode>,
        limit: u32,
        include_child_units: bool,
    ) -> Result<Vec<VectorSearchHit>, VectorQueryError> {
        vector::query_vector_index(
            &mut self.diesel_connection.borrow_mut(),
            query_vector,
            filter,
            limit,
            include_child_units,
        )
    }

    pub fn load_record_embedding_vectors(
        &self,
        record_key: &RecordKey,
    ) -> Result<Vec<crate::RecordEmbeddingVector>, VectorQueryError> {
        vector::load_record_embedding_vectors(&mut self.diesel_connection.borrow_mut(), record_key)
    }

    /// Query the broad weighted FTS projection directly.
    ///
    /// Product ranked search uses [`Self::query_precision_fts_index`]. This
    /// lower-level path remains available for diagnostics, tests, and measured
    /// tuning of broad FTS behavior.
    pub fn query_weighted_fts_index(
        &self,
        fts_query: &FtsQuery,
        filter: Option<&SearchFilterNode>,
        limit: u32,
        weights: FtsColumnWeights,
    ) -> Result<Vec<FtsSearchHit>, FilterCompileError> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        fts::query_weighted_fts_index(
            &mut self.diesel_connection.borrow_mut(),
            fts_query,
            filter,
            limit,
            weights,
        )
    }

    pub fn query_precision_fts_index(
        &self,
        fts_query: &FtsQuery,
        filter: Option<&SearchFilterNode>,
        limit: u32,
    ) -> Result<Vec<FtsSearchHit>, FilterCompileError> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        fts::query_precision_fts_index(
            &mut self.diesel_connection.borrow_mut(),
            fts_query,
            filter,
            limit,
        )
    }

    pub fn query_fts_record_keys(
        &self,
        fts_query: &FtsQuery,
        filter: Option<&SearchFilterNode>,
        limit: u32,
    ) -> Result<Vec<RecordKey>, FilterCompileError> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        fts::query_fts_record_keys(
            &mut self.diesel_connection.borrow_mut(),
            fts_query,
            filter,
            limit,
        )
    }

    pub fn query_fts_candidate_record_keys(
        &self,
        fts_query: &FtsQuery,
        candidate_keys: &[RecordKey],
    ) -> Result<Vec<RecordKey>, FilterCompileError> {
        if candidate_keys.is_empty() {
            return Ok(Vec::new());
        }
        fts::query_fts_candidate_record_keys(
            &mut self.diesel_connection.borrow_mut(),
            fts_query,
            candidate_keys,
        )
    }
}

fn count_filtered_records(
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
) -> Result<u64, FilterCompileError> {
    let query = SqliteEligibleRecordKeyset::new(filter)
        .compile()?
        .count_query();
    bind_sql_query(query.sql, &query.parameters)
        .get_result::<CountRow>(connection)
        .map(|row| row.count as u64)
        .map_err(|error| FilterCompileError::QueryFailed(error.to_string()))
}

fn read_record_keys(
    connection: &mut SqliteConnection,
    query: &SqliteFilterSqlQuery,
) -> Result<Vec<RecordKey>, FilterCompileError> {
    let keys = bind_sql_query(query.sql.clone(), &query.parameters)
        .load::<RecordKeyRow>(connection)
        .map_err(|error| FilterCompileError::QueryFailed(error.to_string()))?
        .into_iter()
        .map(|row| {
            RecordKey::parse(&row.record_key)
                .map_err(|error| FilterCompileError::InvalidValue(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(keys)
}

fn sql_sort(sort: FilteredRecordSort) -> SqliteKeysetRecordSort {
    match sort {
        FilteredRecordSort::RecordKey => SqliteKeysetRecordSort::RecordKeyAsc,
        FilteredRecordSort::Alphabetical => SqliteKeysetRecordSort::NameAsc,
        FilteredRecordSort::LevelAsc => SqliteKeysetRecordSort::LevelAsc,
        FilteredRecordSort::LevelDesc => SqliteKeysetRecordSort::LevelDesc,
        FilteredRecordSort::PriceAsc => SqliteKeysetRecordSort::PriceAsc,
        FilteredRecordSort::PriceDesc => SqliteKeysetRecordSort::PriceDesc,
        FilteredRecordSort::Random { .. } => SqliteKeysetRecordSort::RecordKeyAsc,
    }
}

fn seeded_key_hash(seed: u64, key: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64 ^ seed;
    for byte in key.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn read_only_sqlite_uri(path: &Path) -> Result<String, IndexValidationError> {
    let path = path.to_str().ok_or_else(|| {
        IndexValidationError::Unavailable(format!(
            "SQLite artifact path is not valid UTF-8: {}",
            path.display()
        ))
    })?;
    let mut escaped = String::with_capacity(path.len());
    for ch in path.chars() {
        match ch {
            '?' => escaped.push_str("%3f"),
            '#' => escaped.push_str("%23"),
            '%' => escaped.push_str("%25"),
            _ => escaped.push(ch),
        }
    }
    Ok(format!("file:{escaped}?mode=ro"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn read_only_uri_rejects_non_utf8_paths() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let path = PathBuf::from(OsString::from_vec(b"atlas-index-\xff.sqlite".to_vec()));
        let error = read_only_sqlite_uri(&path).expect_err("non-UTF-8 path should be rejected");

        assert!(matches!(error, IndexValidationError::Unavailable(_)));
        assert!(error.to_string().contains("not valid UTF-8"));
    }
}
