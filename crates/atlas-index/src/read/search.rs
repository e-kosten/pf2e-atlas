use atlas_domain::{RecordKey, RecordKind, SearchFilterNode};
use atlas_record::{AtlasRecord, AtlasRecordSet};

pub(crate) mod filters;
pub(crate) mod fts;
pub(crate) mod sqlite_vector_index;
pub(crate) mod vector;

use crate::{
    FilterCompileError, FilteredRecordKeyPage, FilteredRecordSort, FtsQuery, FtsSearchHit,
    RecordEmbeddingVector, RecordLoadError, SqliteIndexReader, VectorQueryError, VectorSearchHit,
};

pub trait RecordReadIndex {
    fn load_records_by_key(&self, keys: &[RecordKey]) -> Result<Vec<AtlasRecord>, RecordLoadError>;

    fn load_record_set(&self) -> Result<AtlasRecordSet, RecordLoadError>;

    fn load_search_candidate_records(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<SearchCandidateRecord>, RecordLoadError>;
}

pub trait IdentityReadIndex {
    fn resolve_record_identity_matches(
        &self,
        query: &str,
        normalized_query: &str,
        filter: Option<&SearchFilterNode>,
    ) -> Result<Vec<RecordIdentityMatch>, FilterCompileError>;
}

pub trait FilterReadIndex {
    fn resolve_metric_filters(
        &self,
        filter: Option<&SearchFilterNode>,
    ) -> Result<Option<SearchFilterNode>, FilterCompileError>;

    fn list_filtered_record_keys(
        &self,
        filter: Option<&SearchFilterNode>,
        sort: FilteredRecordSort,
        limit: u32,
        offset: u32,
    ) -> Result<FilteredRecordKeyPage, FilterCompileError>;
}

pub trait FtsReadIndex {
    fn query_precision_fts_index(
        &self,
        fts_query: &FtsQuery,
        filter: Option<&SearchFilterNode>,
        limit: u32,
    ) -> Result<Vec<FtsSearchHit>, FilterCompileError>;

    fn query_fts_candidate_record_keys(
        &self,
        fts_query: &FtsQuery,
        candidate_keys: &[RecordKey],
    ) -> Result<Vec<RecordKey>, FilterCompileError>;
}

pub trait VectorReadIndex {
    fn query_vector_index(
        &self,
        query_vector: &[f32],
        filter: Option<&SearchFilterNode>,
        limit: u32,
        include_child_units: bool,
    ) -> Result<Vec<VectorSearchHit>, VectorQueryError>;

    fn load_record_embedding_vectors(
        &self,
        record_key: &RecordKey,
    ) -> Result<Vec<RecordEmbeddingVector>, VectorQueryError>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchCandidateRecord {
    pub key: RecordKey,
    pub name: String,
    pub traits: Vec<String>,
    pub kind: RecordKind,
    pub foundry_record_type: String,
    pub taxonomy_families: Vec<String>,
    pub system_category: Option<String>,
    pub system_group: Option<String>,
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
pub enum RecordIdentityMatchKind {
    Name,
    NormalizedName,
    Alias,
    VariantName,
}

impl RecordReadIndex for SqliteIndexReader {
    fn load_records_by_key(&self, keys: &[RecordKey]) -> Result<Vec<AtlasRecord>, RecordLoadError> {
        SqliteIndexReader::load_records_by_key(self, keys)
    }

    fn load_record_set(&self) -> Result<AtlasRecordSet, RecordLoadError> {
        SqliteIndexReader::load_record_set(self)
    }

    fn load_search_candidate_records(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<SearchCandidateRecord>, RecordLoadError> {
        SqliteIndexReader::load_search_candidate_records(self, keys)
    }
}

impl IdentityReadIndex for SqliteIndexReader {
    fn resolve_record_identity_matches(
        &self,
        query: &str,
        normalized_query: &str,
        filter: Option<&SearchFilterNode>,
    ) -> Result<Vec<RecordIdentityMatch>, FilterCompileError> {
        SqliteIndexReader::resolve_record_identity_matches(self, query, normalized_query, filter)
    }
}

impl FilterReadIndex for SqliteIndexReader {
    fn resolve_metric_filters(
        &self,
        filter: Option<&SearchFilterNode>,
    ) -> Result<Option<SearchFilterNode>, FilterCompileError> {
        SqliteIndexReader::resolve_metric_filters(self, filter)
    }

    fn list_filtered_record_keys(
        &self,
        filter: Option<&SearchFilterNode>,
        sort: FilteredRecordSort,
        limit: u32,
        offset: u32,
    ) -> Result<FilteredRecordKeyPage, FilterCompileError> {
        SqliteIndexReader::list_filtered_record_keys(self, filter, sort, limit, offset)
    }
}

impl FtsReadIndex for SqliteIndexReader {
    fn query_precision_fts_index(
        &self,
        fts_query: &FtsQuery,
        filter: Option<&SearchFilterNode>,
        limit: u32,
    ) -> Result<Vec<FtsSearchHit>, FilterCompileError> {
        SqliteIndexReader::query_precision_fts_index(self, fts_query, filter, limit)
    }

    fn query_fts_candidate_record_keys(
        &self,
        fts_query: &FtsQuery,
        candidate_keys: &[RecordKey],
    ) -> Result<Vec<RecordKey>, FilterCompileError> {
        SqliteIndexReader::query_fts_candidate_record_keys(self, fts_query, candidate_keys)
    }
}

impl VectorReadIndex for SqliteIndexReader {
    fn query_vector_index(
        &self,
        query_vector: &[f32],
        filter: Option<&SearchFilterNode>,
        limit: u32,
        include_child_units: bool,
    ) -> Result<Vec<VectorSearchHit>, VectorQueryError> {
        SqliteIndexReader::query_vector_index(
            self,
            query_vector,
            filter,
            limit,
            include_child_units,
        )
    }

    fn load_record_embedding_vectors(
        &self,
        record_key: &RecordKey,
    ) -> Result<Vec<RecordEmbeddingVector>, VectorQueryError> {
        SqliteIndexReader::load_record_embedding_vectors(self, record_key)
    }
}
