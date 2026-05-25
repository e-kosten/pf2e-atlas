use std::collections::BTreeMap;

use atlas_domain::{FilterFieldDiscovery, FilterValueDiscovery, RecordKey, SearchFilterNode};
use atlas_record::{PersistedRecord, PersistedRecordSet};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    DiscoveryError, FilterCompileError, FilterValueRequest, FilteredRecordKeyPage,
    FilteredRecordSort, FtsColumnWeights, FtsQuery, FtsSearchHit, GraphReferenceEdge,
    IndexValidationError, RecordIdentityMatchKind, RecordLoadError, ReferenceEdgeDirection,
    SqliteIndexReader, VectorQueryError, VectorSearchHit,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordLoadOptions {
    pub include_raw_json: bool,
}

impl RecordLoadOptions {
    pub const fn include_raw_json() -> Self {
        Self {
            include_raw_json: true,
        }
    }

    pub const fn omit_raw_json() -> Self {
        Self {
            include_raw_json: false,
        }
    }
}

impl Default for RecordLoadOptions {
    fn default() -> Self {
        Self::include_raw_json()
    }
}

pub trait SearchIndex {
    fn load_records_by_key(&self, keys: &[RecordKey]) -> Result<Vec<PersistedRecord>, SearchError>;
    fn load_records_by_key_with_options(
        &self,
        keys: &[RecordKey],
        options: RecordLoadOptions,
    ) -> Result<Vec<PersistedRecord>, SearchError> {
        let _ = options;
        self.load_records_by_key(keys)
    }
    fn load_search_candidate_records(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<SearchCandidateRecord>, SearchError>;
    fn load_record_set(&self) -> Result<PersistedRecordSet, SearchError>;
    fn resolve_record_matches(
        &self,
        _query: &str,
        _normalized_query: &str,
        _filter: Option<&SearchFilterNode>,
    ) -> Result<Option<Vec<RecordResolutionResult>>, SearchError> {
        Ok(None)
    }
    fn resolve_record_matches_with_options(
        &self,
        query: &str,
        normalized_query: &str,
        filter: Option<&SearchFilterNode>,
        options: RecordLoadOptions,
    ) -> Result<Option<Vec<RecordResolutionResult>>, SearchError> {
        let _ = options;
        self.resolve_record_matches(query, normalized_query, filter)
    }
    fn reference_edges_for_seed(
        &self,
        seed: &RecordKey,
        direction: ReferenceEdgeDirection,
    ) -> Result<Vec<GraphReferenceEdge>, SearchError>;
    fn resolve_metric_filters(
        &self,
        filter: Option<&SearchFilterNode>,
    ) -> Result<Option<SearchFilterNode>, SearchError>;
    fn list_filtered_record_keys(
        &self,
        filter: Option<&SearchFilterNode>,
        sort: FilteredRecordSort,
        limit: u32,
        offset: u32,
    ) -> Result<FilteredRecordKeyPage, SearchError>;
    fn list_filter_fields(
        &self,
        filter: Option<&SearchFilterNode>,
        filter_json: Option<serde_json::Value>,
        force_dynamic: bool,
    ) -> Result<FilterFieldDiscovery, DiscoveryError>;
    fn list_filter_values(
        &self,
        filter: Option<&SearchFilterNode>,
        request: FilterValueRequest,
    ) -> Result<FilterValueDiscovery, DiscoveryError>;
    fn query_fts_index(
        &self,
        fts_query: &FtsQuery,
        filter: Option<&SearchFilterNode>,
        limit: u32,
        weights: FtsColumnWeights,
    ) -> Result<Vec<FtsSearchHit>, SearchError>;
    fn query_fts_candidate_record_keys(
        &self,
        fts_query: &FtsQuery,
        candidate_keys: &[RecordKey],
    ) -> Result<Vec<RecordKey>, SearchError>;
    fn query_vector_index(
        &self,
        query_vector: &[f32],
        filter: Option<&SearchFilterNode>,
        limit: u32,
        include_child_units: bool,
    ) -> Result<Vec<VectorSearchHit>, SearchError>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchCandidateRecord {
    pub key: RecordKey,
    pub name: String,
    pub traits: Vec<String>,
    pub taxonomy_families: Vec<String>,
    pub prerequisites: Vec<String>,
    pub system_category: Option<String>,
    pub system_group: Option<String>,
}

impl SearchIndex for SqliteIndexReader {
    fn load_records_by_key(&self, keys: &[RecordKey]) -> Result<Vec<PersistedRecord>, SearchError> {
        Ok(SqliteIndexReader::load_records_by_key(self, keys)?)
    }

    fn load_records_by_key_with_options(
        &self,
        keys: &[RecordKey],
        options: RecordLoadOptions,
    ) -> Result<Vec<PersistedRecord>, SearchError> {
        Ok(SqliteIndexReader::load_records_by_key_with_options(
            self, keys, options,
        )?)
    }

    fn load_search_candidate_records(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<SearchCandidateRecord>, SearchError> {
        Ok(SqliteIndexReader::load_search_candidate_records(
            self, keys,
        )?)
    }

    fn load_record_set(&self) -> Result<PersistedRecordSet, SearchError> {
        Ok(SqliteIndexReader::load_record_set(self)?)
    }

    fn resolve_record_matches(
        &self,
        query: &str,
        normalized_query: &str,
        filter: Option<&SearchFilterNode>,
    ) -> Result<Option<Vec<RecordResolutionResult>>, SearchError> {
        self.resolve_record_matches_with_options(
            query,
            normalized_query,
            filter,
            RecordLoadOptions::include_raw_json(),
        )
    }

    fn resolve_record_matches_with_options(
        &self,
        query: &str,
        normalized_query: &str,
        filter: Option<&SearchFilterNode>,
        options: RecordLoadOptions,
    ) -> Result<Option<Vec<RecordResolutionResult>>, SearchError> {
        let matches = SqliteIndexReader::resolve_record_identity_matches(
            self,
            query,
            normalized_query,
            filter,
        )?;
        if matches.is_empty() {
            return Ok(Some(Vec::new()));
        }
        let keys = matches
            .iter()
            .map(|hit| hit.record_key.clone())
            .collect::<Vec<_>>();
        let records_by_key =
            SqliteIndexReader::load_records_by_key_with_options(self, &keys, options)?
                .into_iter()
                .map(|record| (record.key.clone(), record))
                .collect::<BTreeMap<_, _>>();
        Ok(Some(
            matches
                .into_iter()
                .filter_map(|hit| {
                    let record = records_by_key.get(&hit.record_key)?.clone();
                    Some(RecordResolutionResult {
                        query: query.to_string(),
                        normalized_query: normalized_query.to_string(),
                        match_kind: match hit.match_kind {
                            RecordIdentityMatchKind::Name => RecordResolutionMatchKind::Name,
                            RecordIdentityMatchKind::NormalizedName => {
                                RecordResolutionMatchKind::NormalizedName
                            }
                            RecordIdentityMatchKind::Alias => RecordResolutionMatchKind::Alias,
                            RecordIdentityMatchKind::VariantName => {
                                RecordResolutionMatchKind::VariantName
                            }
                        },
                        matched_text: hit.matched_text,
                        alias_source: hit.alias_source,
                        alias_source_ref: hit.alias_source_ref,
                        record,
                    })
                })
                .collect(),
        ))
    }

    fn reference_edges_for_seed(
        &self,
        seed: &RecordKey,
        direction: ReferenceEdgeDirection,
    ) -> Result<Vec<GraphReferenceEdge>, SearchError> {
        Ok(SqliteIndexReader::reference_edges_for_seed(
            self, seed, direction,
        )?)
    }

    fn resolve_metric_filters(
        &self,
        filter: Option<&SearchFilterNode>,
    ) -> Result<Option<SearchFilterNode>, SearchError> {
        Ok(SqliteIndexReader::resolve_metric_filters(self, filter)?)
    }

    fn list_filtered_record_keys(
        &self,
        filter: Option<&SearchFilterNode>,
        sort: FilteredRecordSort,
        limit: u32,
        offset: u32,
    ) -> Result<FilteredRecordKeyPage, SearchError> {
        Ok(SqliteIndexReader::list_filtered_record_keys(
            self, filter, sort, limit, offset,
        )?)
    }

    fn list_filter_fields(
        &self,
        filter: Option<&SearchFilterNode>,
        filter_json: Option<serde_json::Value>,
        force_dynamic: bool,
    ) -> Result<FilterFieldDiscovery, DiscoveryError> {
        SqliteIndexReader::list_filter_fields(self, filter, filter_json, force_dynamic)
    }

    fn list_filter_values(
        &self,
        filter: Option<&SearchFilterNode>,
        request: FilterValueRequest,
    ) -> Result<FilterValueDiscovery, DiscoveryError> {
        SqliteIndexReader::list_filter_values(self, filter, request)
    }

    fn query_fts_index(
        &self,
        fts_query: &FtsQuery,
        filter: Option<&SearchFilterNode>,
        limit: u32,
        _weights: FtsColumnWeights,
    ) -> Result<Vec<FtsSearchHit>, SearchError> {
        Ok(SqliteIndexReader::query_precision_fts_index(
            self, fts_query, filter, limit,
        )?)
    }

    fn query_fts_candidate_record_keys(
        &self,
        fts_query: &FtsQuery,
        candidate_keys: &[RecordKey],
    ) -> Result<Vec<RecordKey>, SearchError> {
        Ok(SqliteIndexReader::query_fts_candidate_record_keys(
            self,
            fts_query,
            candidate_keys,
        )?)
    }

    fn query_vector_index(
        &self,
        query_vector: &[f32],
        filter: Option<&SearchFilterNode>,
        limit: u32,
        include_child_units: bool,
    ) -> Result<Vec<VectorSearchHit>, SearchError> {
        Ok(SqliteIndexReader::query_vector_index(
            self,
            query_vector,
            filter,
            limit,
            include_child_units,
        )?)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordResolutionResult {
    pub query: String,
    pub normalized_query: String,
    pub match_kind: RecordResolutionMatchKind,
    pub matched_text: String,
    pub alias_source: Option<String>,
    pub alias_source_ref: Option<String>,
    pub record: PersistedRecord,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordResolutionMatchKind {
    Name,
    NormalizedName,
    Alias,
    VariantName,
}

impl RecordResolutionMatchKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::NormalizedName => "normalized_name",
            Self::Alias => "alias",
            Self::VariantName => "variant_name",
        }
    }
}

#[derive(Debug, Error)]
pub enum SearchError {
    #[error(transparent)]
    Index(#[from] IndexValidationError),
    #[error(transparent)]
    RecordLoad(#[from] RecordLoadError),
    #[error("invalid embedding model `{model}`: {message}")]
    InvalidEmbeddingModel { model: String, message: String },
    #[error("embedding operation failed: {0}")]
    Embedding(String),
    #[error("invalid search options: {0}")]
    InvalidSearchOptions(String),
    #[error("retrieval pattern is not implemented yet: {0}")]
    UnsupportedRetrievalPattern(&'static str),
    #[error(transparent)]
    Filter(#[from] FilterCompileError),
    #[error(transparent)]
    Vector(#[from] VectorQueryError),
}
