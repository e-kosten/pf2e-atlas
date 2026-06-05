#![deny(unsafe_code)]

mod error;
mod fusion;
mod graph;
mod page;
mod query;
mod records;
mod remaster;
mod semantic;
mod service;
mod similar;
mod text;
mod variants;

pub use error::{SearchError, SearchErrorKind};
pub use graph::{
    GraphContextEdge, GraphContextEdgeSource, GraphContextRequest, GraphContextResult,
    GraphContextSection, GraphRetrieval,
};
pub use page::{DEFAULT_SEARCH_PAGE_SIZE, MAX_SEARCH_PAGE_SIZE, SearchPage, SearchPageInfo};
pub use query::TextQueryDiagnostics;
pub use records::{
    GetRecordRequest, GetRecordsRequest, ListRecordsRequest, ListRecordsResult, RecordListSort,
    RecordResolutionMatchKind, RecordResolutionResult, RecordRetrieval, ResolveRecordRequest,
};
pub use remaster::{
    RemasterLinkResult, RemasterLinksRequest, RemasterLinksResult, RemasterRetrieval,
};
pub use service::{AtlasRetrievalService, SearchEmbeddingConfig};
pub use similar::{
    SimilarRecord, SimilarRecordGraphEvidence, SimilarRecordRequest, SimilarRecordResult,
    SimilarRecordSemanticEvidence, SimilarRetrieval, SimilarScoreWeights, SimilarSharedReference,
};
pub use text::{
    DEFAULT_RANKED_CANDIDATE_WINDOW, MAX_RANKED_CANDIDATE_WINDOW, RetrievalMode, TextRetrieval,
    TextSearchDiagnostics, TextSearchMatch, TextSearchMatchDiagnostics, TextSearchRecord,
    TextSearchRequest, TextSearchResult, TextSearchTuning,
};
pub use variants::{
    VariantBaseNameRequest, VariantGroupRequest, VariantGroupResult, VariantRetrieval,
};

pub mod expert {
    pub use crate::fusion::{
        DEFAULT_FTS_FUSION_POLICY_NAME, FtsLane, FtsMatchConfidence, FusionMethod, FusionOptions,
        TextSearchExplain,
    };
    pub use crate::semantic::{
        SemanticRetrieval, SemanticSearchHit, SemanticSearchMode, SemanticSearchRequest,
        SemanticSearchResult, SemanticSearchTiming,
    };
}
