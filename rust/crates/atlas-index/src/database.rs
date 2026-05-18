use std::path::{Path, PathBuf};

use atlas_artifact::schema::TABLE_RECORDS_FTS;
use atlas_domain::{FilterFieldDiscovery, FilterValueDiscovery, RecordKey, SearchFilterNode};
use atlas_record::{PersistedRecord, PersistedRecordSet};
use rusqlite::types::Value;
use rusqlite::{Connection, OpenFlags, params_from_iter};

use crate::discovery::{self, DiscoveryError, FilterValueRequest};
use crate::filters::{
    FilterCompileError, FilteredRecordKeysQuery, FilteredRecordSort as SqlFilteredRecordSort,
    compile_eligible_records_query, compile_filtered_record_keys_query,
};
use crate::vector::register_sqlite_vec_extension;
use crate::{
    ArtifactValidationReport, IndexInspectionReport, IndexValidationError, RecordLoadError,
    ValidationTarget, VectorQueryError, VectorSearchHit, check_index_connection, inspect, records,
    validate_index_connection, vector,
};

pub struct AtlasIndex {
    path: PathBuf,
    connection: Connection,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FtsColumnWeights {
    pub title: f64,
    pub aliases: f64,
    pub traits: f64,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FtsQuery {
    tokens: Vec<String>,
}

impl FtsQuery {
    pub fn from_tokens(tokens: Vec<String>) -> Option<Self> {
        let tokens = tokens
            .into_iter()
            .filter(|token| is_safe_fts_token(token))
            .collect::<Vec<_>>();
        (!tokens.is_empty()).then_some(Self { tokens })
    }

    pub fn as_match_query(&self) -> String {
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
    Random { seed: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilteredRecordKeyPage {
    pub record_keys: Vec<RecordKey>,
    pub total: u64,
}

impl AtlasIndex {
    pub fn open_read_only(path: impl AsRef<Path>) -> Result<Self, IndexValidationError> {
        let path = path.as_ref().to_path_buf();
        let connection = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
        Ok(Self { path, connection })
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

    pub fn validate(&self) -> Result<ArtifactValidationReport, IndexValidationError> {
        validate_index_connection(self.path.display().to_string(), &self.connection)
    }

    pub fn validate_report(&self) -> ArtifactValidationReport {
        match self.validate() {
            Ok(report) => report,
            Err(error) => crate::validation_report_from_error(&self.path, error),
        }
    }

    pub fn check(&self) -> Result<ArtifactValidationReport, IndexValidationError> {
        check_index_connection(self.path.display().to_string(), &self.connection)
    }

    pub fn check_report(&self) -> ArtifactValidationReport {
        match self.check() {
            Ok(report) => report,
            Err(error) => crate::validation_report_from_error(&self.path, error),
        }
    }

    pub fn check_embedding_readiness_report(&self) -> ArtifactValidationReport {
        match self.check() {
            Ok(report) => match vector::check_embedding_readiness_connection(
                self.path.display().to_string(),
                report,
                &self.connection,
            ) {
                Ok(report) => report,
                Err(error) => crate::validation_report_from_error(&self.path, error),
            },
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
            &self.connection,
        )
    }

    pub fn vector_extension_unavailable_report(
        &self,
        target: ValidationTarget,
        message: String,
    ) -> ArtifactValidationReport {
        let base_report = match target {
            ValidationTarget::EmbeddingsOnly => {
                match crate::validate_index_metadata_connection(
                    self.path.display().to_string(),
                    &self.connection,
                ) {
                    Ok(report) => report,
                    Err(error) => crate::validation_report_from_error(&self.path, error),
                }
            }
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
            &self.connection,
        )
    }

    pub fn load_records(&self) -> Result<Vec<PersistedRecord>, RecordLoadError> {
        records::load_persisted_records_from_connection(&self.connection)
    }

    pub fn load_record_set(&self) -> Result<PersistedRecordSet, RecordLoadError> {
        records::load_persisted_record_set_from_connection(&self.connection)
    }

    pub fn load_records_by_key(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<PersistedRecord>, RecordLoadError> {
        records::load_persisted_records_by_key_from_connection(&self.connection, keys)
    }

    pub fn list_filter_fields(
        &self,
        filter: Option<&SearchFilterNode>,
        filter_json: Option<serde_json::Value>,
    ) -> Result<FilterFieldDiscovery, DiscoveryError> {
        discovery::list_filter_fields(&self.connection, filter, filter_json)
    }

    pub fn list_filter_values(
        &self,
        filter: Option<&SearchFilterNode>,
        request: FilterValueRequest,
    ) -> Result<FilterValueDiscovery, DiscoveryError> {
        discovery::list_filter_values(&self.connection, filter, request)
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
                let query = compile_filtered_record_keys_query(
                    filter,
                    SqlFilteredRecordSort::RecordKeyAsc,
                    None,
                    None,
                )?;
                let mut record_keys = read_record_keys(&self.connection, &query)?;
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
                let total = count_filtered_records(&self.connection, filter)?;
                let query = compile_filtered_record_keys_query(
                    filter,
                    sql_sort(sort),
                    Some(limit),
                    Some(offset),
                )?;
                Ok(FilteredRecordKeyPage {
                    record_keys: read_record_keys(&self.connection, &query)?,
                    total,
                })
            }
        }
    }

    pub fn validate_vector_index(&self) -> Result<ArtifactValidationReport, IndexValidationError> {
        vector::validate_vector_index_connection(
            self.path.display().to_string(),
            self.validate()?,
            &self.connection,
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
            &self.connection,
            query_vector,
            filter,
            limit,
            include_child_units,
        )
    }

    pub fn query_fts_index(
        &self,
        fts_query: &FtsQuery,
        filter: Option<&SearchFilterNode>,
        limit: u32,
        weights: FtsColumnWeights,
    ) -> Result<Vec<FtsSearchHit>, FilterCompileError> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        query_fts_index(&self.connection, fts_query, filter, limit, weights)
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
        query_fts_record_keys(&self.connection, fts_query, filter, limit)
    }

    pub fn query_fts_candidate_record_keys(
        &self,
        fts_query: &FtsQuery,
        candidate_keys: &[RecordKey],
    ) -> Result<Vec<RecordKey>, FilterCompileError> {
        if candidate_keys.is_empty() {
            return Ok(Vec::new());
        }
        query_fts_candidate_record_keys(&self.connection, fts_query, candidate_keys)
    }
}

fn query_fts_index(
    connection: &Connection,
    fts_query: &FtsQuery,
    filter: Option<&SearchFilterNode>,
    limit: u32,
    weights: FtsColumnWeights,
) -> Result<Vec<FtsSearchHit>, FilterCompileError> {
    let eligible = compile_eligible_records_query(filter)?;
    let mut parameters = eligible.parameters;
    parameters.push(Value::Text(fts_query.as_match_query()));
    parameters.push(Value::Integer(i64::from(limit)));
    let sql = format!(
        "WITH eligible(record_key) AS ({eligible_sql})
         SELECT f.record_key,
                bm25({fts_table}, 0.0, {title}, {aliases}, {traits}, {headings}, {body}, {facts}, {reference_terms}, {embedded_content}) AS rank
         FROM {fts_table} f
         WHERE {fts_table} MATCH ?{query_index}
           AND f.record_key IN (SELECT record_key FROM eligible)
         ORDER BY rank ASC, f.record_key ASC
         LIMIT ?{limit_index}",
        eligible_sql = eligible.sql,
        fts_table = TABLE_RECORDS_FTS,
        title = weights.title,
        aliases = weights.aliases,
        traits = weights.traits,
        headings = weights.headings,
        body = weights.body,
        facts = weights.facts,
        reference_terms = weights.reference_terms,
        embedded_content = weights.embedded_content,
        query_index = parameters.len() - 1,
        limit_index = parameters.len(),
    );

    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| FilterCompileError::QueryFailed(error.to_string()))?;
    statement
        .query_map(params_from_iter(parameters.iter()), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })
        .map_err(|error| FilterCompileError::QueryFailed(error.to_string()))?
        .map(|row| {
            row.map_err(|error| FilterCompileError::QueryFailed(error.to_string()))
                .and_then(|(record_key, rank)| {
                    Ok(FtsSearchHit {
                        record_key: RecordKey::parse(&record_key)
                            .map_err(|error| FilterCompileError::InvalidValue(error.to_string()))?,
                        rank,
                    })
                })
        })
        .collect()
}

fn query_fts_record_keys(
    connection: &Connection,
    fts_query: &FtsQuery,
    filter: Option<&SearchFilterNode>,
    limit: u32,
) -> Result<Vec<RecordKey>, FilterCompileError> {
    let eligible = compile_eligible_records_query(filter)?;
    let mut parameters = eligible.parameters;
    parameters.push(Value::Text(fts_query.as_match_query()));
    parameters.push(Value::Integer(i64::from(limit)));
    let sql = format!(
        "WITH eligible(record_key) AS ({eligible_sql})
         SELECT f.record_key
         FROM {fts_table} f
         WHERE {fts_table} MATCH ?{query_index}
           AND f.record_key IN (SELECT record_key FROM eligible)
         ORDER BY f.record_key ASC
         LIMIT ?{limit_index}",
        eligible_sql = eligible.sql,
        fts_table = TABLE_RECORDS_FTS,
        query_index = parameters.len() - 1,
        limit_index = parameters.len(),
    );

    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| FilterCompileError::QueryFailed(error.to_string()))?;
    statement
        .query_map(params_from_iter(parameters.iter()), |row| {
            row.get::<_, String>(0)
        })
        .map_err(|error| FilterCompileError::QueryFailed(error.to_string()))?
        .map(|row| {
            row.map_err(|error| FilterCompileError::QueryFailed(error.to_string()))
                .and_then(|record_key| {
                    RecordKey::parse(&record_key)
                        .map_err(|error| FilterCompileError::InvalidValue(error.to_string()))
                })
        })
        .collect()
}

fn query_fts_candidate_record_keys(
    connection: &Connection,
    fts_query: &FtsQuery,
    candidate_keys: &[RecordKey],
) -> Result<Vec<RecordKey>, FilterCompileError> {
    let mut parameters = vec![Value::Text(fts_query.as_match_query())];
    let query_placeholder = "?1";
    let candidate_placeholders = candidate_keys
        .iter()
        .map(|key| {
            parameters.push(Value::Text(key.to_string()));
            format!("?{}", parameters.len())
        })
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT f.record_key
         FROM {fts_table} f
         WHERE {fts_table} MATCH {query_placeholder}
           AND f.record_key IN ({candidate_placeholders})
         ORDER BY f.record_key ASC",
        fts_table = TABLE_RECORDS_FTS,
    );

    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| FilterCompileError::QueryFailed(error.to_string()))?;
    statement
        .query_map(params_from_iter(parameters.iter()), |row| {
            row.get::<_, String>(0)
        })
        .map_err(|error| FilterCompileError::QueryFailed(error.to_string()))?
        .map(|row| {
            row.map_err(|error| FilterCompileError::QueryFailed(error.to_string()))
                .and_then(|record_key| {
                    RecordKey::parse(&record_key)
                        .map_err(|error| FilterCompileError::InvalidValue(error.to_string()))
                })
        })
        .collect()
}

