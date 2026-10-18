#![deny(unsafe_code)]
mod artifact_instance;
mod codec;
mod component_upgrade;
mod discovery;
mod embedding_checkpoints;
mod persisted_inputs;
pub use component_upgrade::{
    ReferenceUpgrade, classify_reference_upgrade, ensure_query_projections,
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
pub use embedding_checkpoints::{
    read_embedding_checkpoints, recover_reference_publication, save_embedding_checkpoints,
    start_reference_job,
};
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
pub use schema::REFERENCE_SCHEMA_SQL;
#[cfg(any(test, feature = "test-support"))]
pub use validation::validate_artifact;
pub use validation::{ArtifactValidationReport, validate_reference};
pub use writer::{IndexArtifactWriter, SqliteIndexWriter};
