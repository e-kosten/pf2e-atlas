use std::path::{Path, PathBuf};

use atlas_artifact::schema::{record_aliases, records as record_table};
use atlas_domain::{FilterFieldDiscovery, FilterValueDiscovery, RecordKey, SearchFilterNode};
use atlas_record::{PersistedRecord, PersistedRecordSet};
use rusqlite::types::Value;
use rusqlite::{Connection, OpenFlags, params_from_iter};

use crate::discovery::{self, DiscoveryError, FilterValueRequest};
use crate::filters::{
    FilterCompileError, FilteredRecordKeysQuery, FilteredRecordSort as SqlFilteredRecordSort,
    compile_eligible_records_query, compile_filtered_record_keys_query,
};
use crate::fts;
use crate::relationship_edges::{GraphReferenceEdge, read_reference_edges_for_seed};
use crate::vector::register_sqlite_vec_extension;
use crate::{
    ArtifactValidationReport, IndexInspectionReport, IndexValidationError, RecordLoadError,
    RecordLoadOptions, SearchCandidateRecord, ValidationTarget, VectorQueryError, VectorSearchHit,
    check_index_connection, inspect, records, validate_index_connection, vector,
};

pub struct SqliteIndexReader {
    path: PathBuf,
    connection: Connection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordIdentityMatchKind {
    Name,
    NormalizedName,
    Alias,
    VariantName,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordIdentityMatch {
    pub record_key: RecordKey,
    pub match_kind: RecordIdentityMatchKind,
    pub matched_text: String,
    pub alias_source: Option<String>,
    pub alias_source_ref: Option<String>,
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

    pub(crate) fn connection(&self) -> &Connection {
        &self.connection
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

    pub fn resolve_record_identity_matches(
        &self,
        query: &str,
        normalized_query: &str,
        filter: Option<&SearchFilterNode>,
    ) -> Result<Vec<RecordIdentityMatch>, FilterCompileError> {
        let exact = self.resolve_record_identity_matches_for_record_field(
            RecordIdentityMatchKind::Name,
            record_table::columns::NAME.name(),
            query,
            None,
            filter,
        )?;
        if !exact.is_empty() {
            return Ok(exact);
        }

        let normalized = self.resolve_record_identity_matches_for_record_field(
            RecordIdentityMatchKind::NormalizedName,
            record_table::columns::NORMALIZED_NAME.name(),
            normalized_query,
            Some(format!(
                "{} IS NULL",
                record_table::columns::VARIANT_LABEL.name()
            )),
            filter,
        )?;
        if !normalized.is_empty() {
            return Ok(normalized);
        }

        let alias = self.resolve_alias_identity_matches(normalized_query, filter)?;
        if !alias.is_empty() {
            return Ok(alias);
        }

        self.resolve_record_identity_matches_for_record_field(
            RecordIdentityMatchKind::VariantName,
            record_table::columns::NORMALIZED_NAME.name(),
            normalized_query,
            Some(format!(
                "{} IS NOT NULL",
                record_table::columns::VARIANT_LABEL.name()
            )),
            filter,
        )
    }

    fn resolve_record_identity_matches_for_record_field(
        &self,
        match_kind: RecordIdentityMatchKind,
        column: &str,
        value: &str,
        extra_predicate: Option<String>,
        filter: Option<&SearchFilterNode>,
    ) -> Result<Vec<RecordIdentityMatch>, FilterCompileError> {
        let eligible = compile_eligible_records_query(filter)?;
        let mut parameters = eligible.parameters;
        parameters.push(Value::Text(value.to_string()));
        let value_placeholder = format!("?{}", parameters.len());
        let extra_predicate = extra_predicate
            .map(|predicate| format!(" AND {predicate}"))
            .unwrap_or_default();
        let matched_expression = match match_kind {
            RecordIdentityMatchKind::Name => record_table::columns::NAME.name(),
            _ => record_table::columns::NORMALIZED_NAME.name(),
        };
        let sql = format!(
            "WITH eligible(record_key) AS ({})
             SELECT r.{record_key}, r.{matched_expression}
             FROM {records_table} r
             JOIN eligible e ON e.record_key = r.{record_key}
             WHERE r.{column} = {value_placeholder}{extra_predicate}
             ORDER BY r.{record_key}",
            eligible.sql,
            record_key = record_table::columns::RECORD_KEY.name(),
            records_table = record_table::TABLE.name(),
        );
        let mut statement = self.connection.prepare(&sql).map_err(|error| {
            FilterCompileError::QueryFailed(format!("record identity query failed: {error}"))
        })?;
        let rows = statement
            .query_map(params_from_iter(parameters), |row| {
                Ok(RecordIdentityMatch {
                    record_key: RecordKey::parse(&row.get::<_, String>(0)?).map_err(|error| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(error),
                        )
                    })?,
                    match_kind,
                    matched_text: row.get(1)?,
                    alias_source: None,
                    alias_source_ref: None,
                })
            })
            .map_err(|error| {
                FilterCompileError::QueryFailed(format!("record identity query failed: {error}"))
            })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|error| {
            FilterCompileError::QueryFailed(format!("record identity query failed: {error}"))
        })
    }

