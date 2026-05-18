#![deny(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;

use atlas_domain::{RecordKey, SearchFilterNode};
use atlas_embedding::{EmbeddingModelId, EmbeddingRuntimeConfig, EmbeddingUnitKind, TextEmbedder};
use atlas_index::{
    AtlasIndex, FilterCompileError, FilteredRecordKeyPage, FilteredRecordSort, FtsColumnWeights,
    FtsQuery, FtsSearchHit, IndexValidationError, RecordLoadError, VectorQueryError,
    VectorSearchHit,
};
use atlas_record::{ContentDocument, PersistedRecord, RecordAlias, visit_content_references_mut};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Initial top-level retrieval boundary for Rust runtime consumers.
///
/// The service owns product-facing retrieval entrypoints. Record get/resolve,
/// filter-only list, and ranked FTS/vector/hybrid search should be added here
/// rather than as peer public services.
pub struct AtlasRetrievalService {
    index: AtlasIndex,
    embedder: Option<TextEmbedder>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SemanticSearchMode {
    ParentOnly,
    Chunks,
    WeightedChunks,
}

impl SemanticSearchMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ParentOnly => "parent-only",
            Self::Chunks => "chunks",
            Self::WeightedChunks => "weighted-chunks",
        }
    }

    const fn includes_child_units(self) -> bool {
        !matches!(self, Self::ParentOnly)
    }

    const fn uses_rank_weights(self) -> bool {
        matches!(self, Self::WeightedChunks)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticSearchHit {
    pub record_key: String,
    pub embedding_unit_key: String,
    pub unit_kind: String,
    pub label: Option<String>,
    pub distance: f64,
    pub rank_distance: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticSearchTiming {
    pub query_embedding_duration_ms: u128,
    pub vector_search_duration_ms: u128,
    pub total_duration_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticSearchResult {
    pub hits: Vec<SemanticSearchHit>,
    pub timing: SemanticSearchTiming,
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

    const fn uses_fts(self) -> bool {
        matches!(self, Self::Fts | Self::Hybrid)
    }

    const fn uses_vector(self) -> bool {
        matches!(self, Self::Vector | Self::Hybrid)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FusionMethod {
    Rrf,
    WeightedRrf,
}

impl FusionMethod {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Rrf => "rrf",
            Self::WeightedRrf => "weighted-rrf",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FusionOptions {
    pub method: FusionMethod,
    pub fts_weight: f64,
    pub vector_weight: f64,
    pub rank_constant: f64,
}

impl Default for FusionOptions {
    fn default() -> Self {
        Self {
            method: FusionMethod::WeightedRrf,
            fts_weight: 1.0,
            vector_weight: 1.0,
            rank_constant: 60.0,
        }
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
pub struct TextQueryAnalysis {
    pub normalized_query: String,
    pub fts_query: Option<String>,
    pub fts_tokens: Vec<String>,
    pub exclude_query: Option<String>,
    pub exclude_tokens: Vec<String>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct TextSearchExplain {
    pub rank: u32,
    pub fused_score: Option<f64>,
    pub fts_rank: Option<u32>,
    pub fts_score: Option<f64>,
    pub vector_rank: Option<u32>,
    pub vector_distance: Option<f64>,
    pub vector_rank_distance: Option<f64>,
    pub vector_unit_kind: Option<String>,
    pub vector_label: Option<String>,
    pub vector_embedding_unit_key: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchEmbeddingConfig {
    pub model_id: String,
    pub cache_root: PathBuf,
}

impl AtlasRetrievalService {
    pub fn new(
        index: AtlasIndex,
        embedding_config: &SearchEmbeddingConfig,
    ) -> Result<Self, SearchError> {
        index.validate_vector_index()?;
        Ok(Self {
            embedder: Some(load_embedder(embedding_config)?),
            index,
        })
    }

    pub fn without_embeddings(index: AtlasIndex) -> Self {
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
        let normalized_query = normalize_record_query(query);
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

    pub fn text_search(
        &mut self,
        request: TextSearchRequest<'_>,
    ) -> Result<TextSearchPage, SearchError> {
        validate_text_search_request(&request)?;
        let query = analyze_text_query(request.query, request.exclude);
        let fts_query = FtsQuery::from_tokens(query.fts_tokens.clone());
        let exclude_query = FtsQuery::from_tokens(query.exclude_tokens.clone());
        let identity_matches = self.resolve_record(request.query, request.filter)?;
        let fts_hits = if request.retrieval.uses_fts() {
            match fts_query.as_ref() {
                Some(fts_query) => self.index.query_fts_index(
                    fts_query,
                    request.filter,
                    request.fts_top_k,
                    FtsColumnWeights::default(),
                )?,
                None => Vec::new(),
            }
        } else {
            Vec::new()
        };
        let vector_hits = if request.retrieval.uses_vector() {
            self.semantic(
                request.query,
                request.filter,
                request.vector_top_k,
                SemanticSearchMode::WeightedChunks,
            )?
        } else {
            Vec::new()
        };
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
        let identity_matches = identity_matches
            .into_iter()
            .filter(|identity| !excluded_keys.contains(&identity.record.key))
            .collect::<Vec<_>>();
        let identity_keys = identity_matches
            .iter()
            .map(|identity| identity.record.key.clone())
            .collect::<BTreeSet<_>>();
        let fused = fuse_ranked_hits(FusionInput {
            fts_hits: &fts_hits,
            vector_hits: &vector_hits,
            identity_keys: &identity_keys,
            excluded_keys: &excluded_keys,
            retrieval: request.retrieval,
            fusion: request.fusion,
            explain: request.explain,
            identity_count: identity_matches.len(),
        });
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

        let needed_keys = fused
            .iter()
            .map(|ranked| ranked.record_key.clone())
            .collect::<Vec<_>>();
        let records = self.index.load_records_by_key(&needed_keys)?;
        let by_key = records
            .into_iter()
            .map(|record| (record.key.clone(), record))
            .collect::<BTreeMap<_, _>>();
        all_matches.extend(fused.into_iter().filter_map(|ranked| {
            by_key
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
        self.enrich_text_record_reference_labels(&mut page_records)?;

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

fn semantic_unit_limit(limit: u32, mode: SemanticSearchMode) -> u32 {
    if mode.includes_child_units() {
        limit.saturating_mul(20).max(limit).min(1000)
    } else {
        limit
    }
}

fn normalize_record_query(value: &str) -> String {
    value
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn analyze_text_query(query: &str, exclude: Option<&str>) -> TextQueryAnalysis {
    let normalized_query = normalize_record_query(query);
    let fts_tokens = tokenize_fts_query(query);
    let exclude_tokens = exclude.map(tokenize_fts_query).unwrap_or_default();
    let fts_query = FtsQuery::from_tokens(fts_tokens.clone()).map(|query| query.as_match_query());
    let exclude_query =
        FtsQuery::from_tokens(exclude_tokens.clone()).map(|query| query.as_match_query());
    TextQueryAnalysis {
        normalized_query,
        fts_query,
        fts_tokens,
        exclude_query,
        exclude_tokens,
    }
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

fn tokenize_fts_query(query: &str) -> Vec<String> {
    query
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .filter(|token| !is_fts_stop_word(token))
        .map(ToString::to_string)
        .collect()
}

fn is_fts_stop_word(token: &str) -> bool {
    matches!(
        token,
        "a" | "an"
            | "and"
            | "are"
            | "as"
            | "at"
            | "be"
            | "by"
            | "for"
            | "from"
            | "in"
            | "is"
            | "of"
            | "on"
            | "or"
            | "that"
            | "the"
            | "to"
            | "which"
            | "with"
    )
}

#[derive(Debug, Clone, PartialEq)]
struct FusedRankedHit {
    record_key: RecordKey,
    explain: Option<TextSearchExplain>,
}

#[derive(Debug, Clone, PartialEq)]
struct FusionAccumulator {
    record_key: RecordKey,
    fused_score: f64,
    fts_rank: Option<u32>,
    fts_score: Option<f64>,
    vector_rank: Option<u32>,
    vector_distance: Option<f64>,
    vector_rank_distance: Option<f64>,
    vector_unit_kind: Option<String>,
    vector_label: Option<String>,
    vector_embedding_unit_key: Option<String>,
}

struct FusionInput<'a> {
    fts_hits: &'a [FtsSearchHit],
    vector_hits: &'a [SemanticSearchHit],
    identity_keys: &'a BTreeSet<RecordKey>,
    excluded_keys: &'a BTreeSet<RecordKey>,
    retrieval: RetrievalMode,
    fusion: FusionOptions,
    explain: bool,
    identity_count: usize,
}

fn fuse_ranked_hits(input: FusionInput<'_>) -> Vec<FusedRankedHit> {
    let mut by_key = BTreeMap::<RecordKey, FusionAccumulator>::new();
    if input.retrieval.uses_fts() {
        for (index, hit) in input.fts_hits.iter().enumerate() {
            if input.identity_keys.contains(&hit.record_key)
                || input.excluded_keys.contains(&hit.record_key)
            {
                continue;
            }
            let rank = (index + 1) as u32;
            let entry = by_key
                .entry(hit.record_key.clone())
                .or_insert_with(|| FusionAccumulator::new(hit.record_key.clone()));
            entry.fts_rank = Some(rank);
            entry.fts_score = Some(hit.rank);
            entry.fused_score +=
                lane_rrf_score(rank, input.fusion.rank_constant, input.fusion.fts_weight);
        }
    }
    if input.retrieval.uses_vector() {
        for (index, hit) in input.vector_hits.iter().enumerate() {
            let record_key = match RecordKey::parse(&hit.record_key) {
                Ok(record_key) => record_key,
                Err(_) => continue,
            };
            if input.identity_keys.contains(&record_key)
                || input.excluded_keys.contains(&record_key)
            {
                continue;
            }
            let rank = (index + 1) as u32;
            let entry = by_key
                .entry(record_key.clone())
                .or_insert_with(|| FusionAccumulator::new(record_key));
            entry.vector_rank = Some(rank);
            entry.vector_distance = Some(hit.distance);
            entry.vector_rank_distance = Some(hit.rank_distance);
            entry.vector_unit_kind = Some(hit.unit_kind.clone());
            entry.vector_label = hit.label.clone();
            entry.vector_embedding_unit_key = Some(hit.embedding_unit_key.clone());
            entry.fused_score +=
                lane_rrf_score(rank, input.fusion.rank_constant, input.fusion.vector_weight);
        }
    }

    let mut fused = by_key.into_values().collect::<Vec<_>>();
    fused.sort_by(compare_fused_hits);
    fused
        .into_iter()
        .enumerate()
        .map(|(index, hit)| FusedRankedHit {
            record_key: hit.record_key.clone(),
            explain: input.explain.then(|| TextSearchExplain {
                rank: (input.identity_count + index + 1) as u32,
                fused_score: Some(hit.fused_score),
                fts_rank: hit.fts_rank,
                fts_score: hit.fts_score,
                vector_rank: hit.vector_rank,
                vector_distance: hit.vector_distance,
                vector_rank_distance: hit.vector_rank_distance,
                vector_unit_kind: hit.vector_unit_kind,
                vector_label: hit.vector_label,
                vector_embedding_unit_key: hit.vector_embedding_unit_key,
            }),
        })
        .collect()
}

fn identity_explain(index: usize) -> TextSearchExplain {
    TextSearchExplain {
        rank: (index + 1) as u32,
        fused_score: None,
        fts_rank: None,
        fts_score: None,
        vector_rank: None,
        vector_distance: None,
        vector_rank_distance: None,
        vector_unit_kind: None,
        vector_label: None,
        vector_embedding_unit_key: None,
    }
}

impl FusionAccumulator {
    fn new(record_key: RecordKey) -> Self {
        Self {
            record_key,
            fused_score: 0.0,
            fts_rank: None,
            fts_score: None,
            vector_rank: None,
            vector_distance: None,
            vector_rank_distance: None,
            vector_unit_kind: None,
            vector_label: None,
            vector_embedding_unit_key: None,
        }
    }
}

fn lane_rrf_score(rank: u32, rank_constant: f64, weight: f64) -> f64 {
    weight / (rank_constant + f64::from(rank))
}

fn compare_fused_hits(left: &FusionAccumulator, right: &FusionAccumulator) -> std::cmp::Ordering {
    right
        .fused_score
        .total_cmp(&left.fused_score)
        .then_with(|| compare_optional_rank(left.fts_rank, right.fts_rank))
        .then_with(|| compare_optional_rank(left.vector_rank, right.vector_rank))
        .then_with(|| left.record_key.cmp(&right.record_key))
}

fn compare_optional_rank(left: Option<u32>, right: Option<u32>) -> std::cmp::Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
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

fn collapse_vector_hits(
    rows: Vec<VectorSearchHit>,
    limit: usize,
    mode: SemanticSearchMode,
) -> Vec<SemanticSearchHit> {
    let mut grouped = std::collections::BTreeMap::<String, Vec<VectorSearchHit>>::new();
    for hit in rows {
        grouped.entry(hit.record_key.clone()).or_default().push(hit);
    }
    let mut collapsed = grouped
        .into_values()
        .filter_map(|hits| best_record_hit(hits, mode))
        .collect::<Vec<_>>();
    collapsed.sort_by(compare_semantic_hits_for_rank);
    collapsed.truncate(limit);
    collapsed
}

fn best_record_hit(
    hits: Vec<VectorSearchHit>,
    mode: SemanticSearchMode,
) -> Option<SemanticSearchHit> {
    let has_parent = hits
        .iter()
        .any(|hit| parsed_unit_kind(hit) == Some(EmbeddingUnitKind::Parent));
    hits.into_iter()
        .map(|hit| {
            let rank_distance = rank_distance(&hit, has_parent, mode);
            semantic_hit_from_vector_hit(hit, rank_distance)
        })
        .min_by(compare_semantic_hits_for_rank)
}

fn semantic_hit_from_vector_hit(hit: VectorSearchHit, rank_distance: f64) -> SemanticSearchHit {
    SemanticSearchHit {
        record_key: hit.record_key,
        embedding_unit_key: hit.embedding_unit_key,
        unit_kind: hit.unit_kind,
        label: hit.label,
        distance: hit.distance,
        rank_distance,
    }
}

fn rank_distance(hit: &VectorSearchHit, has_parent: bool, mode: SemanticSearchMode) -> f64 {
    if !mode.uses_rank_weights() {
        return hit.distance;
    }
    let unit_penalty = match parsed_unit_kind(hit) {
        Some(EmbeddingUnitKind::Parent) => 0.0,
        Some(EmbeddingUnitKind::HeadingSection) => 0.025,
        Some(EmbeddingUnitKind::TitledOption) => 0.040,
        _ => 0.050,
    };
    let missing_parent_penalty = if has_parent { 0.0 } else { 0.025 };
    hit.distance + unit_penalty + missing_parent_penalty
}

fn parsed_unit_kind(hit: &VectorSearchHit) -> Option<EmbeddingUnitKind> {
    hit.unit_kind.parse().ok()
}

fn compare_semantic_hits_for_rank(
    left: &SemanticSearchHit,
    right: &SemanticSearchHit,
) -> std::cmp::Ordering {
    left.rank_distance
        .total_cmp(&right.rank_distance)
        .then_with(|| left.distance.total_cmp(&right.distance))
        .then_with(|| left.record_key.cmp(&right.record_key))
        .then_with(|| left.embedding_unit_key.cmp(&right.embedding_unit_key))
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

        let fused = fuse_ranked_hits(FusionInput {
            fts_hits: &fts_hits,
            vector_hits: &vector_hits,
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
