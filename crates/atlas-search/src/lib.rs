#![deny(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;

use atlas_domain::{FilterFieldDiscovery, FilterValueDiscovery, RecordKey, SearchFilterNode};
use atlas_embedding::{EmbeddingModelId, EmbeddingRuntimeConfig, TextEmbedder};
#[cfg(test)]
use atlas_index::VectorSearchHit;
use atlas_index::{
    DiscoveryError, FilterValueRequest, FilteredRecordKeyPage, FilteredRecordSort,
    FtsColumnWeights, FtsQuery, FtsSearchHit, SqliteIndexReader,
};
use atlas_record::{ContentDocument, PersistedRecord, RecordAlias, visit_content_references_mut};
use serde::{Deserialize, Serialize};

mod fusion;
mod graph_context;
mod query;
mod semantic;
pub use atlas_index::{
    RecordResolutionMatchKind, RecordResolutionResult, SearchError, SearchIndex,
};
#[cfg(test)]
use fusion::classify_fts_match;
pub use fusion::{
    FtsFusionPolicy, FtsMatchConfidence, FusionMethod, FusionOptions, TextSearchExplain,
};
use fusion::{FusionInput, fuse_ranked_hits, identity_explain};
pub use graph_context::{
    GraphContextEdge, GraphContextEdgeSource, GraphContextRequest, GraphContextResult,
    GraphContextSection,
};
pub use query::TextQueryAnalysis;
use query::{analyze_text_query, normalize_record_query};
pub use semantic::{
    SemanticSearchHit, SemanticSearchMode, SemanticSearchResult, SemanticSearchTiming,
};
use semantic::{collapse_vector_hits, semantic_unit_limit};

/// Initial top-level retrieval boundary for Rust runtime consumers.
///
/// The service owns product-facing retrieval entrypoints. Record get/resolve,
/// filter-only list, and ranked FTS/vector/hybrid search should be added here
/// rather than as peer public services.
pub struct AtlasRetrievalService {
    index: Box<dyn SearchIndex>,
    embedder: Option<TextEmbedder>,
}