fn count_filtered_records(
    connection: &Connection,
    filter: Option<&SearchFilterNode>,
) -> Result<u64, FilterCompileError> {
    let eligible = compile_eligible_records_query(filter)?;
    let sql = format!(
        "WITH eligible(record_key) AS ({}) SELECT COUNT(*) FROM eligible",
        eligible.sql
    );
    connection
        .query_row(&sql, params_from_iter(eligible.parameters.iter()), |row| {
            row.get::<_, u64>(0)
        })
        .map_err(|error| FilterCompileError::QueryFailed(error.to_string()))
}

fn read_record_keys(
    connection: &Connection,
    query: &FilteredRecordKeysQuery,
) -> Result<Vec<RecordKey>, FilterCompileError> {
    let mut statement = connection
        .prepare(&query.sql)
        .map_err(|error| FilterCompileError::QueryFailed(error.to_string()))?;
    let keys = statement
        .query_map(params_from_iter(query.parameters.iter()), |row| {
            row.get::<_, String>(0)
        })
        .map_err(|error| FilterCompileError::QueryFailed(error.to_string()))?
        .map(|row| {
            row.map_err(|error| FilterCompileError::QueryFailed(error.to_string()))
                .and_then(|value| {
                    RecordKey::parse(&value)
                        .map_err(|error| FilterCompileError::InvalidValue(error.to_string()))
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(keys)
}

fn sql_sort(sort: FilteredRecordSort) -> SqlFilteredRecordSort {
    match sort {
        FilteredRecordSort::RecordKey => SqlFilteredRecordSort::RecordKeyAsc,
        FilteredRecordSort::Alphabetical => SqlFilteredRecordSort::NameAsc,
        FilteredRecordSort::LevelAsc => SqlFilteredRecordSort::LevelAsc,
        FilteredRecordSort::LevelDesc => SqlFilteredRecordSort::LevelDesc,
        FilteredRecordSort::Random { .. } => SqlFilteredRecordSort::RecordKeyAsc,
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
