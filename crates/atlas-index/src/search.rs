use atlas_domain::{RecordKey, SearchFilterNode};
use atlas_record::{PersistedRecord, PersistedRecordSet};

use crate::{
    FilterCompileError, FilteredRecordKeyPage, FilteredRecordSort, FtsQuery, FtsSearchHit,
    RecordEmbeddingVector, RecordLoadError, SqliteIndexReader, VectorQueryError, VectorSearchHit,
};

pub trait SearchIndex {
    fn load_records_by_key(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<PersistedRecord>, RecordLoadError>;

    fn load_record_set(&self) -> Result<PersistedRecordSet, RecordLoadError>;

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

impl SearchIndex for SqliteIndexReader {
    fn load_records_by_key(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<PersistedRecord>, RecordLoadError> {
        SqliteIndexReader::load_records_by_key(self, keys)
    }

    fn load_record_set(&self) -> Result<PersistedRecordSet, RecordLoadError> {
        SqliteIndexReader::load_record_set(self)
    }

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
