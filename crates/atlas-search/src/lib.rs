#![deny(unsafe_code)]

mod discovery;
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
#[cfg(feature = "test-support")]
pub mod test_support;
mod text;
mod variants;

pub use discovery::{
    DiscoverFilterFieldsRequest, DiscoverFilterValuesRequest, FilterDiscoveryError,
    FilterDiscoveryRetrieval, MetricDiscoverySelector,
};
pub use error::{SearchError, SearchErrorKind};
pub use graph::{
    DEFAULT_GRAPH_BACKLINK_LIMIT, DEFAULT_GRAPH_OUTGOING_LIMIT, DEFAULT_GRAPH_USES_LIMIT,
    GraphContextEdge, GraphContextEdgeSource, GraphContextRequest, GraphContextResult,
    GraphContextSection, GraphRetrieval, MAX_GRAPH_CONTEXT_LIMIT,
};
pub use page::{DEFAULT_SEARCH_PAGE_SIZE, MAX_SEARCH_PAGE_SIZE, SearchPage, SearchPageInfo};
pub use query::TextQueryDiagnostics;
pub use records::{
    GetRecordRequest, GetRecordsRequest, ListRecordsRequest, ListRecordsResult, RecordListSort,
    RecordRefResolutionResult, RecordResolutionMatchKind, RecordResolutionResult, RecordRetrieval,
    ResolveRecordRefRequest, ResolveRecordRequest,
};
pub use remaster::{
    RemasterLinkResult, RemasterLinksRequest, RemasterLinksResult, RemasterRetrieval,
};
pub use service::{AtlasRetrievalService, SearchEmbeddingConfig};
pub use similar::{
    DEFAULT_SIMILAR_CANDIDATE_LIMIT, DEFAULT_SIMILAR_RECORD_LIMIT,
    DEFAULT_SIMILAR_REFERENCE_WEIGHT, DEFAULT_SIMILAR_SEMANTIC_WEIGHT,
    DEFAULT_SIMILAR_TRAIT_WEIGHT, MAX_SIMILAR_CANDIDATE_LIMIT, MAX_SIMILAR_RECORD_LIMIT,
    SimilarRecord, SimilarRecordGraphEvidence, SimilarRecordRefRequest, SimilarRecordRefResult,
    SimilarRecordRequest, SimilarRecordResult, SimilarRecordSemanticEvidence, SimilarRetrieval,
    SimilarScoreWeights, SimilarSharedReference,
};
pub use text::{
    DEFAULT_RANKED_CANDIDATE_WINDOW, MAX_RANKED_CANDIDATE_WINDOW, RetrievalMode, TextRetrieval,
    TextSearchDiagnostics, TextSearchMatch, TextSearchMatchDiagnostics, TextSearchRecord,
    TextSearchRequest, TextSearchResult, TextSearchTuning,
};
pub use variants::{
    ResolveVariantGroupRefRequest, VariantBaseNameRequest, VariantGroupRefMatch,
    VariantGroupRefResolutionResult, VariantGroupRequest, VariantGroupResult, VariantRetrieval,
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
