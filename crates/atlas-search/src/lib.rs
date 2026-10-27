#![deny(unsafe_code)]
mod configurable_similarity;
mod discovery;
mod error;
mod graph;
mod page;
#[cfg(any(test, feature = "read-metrics"))]
mod read_metrics;
mod records;
mod reference_resolution;
mod service;
mod similar;
mod similarity_evaluation;
mod similarity_scoring;
pub use configurable_similarity::{
    ConfigurableSimilarRequest, ConfigurableSimilarResult, SimilarityDiagnostics,
    SimilarityEvidence, SimilarityPair, SimilarityPartScore, SimilarityRoleScore,
    SimilarityTimings,
};
pub use similarity_evaluation::{
    SimilarityEvaluationEvidence, SimilarityEvaluationRequest, SimilarityEvaluationResult,
    SimilarityEvaluationRow, SimilarityEvaluationRun, SimilarityEvaluationVariant,
};
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
#[cfg(test)]
mod tests;
mod text;
mod variants;
#[cfg(any(test, feature = "read-metrics"))]
pub use atlas_index::SourceReadMetrics;
pub use atlas_index::{
    ArtifactInstance, SourcePreparedContent, SourceStoredRelationship, SourceUnitLocation,
};
pub use atlas_index::{
    SourceRelationshipBundle, SourceRelationshipDirection, SourceRelationshipRequest,
};
pub use discovery::{
    DiscoverFilterCountsRequest, DiscoverFilterValueCountsRequest, DiscoverFilterValuesRequest,
    DiscoverFilterVocabularyRequest, FilterDiscoveryContext,
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
pub use service::{AtlasRetrievalService, PreparedQueryEmbedding};
pub use similar::{SimilarRecordRequest, SimilarRecordResult};
pub use text::{
    RetrievalMode, SearchCoverage, SearchLane, SearchWitness, TextSearchRecord, TextSearchRequest,
    TextSearchResult,
};
pub use variants::{
    VariantGroupOutcome, VariantGroupRequest, VariantGroupResult, VariantSuggestionEvidence,
};

mod fusion;

mod traits;
pub use traits::{
    SimilarTraitRequest, SimilarTraitResult, TraitAssociationRequest, TraitDefinition,
    TraitMembership, TraitMembershipResult, TraitReference, TraitReferenceResult,
    TraitSourceLocation,
};

pub use atlas_index::{RelationshipOccurrencePage, RelationshipOccurrencePageRequest};

#[cfg(test)]
mod relationship_tests;

mod result_order;
pub use atlas_index::SourceUnitPrice;

#[cfg(test)]
mod result_order_tests;

mod relationship_selection;
pub use relationship_selection::CheckedRelationshipSelection;
