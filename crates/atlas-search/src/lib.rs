#![deny(unsafe_code)]

use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;

use atlas_domain::{FilterFieldDiscovery, FilterValueDiscovery, RecordKey, SearchFilterNode};
use atlas_embedding::{EmbeddingModelId, EmbeddingRuntimeConfig, TextEmbedder};
use atlas_index::{
    DiscoveryError, FilterValueRequest, FilteredRecordKeyPage, FilteredRecordSort,
    GraphProductIndex, RecordLoadOptions, RemasterLinks, SqliteIndexReader, VariantGroup,
};
use atlas_record::PersistedRecord;

mod fusion;
mod graph_context;
mod graph_expand;
mod query;
mod references;
mod resolution;
mod semantic;
mod text;

pub use atlas_index::{
    RecordResolutionMatchKind, RecordResolutionResult, SearchError, SearchIndex,
};
pub use fusion::{
    FtsFusionPolicy, FtsMatchConfidence, FusionMethod, FusionOptions, TextSearchExplain,
};
pub use graph_context::{
    GraphContextEdge, GraphContextEdgeSource, GraphContextRequest, GraphContextResult,
    GraphContextSection,
};
pub use graph_expand::{
    GraphExpandEvidence, GraphExpandMechanic, GraphExpandRecord, GraphExpandRequest,
    GraphExpandResult, GraphExpandSemanticSeed,
};
pub use query::TextQueryAnalysis;
pub(crate) use query::{analyze_text_query, normalize_record_query};
pub use semantic::{
    SemanticSearchHit, SemanticSearchMode, SemanticSearchResult, SemanticSearchTiming,
};
use semantic::{collapse_vector_hits, semantic_unit_limit};
pub use text::{
    AtlasSearchRequest, AtlasSearchResult, RetrievalMode, TextSearchMatch, TextSearchPage,
    TextSearchRecord, TextSearchRequest,
};

/// Product-facing retrieval boundary for runtime consumers.
///
/// The service owns record lookup, filter-only browse, graph context, and
/// ranked lexical/vector/hybrid retrieval. Concrete index implementations stay
/// behind focused index traits.
pub struct AtlasRetrievalService {
    pub(crate) index: Box<dyn RetrievalIndex>,
    embedder: Option<TextEmbedder>,
}

pub trait RetrievalIndex: SearchIndex + GraphProductIndex {}

impl<T> RetrievalIndex for T where T: SearchIndex + GraphProductIndex {}

