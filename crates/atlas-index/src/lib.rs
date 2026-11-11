#![deny(unsafe_code)]
mod source_admission;
pub use source_admission::{CompleteSourceInventory, SourceAdmissionSummary, SourceExcludedInput};
mod artifact_instance;
mod catalog_fingerprint;
mod codec;
mod component_digests;
mod component_upgrade;
mod core_projection;
#[cfg(any(test, feature = "test-support"))]
mod core_projection_pilot;
mod discovery;
mod embedding_seed;
pub use embedding_seed::{
    EmbeddingSeed, EmbeddingSeedFile, EmbeddingSeedHeader, MAX_EMBEDDING_SEED_BYTES, SeedSelection,
    export_embedding_seed,
};
mod embedding_values;
mod native_allocation;
mod native_token_counts;
mod native_values;
pub use native_token_counts::{
    NATIVE_TOKEN_COUNT_DIGEST_SCHEMA_VERSION, VerifiedNativeTokenCounts,
};
mod passage_layouts;
mod persisted_inputs;
pub use persisted_inputs::ReusableSourceAdmission;
mod reuse_reader;
pub use passage_layouts::SourcePassageLayout;
#[cfg(any(test, feature = "test-support"))]
pub use passage_layouts::read_reusable_passage_layouts;
pub use reuse_reader::{
    ReferenceReuseReader, ReusableSourceIdentity, SOURCE_REUSE_BATCH_LIMIT, visit_persisted_inputs,
};
mod maintenance_recovery;
mod selected_sections;
mod semantic_locations;
pub use component_upgrade::{
    ReferenceUpgrade, classify_reference_upgrade, ensure_reference_projections,
    read_stored_reference_context,
};
#[cfg(any(test, feature = "test-support"))]
pub use persisted_inputs::collect_persisted_inputs_for_fixture;
pub use selected_sections::SELECTED_SECTIONS_RECIPE;
mod requested_config;
pub use requested_config::{
    RequestedReferenceConfig, read_requested_reference_config, write_requested_reference_config,
};
mod build_scratch;
mod build_spool;
pub use build_scratch::BuildScratch;
mod prepared_publication;
pub use prepared_publication::{
    NATIVE_VALUE_BATCH_LIMIT, NativeValueSelection, PreparedPublication,
};
#[cfg(test)]
mod core_tests;
mod eligibility_buffer;
mod error;
#[cfg(test)]
mod foundation_tests;
mod identity;
mod input;
mod publication_source;
pub use build_spool::{
    BuildFileUsage, BuildFrameCursor, BuildFrameOffset, BuildFrameStream, PreparedRecordInput,
    PublicationSpool,
};
pub use publication_source::{PreparedRecordRef, SemanticOccurrence};
mod numeric;
mod persistence;
#[cfg(test)]
mod profile_tests;
mod projections;
mod publication_delta;
pub mod query;
mod read_content;
mod read_core;
mod read_similarity;
mod similarity_candidates;
#[cfg(any(test, feature = "test-support"))]
pub use eligibility_buffer::EligibilityStats;
pub use similarity_candidates::ConfiguredSimilarityCandidates;
mod read_similarity_sections;
pub use read_similarity::{SourceSimilarityCandidateRequest, SourceSimilarityUnit};
mod read_units;
mod vector_neighbors;
#[cfg(test)]
mod vector_neighbors_tests;
pub use vector_neighbors::SourceVectorNeighbors;
mod reader;
mod reference_reset;
mod schema;
mod stream_publication;
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
mod validation;
#[cfg(test)]
mod validation_tests;
mod validation_units;
mod writer;

pub use artifact_instance::ArtifactInstance;
pub use discovery::{
    QueryCountsRequest, QueryFacetContext, QueryValueCountsRequest, QueryValuesRequest,
    QueryVocabularyRequest, facet_base_query, validate_value_counts, validate_value_discovery,
};
pub use error::IndexError;
pub use input::{
    ARTIFACT_CONTRACT_VERSION, ARTIFACT_FORMAT_VERSION, ASSET_POLICY_VERSION, EXPECTED_SOURCE_KIND,
    IndexBuildPack, LEXICAL_SELECTION_VERSION, LOCALIZATION_POLICY_VERSION,
    RELATIONSHIP_POLICY_VERSION, SOURCE_LOOKUP_VERSION, SOURCE_PROJECTION_VERSION,
    SourceAliasInput, SourceArtifactBuildContext, SourceArtifactDiagnostic,
    SourceArtifactRecordInput, SourceLexicalUnitInput, SourceLexicalUnitKind,
    SourcePreparedContent, SourceRelationshipDetails, SourceRemasterPairInput,
    SourceSearchProfileIdentity, SourceStoredRelationship, SourceUnitLocation,
};
#[cfg(any(test, feature = "test-support"))]
pub use input::{IndexBuildInput, SourceSemanticUnitInput};
pub use maintenance_recovery::recover_reference_publication;
pub use query::{ValidatedQuery, parse_where, query_capabilities, validate_query};
pub use read_content::{
    SourceAttributedContentBundle, SourceContentBundle, SourceRelationshipBundle,
    SourceRelationshipDirection, SourceRelationshipRequest,
};
pub use read_units::{
    SourceIdentityMatch, SourceIdentityVector, SourceKeyPage, SourceLexicalHit,
    SourceLexicalRootHit, SourceRemasterPair, SourceVectorCandidateRequest, SourceVectorHit,
};
#[cfg(any(test, feature = "test-support"))]
pub use reader::SourceReadMetrics;
pub use reader::{
    SourceArtifactStatistics, SourceProfileStatistics, SourceReadCapabilities, SourceSummaryBatch,
    SqliteIndexReader,
};
pub use reader::{read_product_summaries, record_projection_context};
#[cfg(any(test, feature = "test-support"))]
pub use validation::validate_artifact;
pub use validation::{
    ArtifactValidationReport, validate_reference, validate_reference_with_token_count_digests,
};

mod lexical_projection;
pub use lexical_projection::project_lexical_units;

mod owned_abilities;
#[cfg(test)]
mod reference_coverage_tests;
pub use owned_abilities::{OwnedAbilityCatalog, OwnedAbilityEvidence, OwnedAbilityTarget};
#[cfg(test)]
mod reference_tests;
mod references;
mod traits;
pub use references::{
    ReferenceAvailability, ReferenceBacking, ReferenceCoverage, ReferenceEntry,
    ReferenceGrantScope, ReferenceGrantUse, ReferenceNotice, ReferencePermission,
    ReferencePermissionDisposition, ReferencePreparation, ReferenceProvenance,
    ReferencePublication, ReferenceRecordMapping, ReferenceSourceLocation, ReferenceSummary,
    ReferenceSummaryBacking, reference_inventory_identity, reference_pack_applies,
};
pub use traits::{TraitDefinition, TraitMembership};

mod relationship_query;
pub use relationship_query::{RelationshipOccurrencePage, RelationshipOccurrencePageRequest};

mod result_order;
pub use result_order::SourceUnitPrice;

#[cfg(test)]
mod embedding_values_tests;

#[cfg(test)]
mod publication_crash_tests;

mod remaster;
