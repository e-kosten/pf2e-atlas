#![deny(unsafe_code)]

mod error;
mod fusion;
mod graph;
mod query;
mod records;
mod remaster;
mod semantic;
mod service;
mod similar;
mod text;
mod variants;

pub use error::{SearchError, SearchErrorKind};
pub use fusion::{
    DEFAULT_FTS_FUSION_POLICY_NAME, FtsLane, FtsMatchConfidence, FusionMethod, FusionOptions,
    TextSearchExplain,
};
pub use graph::{
    GraphContextEdge, GraphContextEdgeSource, GraphContextRequest, GraphContextResult,
    GraphContextSection, GraphRetrieval,
};
pub use query::TextQueryAnalysis;
pub use records::{
    GetRecordRequest, GetRecordsRequest, ListRecordsRequest, ListRecordsResult, RecordListSort,
    RecordResolutionMatchKind, RecordResolutionResult, RecordRetrieval, ResolveRecordRequest,
};
pub use remaster::{
    RemasterLinkResult, RemasterLinksRequest, RemasterLinksResult, RemasterRetrieval,
};
pub use semantic::{
    SemanticRetrieval, SemanticSearchHit, SemanticSearchMode, SemanticSearchRequest,
    SemanticSearchResult, SemanticSearchTiming,
};
pub use service::{AtlasRetrievalService, SearchEmbeddingConfig};
pub use similar::{
    SimilarRecord, SimilarRecordGraphEvidence, SimilarRecordRequest, SimilarRecordResult,
    SimilarRecordSemanticEvidence, SimilarRetrieval, SimilarScoreWeights, SimilarSharedReference,
};
pub use text::{
    RetrievalMode, TextRetrieval, TextSearchMatch, TextSearchRecord, TextSearchRequest,
    TextSearchResult,
};
pub use variants::{
    VariantBaseNameRequest, VariantGroupRequest, VariantGroupResult, VariantRetrieval,
};