#[derive(Debug, Clone, PartialEq)]
pub struct FilterOnlyRecordPage {
    pub record_keys: Vec<RecordKey>,
    pub records: Vec<PersistedRecord>,
    pub total: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchEmbeddingConfig {
    pub model_id: String,
    pub cache_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphVariantGroupResult {
    pub seed: Option<PersistedRecord>,
    pub group: VariantGroup,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphRemasterLinksResult {
    pub seed: PersistedRecord,
    pub links: RemasterLinks,
}

impl AtlasRetrievalService {
    pub fn new(
        index: SqliteIndexReader,
        embedding_config: &SearchEmbeddingConfig,
    ) -> Result<Self, SearchError> {
        Self::new_with_index(Box::new(index), embedding_config)
    }

    pub fn new_with_index(
        index: Box<dyn RetrievalIndex>,
        embedding_config: &SearchEmbeddingConfig,
    ) -> Result<Self, SearchError> {
        Ok(Self {
            embedder: Some(load_embedder(embedding_config)?),
            index,
        })
    }

    pub fn without_embeddings(index: SqliteIndexReader) -> Self {
        Self::without_embeddings_with_index(Box::new(index))
    }

    pub fn without_embeddings_with_index(index: Box<dyn RetrievalIndex>) -> Self {
        Self {
            index,
            embedder: None,
        }
    }

    pub fn semantic(
        &mut self,
        query: &str,
        filter: Option<&SearchFilterNode>,
        limit: u32,
        mode: SemanticSearchMode,
    ) -> Result<Vec<SemanticSearchHit>, SearchError> {
        Ok(self.semantic_with_timing(query, filter, limit, mode)?.hits)
    }

    pub fn semantic_with_timing(
        &mut self,
        query: &str,
        filter: Option<&SearchFilterNode>,
        limit: u32,
        mode: SemanticSearchMode,
    ) -> Result<SemanticSearchResult, SearchError> {
        let resolved_filter = self.index.resolve_metric_filters(filter)?;
        let filter = resolved_filter.as_ref().or(filter);
        if let Some(filter) = filter {
            filter
                .validate()
                .map_err(|error| SearchError::InvalidSearchOptions(error.to_string()))?;
        }
        let total_started_at = Instant::now();
        let embedding_started_at = Instant::now();
        let embedder = self
            .embedder
            .as_mut()
            .ok_or(SearchError::UnsupportedRetrievalPattern("semantic search"))?;
        let query_vector = embedder
            .embed_query(query)
            .map_err(|error| SearchError::Embedding(error.to_string()))?;
        let query_embedding_duration_ms = embedding_started_at.elapsed().as_millis();
        let vector_started_at = Instant::now();
        let raw_limit = semantic_unit_limit(limit, mode);
        let hits = self.index.query_vector_index(
            &query_vector,
            filter,
            raw_limit,
            mode.includes_child_units(),
        )?;
        let vector_search_duration_ms = vector_started_at.elapsed().as_millis();
        Ok(SemanticSearchResult {
            hits: collapse_vector_hits(hits, limit as usize, mode),
            timing: SemanticSearchTiming {
                query_embedding_duration_ms,
                vector_search_duration_ms,
                total_duration_ms: total_started_at.elapsed().as_millis(),
            },
        })
    }

    pub fn search(
        &mut self,
        request: AtlasSearchRequest<'_>,
    ) -> Result<AtlasSearchResult, SearchError> {
        match request {
            AtlasSearchRequest::Semantic {
                query,
                filter,
                limit,
                mode,
            } => self
                .semantic_with_timing(query, filter, limit, mode)
                .map(AtlasSearchResult::Semantic),
            AtlasSearchRequest::FilterOnly { .. } => Err(SearchError::UnsupportedRetrievalPattern(
                "filter-only search",
            )),
            AtlasSearchRequest::Text(request) => {
                self.text_search(request).map(AtlasSearchResult::Text)
            }
        }
    }

    pub fn get_records(
        &self,
        record_keys: &[RecordKey],
    ) -> Result<Vec<PersistedRecord>, SearchError> {
        self.get_records_with_options(record_keys, RecordLoadOptions::include_raw_json())
    }

    pub fn get_records_with_options(
        &self,
        record_keys: &[RecordKey],
        options: RecordLoadOptions,
    ) -> Result<Vec<PersistedRecord>, SearchError> {
        let mut records = self
            .index
            .load_records_by_key_with_options(record_keys, options)?;
        self.enrich_reference_labels(&mut records)?;
        Ok(records)
    }

    pub fn get_record(
        &self,
        record_key: &RecordKey,
    ) -> Result<Option<PersistedRecord>, SearchError> {
        Ok(self
            .get_records(std::slice::from_ref(record_key))?
            .into_iter()
            .next())
    }

    pub fn filter_only_records(
        &self,
        filter: Option<&SearchFilterNode>,
        sort: FilteredRecordSort,
        limit: u32,
        offset: u32,
    ) -> Result<FilterOnlyRecordPage, SearchError> {
        self.filter_only_records_with_options(
            filter,
            sort,
            limit,
            offset,
            RecordLoadOptions::include_raw_json(),
        )
    }

    pub fn filter_only_records_with_options(
        &self,
        filter: Option<&SearchFilterNode>,
        sort: FilteredRecordSort,
        limit: u32,
        offset: u32,
        options: RecordLoadOptions,
    ) -> Result<FilterOnlyRecordPage, SearchError> {
        let resolved_filter = self.index.resolve_metric_filters(filter)?;
        let filter = resolved_filter.as_ref().or(filter);
        let FilteredRecordKeyPage { record_keys, total } = self
            .index
            .list_filtered_record_keys(filter, sort, limit, offset)?;
        let mut records = self
            .index
            .load_records_by_key_with_options(&record_keys, options)?;
        self.enrich_reference_labels(&mut records)?;
        Ok(FilterOnlyRecordPage {
            record_keys,
            records,
            total,
        })
    }

    pub fn list_filter_fields(
        &self,
        filter: Option<&SearchFilterNode>,
        filter_json: Option<serde_json::Value>,
        force_dynamic: bool,
    ) -> Result<FilterFieldDiscovery, DiscoveryError> {
        self.index
            .list_filter_fields(filter, filter_json, force_dynamic)
    }

    pub fn list_filter_values(
        &self,
        filter: Option<&SearchFilterNode>,
        request: FilterValueRequest,
    ) -> Result<FilterValueDiscovery, DiscoveryError> {
        self.index.list_filter_values(filter, request)
    }

    pub fn variant_group(
        &self,
        record_key: &RecordKey,
    ) -> Result<Option<GraphVariantGroupResult>, SearchError> {
        let Some(seed) = self
            .get_records_with_options(
                std::slice::from_ref(record_key),
                RecordLoadOptions::omit_raw_json(),
            )?
            .into_iter()
            .next()
        else {
            return Ok(None);
        };
        let group = self
            .index
            .variant_group_for_record(record_key)?
            .unwrap_or_else(|| VariantGroup {
                seed: Some(record_key.clone()),
                variant_group_key: None,
                records: Vec::new(),
            });
        Ok(Some(GraphVariantGroupResult {
            seed: Some(seed),
            group,
        }))
    }

    pub fn variant_groups_by_base_name(
        &self,
        base_name: &str,
    ) -> Result<Vec<GraphVariantGroupResult>, SearchError> {
        let normalized_base_name = normalize_record_query(base_name);
        self.index
            .variant_groups_by_base_name(&normalized_base_name)?
            .into_iter()
            .map(|group| Ok(GraphVariantGroupResult { seed: None, group }))
            .collect()
    }

    pub fn remaster_links(
        &self,
        record_key: &RecordKey,
    ) -> Result<Option<GraphRemasterLinksResult>, SearchError> {
        let Some(seed) = self
            .get_records_with_options(
                std::slice::from_ref(record_key),
                RecordLoadOptions::omit_raw_json(),
            )?
            .into_iter()
            .next()
        else {
            return Ok(None);
        };
        let links = self
            .index
            .remaster_links_for_record(record_key)?
            .unwrap_or_else(|| RemasterLinks {
                seed: record_key.clone(),
                links: Vec::new(),
            });
        Ok(Some(GraphRemasterLinksResult { seed, links }))
    }
}

pub(crate) fn trace_search_phase(phase: &str, started_at: Instant) {
    if std::env::var_os("ATLAS_SEARCH_TRACE").is_some() {
        eprintln!(
            "atlas-search trace: {phase} completed in {}ms",
            started_at.elapsed().as_millis()
        );
    }
}

fn load_embedder(embedding_config: &SearchEmbeddingConfig) -> Result<TextEmbedder, SearchError> {
    let model = EmbeddingModelId::from_str(&embedding_config.model_id).map_err(|error| {
        SearchError::InvalidEmbeddingModel {
            model: embedding_config.model_id.clone(),
            message: error.to_string(),
        }
    })?;
    let embedding_config = EmbeddingRuntimeConfig::new(model, &embedding_config.cache_root);
    TextEmbedder::load(&embedding_config).map_err(|error| SearchError::Embedding(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_retrieval_patterns_are_typed_errors() {
        let error = SearchError::UnsupportedRetrievalPattern("record get");

        assert!(matches!(
            error,
            SearchError::UnsupportedRetrievalPattern("record get")
        ));
        assert_eq!(
            error.to_string(),
            "retrieval pattern is not implemented yet: record get"
        );
    }
}
