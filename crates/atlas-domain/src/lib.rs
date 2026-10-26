#![deny(unsafe_code)]

pub mod categories;
mod name_lookup;
mod search_profile;
pub use search_profile::SearchProfileId;
pub mod query;
pub mod query_discovery;
pub mod record_key;
pub use query_discovery::{
    FacetUnavailableReason, QueryFacetOutcome, QueryFieldCounts, QueryStateCount, QueryValueOption,
    QueryValueOptions, QueryVocabulary, QueryVocabularyBasis, QueryVocabularyOption,
};
pub mod source_address;
pub mod source_summary;

pub use source_summary::{SourceLevelBasis, SourceQueryFact, SourceRecordSummary};

pub use query::{
    QueryCapability, QueryCompare, QueryError, QueryExpression, QueryFieldDefinition,
    QueryFieldState, QueryFieldType, QueryLimits, QueryLiteral, QueryPredicate, QuerySetMatch,
    QueryTruth, QueryValueDiscovery,
};
pub use source_address::{
    InvalidSourceByteRange, OwnedContentIdentity, OwnedContentLocator, SourceByteRange,
    SourcePassageAddress,
};
mod query_relationship;
pub use query_relationship::{
    QueryRecordSelection, QueryRelationship, QueryRelationshipDirection,
    assign_relationship_clause_ids, relationship_clauses, select_relationship_clause_id,
};

pub use categories::RecordKind;
pub use name_lookup::normalize_record_name;
pub use record_key::{PackName, RecordId, RecordKey, RecordKeyParseError};

mod search_ranking;
pub use search_ranking::SearchRanking;

mod trait_key;
pub use trait_key::{TraitKey, TraitKeyParseError};

mod result_sort;
pub use result_sort::{ResultSort, SortDirection, SortField, SortRule};

mod similarity;
pub use similarity::{
    SIMILARITY_RECIPE, SIMILARITY_ROLE_RECIPE, SimilarityCandidateParts, SimilarityOptions,
    SimilarityPart, SimilarityPartSelector, SimilarityParts, SimilarityRole, SimilaritySeed,
    default_similarity_decay,
};