fn trace_search_phase(phase: &str, started_at: Instant) {
    if std::env::var_os("ATLAS_SEARCH_TRACE").is_some() {
        eprintln!(
            "atlas-search trace: {phase} completed in {}ms",
            started_at.elapsed().as_millis()
        );
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FilterOnlyRecordPage {
    pub record_keys: Vec<RecordKey>,
    pub records: Vec<PersistedRecord>,
    pub total: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RetrievalMode {
    Fts,
    Vector,
    Hybrid,
}

impl RetrievalMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fts => "fts",
            Self::Vector => "vector",
            Self::Hybrid => "hybrid",
        }
    }

    pub(crate) const fn uses_fts(self) -> bool {
        matches!(self, Self::Fts | Self::Hybrid)
    }

    pub(crate) const fn uses_vector(self) -> bool {
        matches!(self, Self::Vector | Self::Hybrid)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSearchRequest<'a> {
    pub query: &'a str,
    pub exclude: Option<&'a str>,
    pub filter: Option<&'a SearchFilterNode>,
    pub limit: u32,
    pub offset: u32,
    pub retrieval: RetrievalMode,
    pub fusion: FusionOptions,
    pub fts_top_k: u32,
    pub vector_top_k: u32,
    pub explain: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSearchPage {
    pub query: TextQueryAnalysis,
    pub retrieval: RetrievalMode,
    pub fusion: FusionOptions,
    pub records: Vec<TextSearchRecord>,
    pub total: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSearchRecord {
    pub record: PersistedRecord,
    pub match_info: TextSearchMatch,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextSearchMatch {
    Identity {
        retrieval: RetrievalMode,
        identity_match_kind: RecordResolutionMatchKind,
        explain: Option<TextSearchExplain>,
    },
    Ranked {
        retrieval: RetrievalMode,
        explain: Option<TextSearchExplain>,
    },
}

#[derive(Debug, Clone)]
pub enum AtlasSearchRequest<'a> {
    Semantic {
        query: &'a str,
        filter: Option<&'a SearchFilterNode>,
        limit: u32,
        mode: SemanticSearchMode,
    },
    FilterOnly {
        filter: Option<&'a SearchFilterNode>,
        limit: u32,
    },
    Text(TextSearchRequest<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AtlasSearchResult {
    Semantic(SemanticSearchResult),
    Text(TextSearchPage),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchEmbeddingConfig {
    pub model_id: String,
    pub cache_root: PathBuf,
}

impl AtlasRetrievalService {
    pub fn new(
        index: SqliteIndexReader,
        embedding_config: &SearchEmbeddingConfig,
    ) -> Result<Self, SearchError> {
        Self::new_with_index(Box::new(index), embedding_config)
    }

    pub fn new_with_index(
        index: Box<dyn SearchIndex>,
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

    pub fn without_embeddings_with_index(index: Box<dyn SearchIndex>) -> Self {
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
        let mut records = self.index.load_records_by_key(record_keys)?;
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

    fn enrich_reference_labels(&self, records: &mut [PersistedRecord]) -> Result<(), SearchError> {
        let mut target_keys = BTreeSet::new();
        for record in records.iter() {
            collect_reference_target_keys(record, &mut target_keys);
        }
        if target_keys.is_empty() {
            return Ok(());
        }

        let requested_keys = records
            .iter()
            .map(|record| record.key.clone())
            .collect::<BTreeSet<_>>();
        let keys_to_load = target_keys
            .into_iter()
            .filter(|key| !requested_keys.contains(key))
            .collect::<Vec<_>>();
        let loaded_targets = self.index.load_records_by_key(&keys_to_load)?;
        let names_by_key = records
            .iter()
            .chain(loaded_targets.iter())
            .map(|record| (record.key.clone(), record.name.clone()))
            .collect::<BTreeMap<_, _>>();

        for record in records {
            apply_reference_target_names(record, &names_by_key);
        }
        Ok(())
    }

    fn enrich_resolution_reference_labels(
        &self,
        matches: &mut [RecordResolutionResult],
    ) -> Result<(), SearchError> {
        let mut target_keys = BTreeSet::new();
        for resolution in matches.iter() {
            collect_reference_target_keys(&resolution.record, &mut target_keys);
        }
        if target_keys.is_empty() {
            return Ok(());
        }

        let requested_keys = matches
            .iter()
            .map(|resolution| resolution.record.key.clone())
            .collect::<BTreeSet<_>>();
        let keys_to_load = target_keys
            .into_iter()
            .filter(|key| !requested_keys.contains(key))
            .collect::<Vec<_>>();
        let loaded_targets = self.index.load_records_by_key(&keys_to_load)?;
        let names_by_key = matches
            .iter()
            .map(|resolution| &resolution.record)
            .chain(loaded_targets.iter())
            .map(|record| (record.key.clone(), record.name.clone()))
            .collect::<BTreeMap<_, _>>();

        for resolution in matches {
            apply_reference_target_names(&mut resolution.record, &names_by_key);
        }
        Ok(())
    }

    fn enrich_text_record_reference_labels(
        &self,
        records: &mut [TextSearchRecord],
    ) -> Result<(), SearchError> {
        let mut target_keys = BTreeSet::new();
        for item in records.iter() {
            collect_reference_target_keys(&item.record, &mut target_keys);
        }
        if target_keys.is_empty() {
            return Ok(());
        }

        let requested_keys = records
            .iter()
            .map(|item| item.record.key.clone())
            .collect::<BTreeSet<_>>();
        let keys_to_load = target_keys
            .into_iter()
            .filter(|key| !requested_keys.contains(key))
            .collect::<Vec<_>>();
        let loaded_targets = self.index.load_records_by_key(&keys_to_load)?;
        let names_by_key = records
            .iter()
            .map(|item| &item.record)
            .chain(loaded_targets.iter())
            .map(|record| (record.key.clone(), record.name.clone()))
            .collect::<BTreeMap<_, _>>();

        for item in records {
            apply_reference_target_names(&mut item.record, &names_by_key);
        }
        Ok(())
    }

    pub fn resolve_record(
        &self,
        query: &str,
        filter: Option<&SearchFilterNode>,
    ) -> Result<Vec<RecordResolutionResult>, SearchError> {
        let resolved_filter = self.index.resolve_metric_filters(filter)?;
        let filter = resolved_filter.as_ref().or(filter);
        let normalized_query = normalize_record_query(query);
        if let Some(mut matches) =
            self.index
                .resolve_record_matches(query, &normalized_query, filter)?
        {
            self.enrich_resolution_reference_labels(&mut matches)?;
            return Ok(matches);
        }
        let mut record_set = self.index.load_record_set()?;
        record_set
            .records
            .retain(|record| record.is_default_visible);
        let default_visible_keys = record_set
            .records
            .iter()
            .map(|record| record.key.clone())
            .collect::<std::collections::BTreeSet<_>>();
        record_set
            .aliases
            .retain(|alias| default_visible_keys.contains(&alias.canonical_record_key));
        if let Some(filter) = filter {
            let allowed = self
                .index
                .list_filtered_record_keys(
                    Some(filter),
                    FilteredRecordSort::RecordKey,
                    u32::MAX,
                    0,
                )?
                .record_keys
                .into_iter()
                .collect::<std::collections::BTreeSet<_>>();
            record_set
                .records
                .retain(|record| allowed.contains(&record.key));
            record_set
                .aliases
                .retain(|alias| allowed.contains(&alias.canonical_record_key));
        }

        let mut matches = resolution_matches_for_kind(
            query,
            &normalized_query,
            RecordResolutionMatchKind::Name,
            &record_set.records,
            &record_set.aliases,
        );
        if matches.is_empty() {
            matches = resolution_matches_for_kind(
                query,
                &normalized_query,
                RecordResolutionMatchKind::NormalizedName,
                &record_set.records,
                &record_set.aliases,
            );
        }
        if matches.is_empty() {
            matches = resolution_matches_for_kind(
                query,
                &normalized_query,
                RecordResolutionMatchKind::Alias,
                &record_set.records,
                &record_set.aliases,
            );
        }
        if matches.is_empty() {
            matches = resolution_matches_for_kind(
                query,
                &normalized_query,
                RecordResolutionMatchKind::VariantName,
                &record_set.records,
                &record_set.aliases,
            );
        }

        self.enrich_resolution_reference_labels(&mut matches)?;
        Ok(matches)
    }

    pub fn filter_only_records(
        &self,
        filter: Option<&SearchFilterNode>,
        sort: FilteredRecordSort,
        limit: u32,
        offset: u32,
    ) -> Result<FilterOnlyRecordPage, SearchError> {
        let resolved_filter = self.index.resolve_metric_filters(filter)?;
        let filter = resolved_filter.as_ref().or(filter);
        let FilteredRecordKeyPage { record_keys, total } = self
            .index
            .list_filtered_record_keys(filter, sort, limit, offset)?;
        let mut records = self.index.load_records_by_key(&record_keys)?;
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

    pub fn text_search(
        &mut self,
        request: TextSearchRequest<'_>,
    ) -> Result<TextSearchPage, SearchError> {
        let total_started_at = Instant::now();
        validate_text_search_request(&request)?;
        trace_search_phase("validate_text_search_request", total_started_at);
        let started_at = Instant::now();
        let resolved_filter = self.index.resolve_metric_filters(request.filter)?;
        let filter = resolved_filter.as_ref().or(request.filter);
        trace_search_phase("resolve_metric_filters", started_at);
        let started_at = Instant::now();
        let query = analyze_text_query(request.query, request.exclude);
        let fts_query = FtsQuery::from_tokens(query.fts_tokens.clone());
        let exclude_query = FtsQuery::from_tokens(query.exclude_tokens.clone());
        trace_search_phase("analyze_text_query", started_at);
        let started_at = Instant::now();
        let identity_matches = self.resolve_record(request.query, filter)?;
        trace_search_phase("resolve_record", started_at);
        let started_at = Instant::now();
        let fts_hits = if request.retrieval.uses_fts() {
            match fts_query.as_ref() {
                Some(fts_query) => self.index.query_fts_index(
                    fts_query,
                    filter,
                    request.fts_top_k,
                    FtsColumnWeights::default(),
                )?,
                None => Vec::new(),
            }
        } else {
            Vec::new()
        };
        trace_search_phase("query_fts_index", started_at);
        let started_at = Instant::now();
        let vector_hits = if request.retrieval.uses_vector() {
            self.semantic(
                request.query,
                filter,
                request.vector_top_k,
                SemanticSearchMode::WeightedChunks,
            )?
        } else {
            Vec::new()
        };
        trace_search_phase("query_vector_index", started_at);
        let started_at = Instant::now();
        let excluded_keys = match exclude_query.as_ref() {
            Some(exclude_query) => self
                .index
                .query_fts_candidate_record_keys(
                    exclude_query,
                    &candidate_keys(&identity_matches, &fts_hits, &vector_hits),
                )?
                .into_iter()
                .collect::<BTreeSet<_>>(),
            None => BTreeSet::new(),
        };
        trace_search_phase("query_excluded_keys", started_at);
        let started_at = Instant::now();
        let identity_matches = identity_matches
            .into_iter()
            .filter(|identity| !excluded_keys.contains(&identity.record.key))
            .collect::<Vec<_>>();
        let identity_keys = identity_matches
            .iter()
            .map(|identity| identity.record.key.clone())
            .collect::<BTreeSet<_>>();
        let ranked_candidate_keys = ranked_candidate_keys(
            &fts_hits,
            &vector_hits,
            &identity_keys,
            &excluded_keys,
            request.retrieval,
        );
        let ranked_records = self.index.load_records_by_key(&ranked_candidate_keys)?;
        let records_by_key = ranked_records
            .into_iter()
            .map(|record| (record.key.clone(), record))
            .collect::<BTreeMap<_, _>>();
        trace_search_phase("load_ranked_candidate_records", started_at);
        let started_at = Instant::now();
        let fused = fuse_ranked_hits(FusionInput {
            fts_hits: &fts_hits,
            vector_hits: &vector_hits,
            records_by_key: &records_by_key,
            fts_tokens: &query.fts_tokens,
            identity_keys: &identity_keys,
            excluded_keys: &excluded_keys,
            retrieval: request.retrieval,
            fusion: request.fusion,
            explain: request.explain,
            identity_count: identity_matches.len(),
        });
        trace_search_phase("fuse_ranked_hits", started_at);
        let total = identity_matches.len() + fused.len();
        let mut all_matches = identity_matches
            .into_iter()
            .enumerate()
            .map(|(index, identity)| TextSearchRecord {
                record: identity.record,
                match_info: TextSearchMatch::Identity {
                    retrieval: request.retrieval,
                    identity_match_kind: identity.match_kind,
                    explain: request.explain.then(|| identity_explain(index)),
                },
            })
            .collect::<Vec<_>>();

        let started_at = Instant::now();
        all_matches.extend(fused.into_iter().filter_map(|ranked| {
            records_by_key
                .get(&ranked.record_key)
                .map(|record| TextSearchRecord {
                    record: record.clone(),
                    match_info: TextSearchMatch::Ranked {
                        retrieval: request.retrieval,
                        explain: ranked.explain,
                    },
                })
        }));

        let mut page_records = all_matches
            .into_iter()
            .skip(request.offset as usize)
            .take(request.limit as usize)
            .collect::<Vec<_>>();
        trace_search_phase("build_page_records", started_at);
        let started_at = Instant::now();
        self.enrich_text_record_reference_labels(&mut page_records)?;
        trace_search_phase("enrich_text_record_reference_labels", started_at);
        trace_search_phase("text_search_total", total_started_at);

        Ok(TextSearchPage {
            query,
            retrieval: request.retrieval,
            fusion: request.fusion,
            records: page_records,
            total: total as u64,
        })
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

fn validate_text_search_request(request: &TextSearchRequest<'_>) -> Result<(), SearchError> {
    if let Some(filter) = request.filter {
        filter
            .validate()
            .map_err(|error| SearchError::InvalidSearchOptions(error.to_string()))?;
    }
    if request.fusion.method == FusionMethod::Rrf
        && ((request.fusion.fts_weight - 1.0).abs() > f64::EPSILON
            || (request.fusion.vector_weight - 1.0).abs() > f64::EPSILON)
    {
        return Err(SearchError::InvalidSearchOptions(
            "unweighted rrf does not accept lane weights; use weighted-rrf".to_string(),
        ));
    }
    if request.fusion.rank_constant <= 0.0
        || request.fusion.fts_weight < 0.0
        || request.fusion.vector_weight < 0.0
    {
        return Err(SearchError::InvalidSearchOptions(
            "fusion weights must be non-negative and rank constant must be greater than zero"
                .to_string(),
        ));
    }
    Ok(())
}

fn candidate_keys(
    identity_matches: &[RecordResolutionResult],
    fts_hits: &[FtsSearchHit],
    vector_hits: &[SemanticSearchHit],
) -> Vec<RecordKey> {
    let mut keys = identity_matches
        .iter()
        .map(|identity| identity.record.key.clone())
        .collect::<BTreeSet<_>>();
    keys.extend(fts_hits.iter().map(|hit| hit.record_key.clone()));
    keys.extend(
        vector_hits
            .iter()
            .filter_map(|hit| RecordKey::parse(&hit.record_key).ok()),
    );
    keys.into_iter().collect()
}

fn ranked_candidate_keys(
    fts_hits: &[FtsSearchHit],
    vector_hits: &[SemanticSearchHit],
    identity_keys: &BTreeSet<RecordKey>,
    excluded_keys: &BTreeSet<RecordKey>,
    retrieval: RetrievalMode,
) -> Vec<RecordKey> {
    let mut keys = BTreeSet::new();
    if retrieval.uses_fts() {
        keys.extend(fts_hits.iter().map(|hit| hit.record_key.clone()));
    }
    if retrieval.uses_vector() {
        keys.extend(
            vector_hits
                .iter()
                .filter_map(|hit| RecordKey::parse(&hit.record_key).ok()),
        );
    }
    keys.retain(|key| !identity_keys.contains(key) && !excluded_keys.contains(key));
    keys.into_iter().collect()
}

fn resolution_matches_for_kind(
    query: &str,
    normalized_query: &str,
    kind: RecordResolutionMatchKind,
    records: &[PersistedRecord],
    aliases: &[RecordAlias],
) -> Vec<RecordResolutionResult> {
    let mut matches = Vec::new();
    match kind {
        RecordResolutionMatchKind::Name => {
            matches.extend(
                records
                    .iter()
                    .filter(|record| record.name == query)
                    .map(|record| {
                        resolution_result(
                            query,
                            normalized_query,
                            kind,
                            record.name.clone(),
                            None,
                            record,
                        )
                    }),
            );
        }
        RecordResolutionMatchKind::NormalizedName => {
            matches.extend(
                records
                    .iter()
                    .filter(|record| {
                        record.variant_label.is_none() && record.normalized_name == normalized_query
                    })
                    .map(|record| {
                        resolution_result(
                            query,
                            normalized_query,
                            kind,
                            record.normalized_name.clone(),
                            None,
                            record,
                        )
                    }),
            );
        }
        RecordResolutionMatchKind::Alias => {
            for alias in aliases
                .iter()
                .filter(|alias| alias.normalized_alias == normalized_query)
            {
                matches.extend(
                    records
                        .iter()
                        .filter(|record| record.key == alias.canonical_record_key)
                        .map(|record| {
                            resolution_result(
                                query,
                                normalized_query,
                                kind,
                                alias.alias_text.clone(),
                                Some(alias),
                                record,
                            )
                        }),
                );
            }
        }
        RecordResolutionMatchKind::VariantName => {
            matches.extend(
                records
                    .iter()
                    .filter(|record| {
                        record.variant_label.is_some() && record.normalized_name == normalized_query
                    })
                    .map(|record| {
                        resolution_result(
                            query,
                            normalized_query,
                            kind,
                            record.normalized_name.clone(),
                            None,
                            record,
                        )
                    }),
            );
        }
    }
    matches.sort_by(|left, right| left.record.key.cmp(&right.record.key));
    matches
}

fn resolution_result(
    query: &str,
    normalized_query: &str,
    match_kind: RecordResolutionMatchKind,
    matched_text: String,
    alias: Option<&RecordAlias>,
    record: &PersistedRecord,
) -> RecordResolutionResult {
    RecordResolutionResult {
        query: query.to_string(),
        normalized_query: normalized_query.to_string(),
        match_kind,
        matched_text,
        alias_source: alias.map(|alias| alias.source.as_str().to_string()),
        alias_source_ref: alias.map(|alias| alias.source_ref.clone()),
        record: record.clone(),
    }
}

fn collect_reference_target_keys(record: &PersistedRecord, target_keys: &mut BTreeSet<RecordKey>) {
    if let Some(document) = &record.description {
        collect_document_reference_target_keys(document, target_keys);
    }
    if let Some(document) = &record.blurb {
        collect_document_reference_target_keys(document, target_keys);
    }
    for supplemental in &record.supplemental_content {
        collect_document_reference_target_keys(&supplemental.document, target_keys);
    }
}

fn collect_document_reference_target_keys(
    document: &ContentDocument,
    target_keys: &mut BTreeSet<RecordKey>,
) {
    for reference in atlas_record::iter_content_references(document) {
        if let Some(record_key) = &reference.resolved_key
            && reference.resolved_name.is_none()
        {
            target_keys.insert(record_key.clone());
        }
    }
}

fn apply_reference_target_names(
    record: &mut PersistedRecord,
    names_by_key: &BTreeMap<RecordKey, String>,
) {
    if let Some(document) = &mut record.description {
        apply_document_reference_target_names(document, names_by_key);
    }
    if let Some(document) = &mut record.blurb {
        apply_document_reference_target_names(document, names_by_key);
    }
    for supplemental in &mut record.supplemental_content {
        apply_document_reference_target_names(&mut supplemental.document, names_by_key);
    }
}

fn apply_document_reference_target_names(
    document: &mut ContentDocument,
    names_by_key: &BTreeMap<RecordKey, String>,
) {
    visit_content_references_mut(document, |reference| {
        if reference.resolved_name.is_none()
            && let Some(record_key) = &reference.resolved_key
            && let Some(name) = names_by_key.get(record_key)
        {
            reference.resolved_name = Some(name.clone());
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_domain::{PublicationFamily, RecordFamily};
    use atlas_record::{ContentBlock, ContentInline, ContentReference, ContentReferenceLocator};

    fn hit(unit: &str, record: &str, unit_kind: &str, distance: f64) -> VectorSearchHit {
        VectorSearchHit {
            embedding_unit_key: unit.to_string(),
            record_key: record.to_string(),
            unit_kind: unit_kind.to_string(),
            label: None,
            distance,
        }
    }

    fn test_record(key: &str, name: &str, traits: &[&str]) -> PersistedRecord {
        let key = RecordKey::parse(key).expect("record key parses");
        PersistedRecord {
            id: key.id().clone(),
            pack_name: key.pack().clone(),
            key,
            name: name.to_string(),
            normalized_name: normalize_record_query(name),
            record_family: RecordFamily::Feat,
            pack_label: "Test Pack".to_string(),
            foundry_document_type: "Item".to_string(),
            foundry_record_type: "feat".to_string(),
            level: None,
            rarity: None,
            traits: traits.iter().map(|value| value.to_string()).collect(),
            prerequisites: Vec::new(),
            system_category: None,
            system_group: None,
            system_base_item: None,
            system_usage: None,
            system_price_json: None,
            system_actions_value: None,
            system_time_value: None,
            system_duration_value: None,
            price_cp: None,
            activation_time: None,
            duration: None,
            metrics: Vec::new(),
            actor_data: None,
            item_data: None,
            spell_data: None,
            publication_title: None,
            publication_remaster: false,
            description: None,
            blurb: None,
            supplemental_content: Vec::new(),
            publication_family: PublicationFamily::Unknown,
            folder_id: None,
            taxonomy_families: Vec::new(),
            variant_group_key: None,
            variant_base_name: None,
            variant_label: None,
            variant_axes: Vec::new(),
            variant_confidence: None,
            variant_source: "test".to_string(),
            source_path: "test.json".to_string(),
            is_default_visible: true,
            raw_json: "{}".to_string(),
        }
    }

    #[test]
    fn reference_label_enrichment_uses_loaded_target_names() {
        let target_key = RecordKey::parse("feats-srd:UKXaMhb9qlPYw1HD").expect("key parses");
        let mut document = ContentDocument::new(vec![ContentBlock::Paragraph {
            content: vec![ContentInline::Reference {
                reference: ContentReference {
                    label: None,
                    locator: ContentReferenceLocator::FoundryUuid {
                        raw_target: "Compendium.pf2e.feats-srd.Item.UKXaMhb9qlPYw1HD".to_string(),
                    },
                    resolved_key: Some(target_key.clone()),
                    resolved_name: None,
                },
            }],
        }]);
        let mut target_keys = BTreeSet::new();
        collect_document_reference_target_keys(&document, &mut target_keys);

        assert_eq!(target_keys, BTreeSet::from([target_key.clone()]));

        let names_by_key = BTreeMap::from([(target_key, "Guardian's Deflection".to_string())]);
        apply_document_reference_target_names(&mut document, &names_by_key);

        assert_eq!(
            atlas_record::render_markdown_like(&document),
            "[Guardian's Deflection](record:feats-srd:UKXaMhb9qlPYw1HD)"
        );
    }

    #[test]
    fn collapse_vector_hits_keeps_one_unit_per_record() {
        let collapsed = collapse_vector_hits(
            vec![
                hit("records:a#parent", "records:a", "parent", 0.1),
                hit(
                    "records:a#heading_section:1",
                    "records:a",
                    "heading_section",
                    0.2,
                ),
                hit("records:b#parent", "records:b", "parent", 0.3),
                hit("records:c#parent", "records:c", "parent", 0.4),
            ],
            2,
            SemanticSearchMode::WeightedChunks,
        );

        assert_eq!(
            collapsed
                .iter()
                .map(|hit| (hit.embedding_unit_key.as_str(), hit.record_key.as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("records:a#parent", "records:a"),
                ("records:b#parent", "records:b"),
            ]
        );
    }

    #[test]
    fn collapse_vector_hits_allows_much_closer_child_to_recover_record() {
        let collapsed = collapse_vector_hits(
            vec![
                hit(
                    "records:a#heading_section:1",
                    "records:a",
                    "heading_section",
                    0.100,
                ),
                hit("records:a#parent", "records:a", "parent", 0.200),
            ],
            10,
            SemanticSearchMode::WeightedChunks,
        );

        assert_eq!(
            collapsed[0].embedding_unit_key,
            "records:a#heading_section:1"
        );
        assert_eq!(collapsed[0].distance, 0.100);
        assert_eq!(collapsed[0].rank_distance, 0.125);
    }

    #[test]
    fn collapse_vector_hits_penalizes_records_without_parent_hit() {
        let collapsed = collapse_vector_hits(
            vec![
                hit(
                    "records:a#heading_section:1",
                    "records:a",
                    "heading_section",
                    0.100,
                ),
                hit("records:b#parent", "records:b", "parent", 0.145),
            ],
            10,
            SemanticSearchMode::WeightedChunks,
        );

        assert_eq!(collapsed[0].embedding_unit_key, "records:b#parent");
        assert_eq!(collapsed[0].rank_distance, 0.145);
        assert_eq!(
            collapsed[1].embedding_unit_key,
            "records:a#heading_section:1"
        );
        assert_eq!(collapsed[1].rank_distance, 0.150);
    }

    #[test]
    fn collapse_vector_hits_can_rank_chunks_without_unit_weights() {
        let collapsed = collapse_vector_hits(
            vec![
                hit(
                    "records:a#heading_section:1",
                    "records:a",
                    "heading_section",
                    0.100,
                ),
                hit("records:a#parent", "records:a", "parent", 0.120),
            ],
            10,
            SemanticSearchMode::Chunks,
        );

        assert_eq!(
            collapsed[0].embedding_unit_key,
            "records:a#heading_section:1"
        );
        assert_eq!(collapsed[0].rank_distance, 0.100);
    }

    #[test]
    fn fts_query_analysis_uses_safe_or_tokens_without_domain_derivation() {
        let analysis = analyze_text_query("monster that breathes fire", Some("water"));

        assert_eq!(analysis.fts_tokens, vec!["monster", "breathes", "fire"]);
        assert_eq!(
            analysis.fts_query.as_deref(),
            Some("\"monster\" OR \"breathes\" OR \"fire\"")
        );
        assert_eq!(analysis.exclude_tokens, vec!["water"]);
        assert_eq!(analysis.exclude_query.as_deref(), Some("\"water\""));
    }

    #[test]
    fn weighted_rrf_combines_lanes_and_excludes_identity_matches() {
        let fts_hits = vec![
            FtsSearchHit {
                record_key: RecordKey::parse("records:a").unwrap(),
                rank: -2.0,
            },
            FtsSearchHit {
                record_key: RecordKey::parse("records:b").unwrap(),
                rank: -1.0,
            },
        ];
        let vector_hits = vec![
            SemanticSearchHit {
                record_key: "records:b".to_string(),
                embedding_unit_key: "records:b#parent".to_string(),
                unit_kind: "parent".to_string(),
                label: None,
                distance: 0.1,
                rank_distance: 0.1,
            },
            SemanticSearchHit {
                record_key: "records:c".to_string(),
                embedding_unit_key: "records:c#parent".to_string(),
                unit_kind: "parent".to_string(),
                label: None,
                distance: 0.2,
                rank_distance: 0.2,
            },
        ];
        let identity_keys = [RecordKey::parse("records:a").unwrap()]
            .into_iter()
            .collect::<BTreeSet<_>>();
        let excluded_keys = BTreeSet::new();
        let records_by_key = BTreeMap::from([
            (
                RecordKey::parse("records:b").unwrap(),
                test_record("records:b", "Battle Medicine", &["healing"]),
            ),
            (
                RecordKey::parse("records:c").unwrap(),
                test_record("records:c", "Risky Surgery", &[]),
            ),
        ]);

        let fused = fuse_ranked_hits(FusionInput {
            fts_hits: &fts_hits,
            vector_hits: &vector_hits,
            records_by_key: &records_by_key,
            fts_tokens: &["battle".to_string(), "medicine".to_string()],
            identity_keys: &identity_keys,
            excluded_keys: &excluded_keys,
            retrieval: RetrievalMode::Hybrid,
            fusion: FusionOptions::default(),
            explain: true,
            identity_count: 1,
        });

        assert_eq!(
            fused
                .iter()
                .map(|hit| hit.record_key.to_string())
                .collect::<Vec<_>>(),
            vec!["records:b", "records:c"]
        );
        assert_eq!(fused[0].explain.as_ref().unwrap().rank, 2);
        assert_eq!(fused[0].explain.as_ref().unwrap().fts_rank, Some(2));
        assert_eq!(fused[0].explain.as_ref().unwrap().vector_rank, Some(1));
    }

    #[test]
    fn min_max_score_fusion_uses_lane_scores_and_weights() {
        let fts_hits = vec![
            FtsSearchHit {
                record_key: RecordKey::parse("records:a").unwrap(),
                rank: -2.0,
            },
            FtsSearchHit {
                record_key: RecordKey::parse("records:b").unwrap(),
                rank: -1.0,
            },
        ];
        let vector_hits = vec![
            SemanticSearchHit {
                record_key: "records:b".to_string(),
                embedding_unit_key: "records:b#parent".to_string(),
                unit_kind: "parent".to_string(),
                label: None,
                distance: 0.1,
                rank_distance: 0.1,
            },
            SemanticSearchHit {
                record_key: "records:c".to_string(),
                embedding_unit_key: "records:c#parent".to_string(),
                unit_kind: "parent".to_string(),
                label: None,
                distance: 0.2,
                rank_distance: 0.2,
            },
        ];
        let records_by_key = BTreeMap::from([
            (
                RecordKey::parse("records:a").unwrap(),
                test_record("records:a", "Direct Result", &[]),
            ),
            (
                RecordKey::parse("records:b").unwrap(),
                test_record("records:b", "Shared Result", &[]),
            ),
            (
                RecordKey::parse("records:c").unwrap(),
                test_record("records:c", "Semantic Result", &[]),
            ),
        ]);

        let fused = fuse_ranked_hits(FusionInput {
            fts_hits: &fts_hits,
            vector_hits: &vector_hits,
            records_by_key: &records_by_key,
            fts_tokens: &["result".to_string()],
            identity_keys: &BTreeSet::new(),
            excluded_keys: &BTreeSet::new(),
            retrieval: RetrievalMode::Hybrid,
            fusion: FusionOptions {
                method: FusionMethod::MinMaxScore,
                fts_weight: 1.0,
                vector_weight: 2.0,
                rank_constant: 60.0,
                fts_policy: FtsFusionPolicy::All,
            },
            explain: true,
            identity_count: 0,
        });

        assert_eq!(
            fused
                .iter()
                .map(|hit| hit.record_key.to_string())
                .collect::<Vec<_>>(),
            vec!["records:b", "records:a", "records:c"]
        );
        assert_eq!(fused[0].explain.as_ref().unwrap().fused_score, Some(2.0));
        assert_eq!(fused[1].explain.as_ref().unwrap().fused_score, Some(1.0));
    }

    #[test]
    fn fts_confidence_distinguishes_direct_and_weak_hits() {
        let direct = test_record("records:a", "Treat Wounds", &["healing"]);
        let strong = test_record("records:b", "Battle Medicine", &["healing", "manipulate"]);
        let weak = test_record("records:c", "Shielded Arm", &["metal"]);

        assert_eq!(
            classify_fts_match(&direct, &["treat".to_string(), "wounds".to_string()]),
            FtsMatchConfidence::DirectTitle
        );
        assert_eq!(
            classify_fts_match(&strong, &["healing".to_string(), "manipulate".to_string()]),
            FtsMatchConfidence::StrongLexical
        );
        assert_eq!(
            classify_fts_match(
                &weak,
                &[
                    "low".to_string(),
                    "level".to_string(),
                    "fear".to_string(),
                    "spell".to_string()
                ],
            ),
            FtsMatchConfidence::WeakLexical
        );
    }

    #[test]
    fn fts_fusion_policy_can_zero_weak_hits() {
        let fts_hits = vec![
            FtsSearchHit {
                record_key: RecordKey::parse("records:weak").unwrap(),
                rank: 10.0,
            },
            FtsSearchHit {
                record_key: RecordKey::parse("records:strong").unwrap(),
                rank: 8.0,
            },
        ];
        let vector_hits = vec![SemanticSearchHit {
            record_key: "records:semantic".to_string(),
            embedding_unit_key: "records:semantic#parent".to_string(),
            unit_kind: "parent".to_string(),
            label: None,
            distance: 0.1,
            rank_distance: 0.1,
        }];
        let records_by_key = BTreeMap::from([
            (
                RecordKey::parse("records:weak").unwrap(),
                test_record("records:weak", "Shielded Arm", &["metal"]),
            ),
            (
                RecordKey::parse("records:strong").unwrap(),
                test_record("records:strong", "Fear", &["fear"]),
            ),
            (
                RecordKey::parse("records:semantic").unwrap(),
                test_record("records:semantic", "Semantic Fear Result", &["fear"]),
            ),
        ]);

        let all = fuse_ranked_hits(FusionInput {
            fts_hits: &fts_hits,
            vector_hits: &vector_hits,
            records_by_key: &records_by_key,
            fts_tokens: &[
                "low".to_string(),
                "level".to_string(),
                "fear".to_string(),
                "spell".to_string(),
            ],
            identity_keys: &BTreeSet::new(),
            excluded_keys: &BTreeSet::new(),
            retrieval: RetrievalMode::Hybrid,
            fusion: FusionOptions {
                method: FusionMethod::MinMaxScore,
                fts_weight: 1.0,
                vector_weight: 1.0,
                rank_constant: 60.0,
                fts_policy: FtsFusionPolicy::All,
            },
            explain: true,
            identity_count: 0,
        });
        let strong_only = fuse_ranked_hits(FusionInput {
            fts_hits: &fts_hits,
            vector_hits: &vector_hits,
            records_by_key: &records_by_key,
            fts_tokens: &[
                "low".to_string(),
                "level".to_string(),
                "fear".to_string(),
                "spell".to_string(),
            ],
            identity_keys: &BTreeSet::new(),
            excluded_keys: &BTreeSet::new(),
            retrieval: RetrievalMode::Hybrid,
            fusion: FusionOptions {
                method: FusionMethod::MinMaxScore,
                fts_weight: 1.0,
                vector_weight: 1.0,
                rank_constant: 60.0,
                fts_policy: FtsFusionPolicy::StrongOnly,
            },
            explain: true,
            identity_count: 0,
        });

        assert_eq!(all[0].record_key.to_string(), "records:weak");
        assert_eq!(strong_only[0].record_key.to_string(), "records:semantic");
        assert!(
            !strong_only
                .iter()
                .any(|hit| hit.record_key.to_string() == "records:weak")
        );
    }

    #[test]
    fn unweighted_rrf_rejects_lane_weights_at_runtime_boundary() {
        let request = TextSearchRequest {
            query: "healing",
            exclude: None,
            filter: None,
            limit: 10,
            offset: 0,
            retrieval: RetrievalMode::Fts,
            fusion: FusionOptions {
                method: FusionMethod::Rrf,
                fts_weight: 2.0,
                vector_weight: 1.0,
                rank_constant: 60.0,
                fts_policy: FtsFusionPolicy::All,
            },
            fts_top_k: 10,
            vector_top_k: 10,
            explain: false,
        };

        assert!(matches!(
            validate_text_search_request(&request),
            Err(SearchError::InvalidSearchOptions(_))
        ));
    }

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
