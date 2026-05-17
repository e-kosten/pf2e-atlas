#![deny(unsafe_code)]

use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;

use atlas_domain::{RecordKey, SearchFilterNode};
use atlas_embedding::{EmbeddingModelId, EmbeddingRuntimeConfig, EmbeddingUnitKind, TextEmbedder};
use atlas_index::{
    AtlasIndex, FilterCompileError, FilteredRecordKeyPage, FilteredRecordSort,
    IndexValidationError, RecordLoadError, VectorQueryError, VectorSearchHit,
};
use atlas_record::{PersistedRecord, RecordAlias};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Initial top-level retrieval boundary for Rust runtime consumers.
///
/// The service owns product-facing retrieval entrypoints. The current Rust
/// migration slice exposes semantic search first, while the semantic-only
/// implementation remains a private component behind this boundary. Future
/// record get/resolve, filter-only list, lexical, and hybrid behavior should
/// be added here rather than as peer public services.
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

#[derive(Debug, Clone, Copy)]
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
    Lexical {
        query: &'a str,
        filter: Option<&'a SearchFilterNode>,
        limit: u32,
    },
    Hybrid {
        query: &'a str,
        filter: Option<&'a SearchFilterNode>,
        limit: u32,
        semantic_mode: SemanticSearchMode,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum AtlasSearchResult {
    Semantic(SemanticSearchResult),
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
            AtlasSearchRequest::Lexical { .. } => {
                Err(SearchError::UnsupportedRetrievalPattern("lexical search"))
            }
            AtlasSearchRequest::Hybrid { .. } => {
                Err(SearchError::UnsupportedRetrievalPattern("hybrid search"))
            }
        }
    }

    pub fn get_records(
        &self,
        record_keys: &[RecordKey],
    ) -> Result<Vec<PersistedRecord>, SearchError> {
        Ok(self.index.load_records_by_key(record_keys)?)
    }

    pub fn get_record(
        &self,
        record_key: &RecordKey,
    ) -> Result<Option<PersistedRecord>, SearchError> {
        Ok(self
            .index
            .load_records_by_key(std::slice::from_ref(record_key))?
            .into_iter()
            .next())
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
        let records = self.index.load_records_by_key(&record_keys)?;
        Ok(FilterOnlyRecordPage {
            record_keys,
            records,
            total,
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

#[cfg(test)]
mod tests {
    use super::*;

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
