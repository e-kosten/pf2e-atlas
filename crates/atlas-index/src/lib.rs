#![deny(unsafe_code)]
mod artifact_instance;
mod codec;
mod component_upgrade;
mod discovery;
mod embedding_storage_migration;
mod embedding_values;
mod persisted_inputs;
mod reference_jobs;
mod semantic_locations;
mod similarity_projection;
pub use similarity_projection::{SIMILARITY_SCHEMA_SQL, backfill_similarity_parts};
mod vector_allocation;
pub use component_upgrade::{
    ReferenceUpgrade, classify_reference_upgrade, ensure_reference_projections,
    read_stored_reference_context,
};
pub use persisted_inputs::read_persisted_inputs;
mod requested_config;
pub use requested_config::{
    RequestedReferenceConfig, read_requested_reference_config, write_requested_reference_config,
};
mod error;
#[cfg(test)]
mod foundation_tests;
mod identity;
mod input;
mod numeric;
mod persistence;
#[cfg(test)]
mod profile_tests;
mod projections;
mod publication_delta;
pub mod query;
mod read_content;
mod read_similarity;
mod read_similarity_sections;
pub use read_similarity::SourceSimilarityUnit;
mod read_units;
mod reader;
mod schema;
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
mod validation;
mod validation_units;
mod writer;

pub use artifact_instance::ArtifactInstance;
pub use discovery::{
    QueryCountsRequest, QueryFacetContext, QueryValueCountsRequest, QueryValuesRequest,
    QueryVocabularyRequest, facet_base_query, validate_value_counts, validate_value_discovery,
};
pub use embedding_storage_migration::{EMBEDDING_STORAGE_SCHEMA_SQL, backfill_embedding_storage};
pub use embedding_values::{read_reusable_embeddings, stage_embedding_values};
pub use error::IndexError;
pub use input::{
    ARTIFACT_CONTRACT_VERSION, ARTIFACT_FORMAT_VERSION, ASSET_POLICY_VERSION, EXPECTED_SOURCE_KIND,
    IndexBuildInput, IndexBuildPack, LEXICAL_SELECTION_VERSION, LOCALIZATION_POLICY_VERSION,
    RELATIONSHIP_POLICY_VERSION, SOURCE_LOOKUP_VERSION, SOURCE_PROJECTION_VERSION,
    SourceAliasInput, SourceArtifactBuildContext, SourceArtifactDiagnostic,
    SourceArtifactRecordInput, SourceLexicalUnitInput, SourceLexicalUnitKind,
    SourcePreparedContent, SourceRelationshipDetails, SourceRemasterPairInput,
    SourceSearchProfileIdentity, SourceSemanticUnitInput, SourceStoredRelationship,
    SourceUnitLocation,
};
pub use query::{ValidatedQuery, parse_where, query_capabilities, validate_query};
pub use read_content::{
    SourceAttributedContentBundle, SourceContentBundle, SourceRelationshipBundle,
    SourceRelationshipDirection, SourceRelationshipRequest,
};
pub use read_units::{
    SourceIdentityMatch, SourceIdentityVector, SourceKeyPage, SourceLexicalHit,
    SourceLexicalRootHit, SourceRemasterPair, SourceVectorHit,
};
#[cfg(any(test, feature = "test-support"))]
pub use reader::SourceReadMetrics;
pub use reader::{
    SourceArtifactStatistics, SourceProfileStatistics, SourceReadCapabilities, SourceSummaryBatch,
    SqliteIndexReader,
};
pub use reference_jobs::{recover_reference_publication, start_reference_job};
pub use schema::{CONSUMABLE_FILTER_SCHEMA_SQL, REFERENCE_SCHEMA_SQL};
#[cfg(any(test, feature = "test-support"))]
pub use validation::validate_artifact;
pub use validation::{ArtifactValidationReport, validate_reference};
pub use writer::{IndexArtifactWriter, SqliteIndexWriter};

mod lexical_projection;
pub use lexical_projection::project_lexical_units;
pub use schema::LEXICAL_ALIAS_SCHEMA_SQL;

mod traits;
pub use traits::{
    TRAIT_SCHEMA_SQL, TraitDefinition, TraitMembership, TraitReference, TraitReferenceInput,
    TraitSeedChunk, TraitSourceLocation, TraitVectorInput,
};

mod relationship_query;
pub use relationship_query::{RelationshipOccurrencePage, RelationshipOccurrencePageRequest};

mod result_order;
pub use result_order::SourceUnitPrice;

#[cfg(test)]
mod embedding_values_tests;
