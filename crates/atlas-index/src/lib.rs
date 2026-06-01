#![deny(unsafe_code)]

mod artifact;
mod discovery;
mod embedding_cache;
mod inspect;
mod metadata;
mod read;
mod schema;
mod sql;
mod sqlite;
#[cfg(feature = "test-support")]
pub mod test_support;
#[cfg(test)]
mod tests;
mod validation;
mod write;

pub use artifact::metadata::{
    ARTIFACT_CONTRACT_VERSION, ARTIFACT_SCHEMA_VERSION, EXPECTED_SOURCE_KIND,
};
pub use artifact::validation::validation_report_for_error;
pub use embedding_cache::{DocumentEmbeddingCacheError, DocumentEmbeddingCacheReader};
pub use inspect::{
    IndexInspectionReport, MetricCoverageReport, RecordCoverageReport, RelationshipCoverageReport,
    TaxonomyCoverageReport, TextCoverageReport, VariantCoverageReport,
};
pub use read::RetrievalReadIndex;
pub use read::discovery::{DiscoveryError, DiscoveryValueSort, FilterValueRequest};
pub use read::graph::edges::GraphReferenceEdge;
pub use read::graph::product::{
    IndexRemasterLinkRecord, IndexRemasterLinks, IndexVariantGroup, ReferenceReadIndex,
    RemasterReadIndex, VariantReadIndex,
};
pub use read::records::RecordLoadError;
pub use read::search::filters::FilterCompileError;
pub use read::search::vector::{RecordEmbeddingVector, VectorQueryError, VectorSearchHit};
pub use read::search::{
    FilterReadIndex, FtsReadIndex, IdentityReadIndex, RecordIdentityMatch, RecordIdentityMatchKind,
    RecordReadIndex, SearchCandidateRecord, VectorReadIndex,
};
pub use sqlite::{
    FilteredRecordKeyPage, FilteredRecordSort, FtsColumnWeights, FtsQuery, FtsSearchHit,
    FtsSearchLane, ReferenceEdgeDirection, SqliteIndexReader, SqliteIndexWriter,
};
pub use validation::{
    ArtifactMetadataSummary, ArtifactValidationDiagnostic, ArtifactValidationFamily,
    ArtifactValidationReport, IndexValidationError, ValidationCode, ValidationStatus,
    ValidationTarget,
};
pub use write::input::{IndexBuildInput, IndexBuildInputError, IndexBuildPack};
pub use write::{IndexArtifactWriter, IndexWriteError};

pub(crate) use artifact::validation::{
    check_index_connection, validate_index_connection, validate_index_metadata_connection,
    validation_report_from_error,
};
