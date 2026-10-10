#![deny(unsafe_code)]
mod discovery;
mod error;
mod graph;
mod page;
mod records;
mod service;
mod similar;
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
#[cfg(test)]
mod tests;
mod text;
mod variants;
pub use atlas_index::{SourcePreparedContent, SourceStoredRelationship, SourceUnitLocation};
pub use atlas_index::{
    SourceRelationshipBundle, SourceRelationshipDirection, SourceRelationshipRequest,
};
pub use discovery::{
    DiscoverFilterCountsRequest, DiscoverFilterValuesRequest, FilterDiscoveryContext,
};
pub use error::{SearchError, SearchErrorKind};
pub use graph::{
    GraphContextRequest, GraphContextResult, GraphContextSection, RemasterLinkResult,
    RemasterLinksRequest, RemasterLinksResult,
};
pub use page::{DEFAULT_SEARCH_PAGE_SIZE, MAX_SEARCH_PAGE_SIZE, SearchPage, SearchPageInfo};
pub use records::{
    GetRecordRequest, GetRecordsRequest, ListRecordsRequest, ListRecordsResult,
    RecordRefResolutionResult, RecordResolutionMatchKind, RecordResolutionResult, RecordScope,
    ResolveRecordRefRequest, ResolveRecordRequest, SourceRecordDetail,
};
pub use service::{AtlasRetrievalService, SearchEmbeddingConfig};
pub use similar::{SimilarRecordRequest, SimilarRecordResult};
pub use text::{
    RetrievalMode, SearchCoverage, SearchLane, SearchWitness, TextSearchRecord, TextSearchRequest,
    TextSearchResult,
};
pub use variants::{VariantGroupRequest, VariantGroupResult, VariantSuggestionEvidence};