    fn resolve_alias_identity_matches(
        &self,
        normalized_query: &str,
        filter: Option<&SearchFilterNode>,
    ) -> Result<Vec<RecordIdentityMatch>, FilterCompileError> {
        let eligible = compile_eligible_records_query(filter)?;
        let mut parameters = eligible.parameters;
        parameters.push(Value::Text(normalized_query.to_string()));
        let value_placeholder = format!("?{}", parameters.len());
        let sql = format!(
            "WITH eligible(record_key) AS ({})
             SELECT a.{canonical_record_key}, a.{alias_text}, a.{source_kind}, a.{source_ref}
             FROM {aliases_table} a
             JOIN eligible e ON e.record_key = a.{canonical_record_key}
             WHERE a.{normalized_alias} = {value_placeholder}
             ORDER BY a.{canonical_record_key}, a.{alias_text}",
            eligible.sql,
            aliases_table = record_aliases::TABLE.name(),
            canonical_record_key = record_aliases::columns::CANONICAL_RECORD_KEY.name(),
            alias_text = record_aliases::columns::ALIAS_TEXT.name(),
            normalized_alias = record_aliases::columns::NORMALIZED_ALIAS.name(),
            source_kind = record_aliases::columns::SOURCE_KIND.name(),
            source_ref = record_aliases::columns::SOURCE_REF.name(),
        );
        let mut statement = self.connection.prepare(&sql).map_err(|error| {
            FilterCompileError::QueryFailed(format!("record alias query failed: {error}"))
        })?;
        let rows = statement
            .query_map(params_from_iter(parameters), |row| {
                Ok(RecordIdentityMatch {
                    record_key: RecordKey::parse(&row.get::<_, String>(0)?).map_err(|error| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(error),
                        )
                    })?,
                    match_kind: RecordIdentityMatchKind::Alias,
                    matched_text: row.get(1)?,
                    alias_source: Some(row.get(2)?),
                    alias_source_ref: Some(row.get(3)?),
                })
            })
            .map_err(|error| {
                FilterCompileError::QueryFailed(format!("record alias query failed: {error}"))
            })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|error| {
            FilterCompileError::QueryFailed(format!("record alias query failed: {error}"))
        })
    }

    pub fn load_records_by_key(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<PersistedRecord>, RecordLoadError> {
        records::load_persisted_records_by_key_from_connection(&self.connection, keys)
    }

    pub fn load_records_by_key_with_options(
        &self,
        keys: &[RecordKey],
        options: RecordLoadOptions,
    ) -> Result<Vec<PersistedRecord>, RecordLoadError> {
        records::load_persisted_records_by_key_from_connection_with_options(
            &self.connection,
            keys,
            options,
        )
    }

    pub fn load_search_candidate_records(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<SearchCandidateRecord>, RecordLoadError> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }
        let parameters = keys
            .iter()
            .map(|key| Value::Text(key.to_string()))
            .collect::<Vec<_>>();
        let placeholders = (1..=parameters.len())
            .map(|index| format!("?{index}"))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "SELECT {record_key}, {name}, {traits_json}, {taxonomy_families_json},
                    {prerequisites_json}, {system_category}, {system_group}
             FROM {table}
             WHERE {record_key} IN ({placeholders})
             ORDER BY {record_key}",
            table = record_table::TABLE.name(),
            record_key = record_table::columns::RECORD_KEY.name(),
            name = record_table::columns::NAME.name(),
            traits_json = record_table::columns::TRAITS_JSON.name(),
            taxonomy_families_json = record_table::columns::TAXONOMY_FAMILIES_JSON.name(),
            prerequisites_json = record_table::columns::PREREQUISITES_JSON.name(),
            system_category = record_table::columns::SYSTEM_CATEGORY.name(),
            system_group = record_table::columns::SYSTEM_GROUP.name(),
        );
        let mut statement = self
            .connection
            .prepare(&sql)
            .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
        let rows = statement
            .query_map(params_from_iter(parameters.iter()), |row| {
                let record_key = row.get::<_, String>(0)?;
                let traits_json = row.get::<_, String>(2)?;
                let taxonomy_families_json = row.get::<_, String>(3)?;
                let prerequisites_json = row.get::<_, String>(4)?;
                Ok((
                    record_key,
                    row.get::<_, String>(1)?,
                    traits_json,
                    taxonomy_families_json,
                    prerequisites_json,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                ))
            })
            .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
        rows.map(|row| {
            let (
                record_key,
                name,
                traits_json,
                taxonomy_families_json,
                prerequisites_json,
                system_category,
                system_group,
            ) = row.map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
            Ok(SearchCandidateRecord {
                key: RecordKey::parse(&record_key)
                    .map_err(|error| RecordLoadError::InvalidData(error.to_string()))?,
                name,
                traits: json_string_array("records.traits_json", &traits_json)?,
                taxonomy_families: json_string_array(
                    "records.taxonomy_families_json",
                    &taxonomy_families_json,
                )?,
                prerequisites: json_string_array(
                    "records.prerequisites_json",
                    &prerequisites_json,
                )?,
                system_category,
                system_group,
            })
        })
        .collect()
    }

    pub fn reference_edges_for_seed(
        &self,
        seed: &RecordKey,
        direction: ReferenceEdgeDirection,
    ) -> Result<Vec<GraphReferenceEdge>, RecordLoadError> {
        read_reference_edges_for_seed(&self.connection, seed, direction)
    }

    pub fn list_filter_fields(
        &self,
        filter: Option<&SearchFilterNode>,
        filter_json: Option<serde_json::Value>,
        force_dynamic: bool,
    ) -> Result<FilterFieldDiscovery, DiscoveryError> {
        discovery::list_filter_fields(&self.connection, filter, filter_json, force_dynamic)
    }

    pub fn list_filter_values(
        &self,
        filter: Option<&SearchFilterNode>,
        request: FilterValueRequest,
    ) -> Result<FilterValueDiscovery, DiscoveryError> {
        discovery::list_filter_values(&self.connection, filter, request)
    }

    pub fn resolve_metric_filters(
        &self,
        filter: Option<&SearchFilterNode>,
    ) -> Result<Option<SearchFilterNode>, FilterCompileError> {
        discovery::resolve_filter_metrics(&self.connection, filter)
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

    pub fn filter_record_keys(
        &self,
        candidate_keys: &[RecordKey],
        filter: Option<&SearchFilterNode>,
    ) -> Result<Vec<RecordKey>, FilterCompileError> {
        if candidate_keys.is_empty() {
            return Ok(Vec::new());
        }
        if filter.is_none() {
            return Ok(candidate_keys.to_vec());
        }

        let eligible = compile_eligible_records_query(filter)?;
        let mut parameters = eligible.parameters;
        let mut candidate_rows = Vec::with_capacity(candidate_keys.len());
        for (ordinal, key) in candidate_keys.iter().enumerate() {
            parameters.push(Value::Text(key.to_string()));
            let key_placeholder = format!("?{}", parameters.len());
            parameters.push(Value::Integer(ordinal as i64));
            let ordinal_placeholder = format!("?{}", parameters.len());
            candidate_rows.push(format!("({key_placeholder}, {ordinal_placeholder})"));
        }
        let sql = format!(
            "WITH eligible(record_key) AS ({}),
                  candidate(record_key, ordinal) AS (VALUES {})
             SELECT candidate.record_key
             FROM candidate
             JOIN eligible ON eligible.record_key = candidate.record_key
             ORDER BY candidate.ordinal",
            eligible.sql,
            candidate_rows.join(", ")
        );
        let query = FilteredRecordKeysQuery { sql, parameters };
        read_record_keys(&self.connection, &query)
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

    pub fn load_record_embedding_units(
        &self,
        record_key: &RecordKey,
    ) -> Result<Vec<crate::RecordEmbeddingUnit>, VectorQueryError> {
        vector::load_record_embedding_units(&self.connection, record_key)
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
        fts::query_fts_index(&self.connection, fts_query, filter, limit, weights)
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
        fts::query_precision_fts_index(&self.connection, fts_query, filter, limit)
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
        fts::query_fts_record_keys(&self.connection, fts_query, filter, limit)
    }

    pub fn query_fts_candidate_record_keys(
        &self,
        fts_query: &FtsQuery,
        candidate_keys: &[RecordKey],
    ) -> Result<Vec<RecordKey>, FilterCompileError> {
        if candidate_keys.is_empty() {
            return Ok(Vec::new());
        }
        fts::query_fts_candidate_record_keys(&self.connection, fts_query, candidate_keys)
    }
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
        FilteredRecordSort::PriceAsc => SqlFilteredRecordSort::PriceAsc,
        FilteredRecordSort::PriceDesc => SqlFilteredRecordSort::PriceDesc,
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

fn json_string_array(context: &str, value: &str) -> Result<Vec<String>, RecordLoadError> {
    serde_json::from_str(value)
        .map_err(|error| RecordLoadError::InvalidData(format!("{context}: {error}")))
}
