#![deny(unsafe_code)]
mod codec;
mod discovery;
mod error;
#[cfg(test)]
mod foundation_tests;
mod input;
mod numeric;
mod persistence;
mod projections;
pub mod query;
mod read_content;
mod read_reuse;
mod read_units;
mod reader;
mod schema;
mod validation;
mod validation_units;
mod writer;

pub use discovery::{QueryCountsRequest, QueryFacetContext, QueryValuesRequest, facet_base_query};
pub use error::IndexError;
pub use input::{
    ARTIFACT_CONTRACT_VERSION, ARTIFACT_FORMAT_VERSION, ARTIFACT_SCHEMA_VERSION,
    ASSET_POLICY_VERSION, EXPECTED_SOURCE_KIND, IndexBuildInput, IndexBuildPack,
    LEXICAL_SELECTION_VERSION, LOCALIZATION_POLICY_VERSION, RELATIONSHIP_POLICY_VERSION,
    SOURCE_LOOKUP_VERSION, SOURCE_PROJECTION_VERSION, SourceAliasInput, SourceArtifactBuildContext,
    SourceArtifactDiagnostic, SourceArtifactRecordInput, SourceLexicalUnitInput,
    SourceLexicalUnitKind, SourcePreparedContent, SourceRelationshipDetails,
    SourceRemasterPairInput, SourceSemanticModelIdentity, SourceSemanticUnitInput,
    SourceStoredRelationship, SourceUnitLocation,
};
pub use query::{ValidatedQuery, parse_where, query_capabilities, validate_query};
pub use read_content::{
    SourceContentBundle, SourceRelationshipBundle, SourceRelationshipDirection,
    SourceRelationshipRequest,
};
pub use read_units::{
    SourceIdentityMatch, SourceIdentityVector, SourceKeyPage, SourceLexicalHit,
    SourceLexicalRootHit, SourceRemasterPair, SourceReuseCandidate, SourceVectorHit,
};
#[cfg(any(test, feature = "test-support"))]
pub use reader::SourceReadMetrics;
pub use reader::{
    SourceArtifactStatistics, SourceReadCapabilities, SourceSummaryBatch, SqliteIndexReader,
};
pub use validation::{ArtifactValidationReport, validate_artifact};
pub use writer::{IndexArtifactWriter, SqliteIndexWriter};
