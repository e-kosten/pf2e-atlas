use crate::sqlite::SqliteIndexReader;
use crate::{ArtifactValidationReport, IndexValidationError};
use atlas_domain::{RecordKey, SearchFilterNode};

mod extension;
mod query;
mod types;
mod validation;

pub(crate) use extension::register_sqlite_vec_extension;
#[cfg(test)]
pub(crate) use query::compile_vector_knn_query;
pub(crate) use query::{load_record_embedding_vectors, query_vector_index};
pub use types::{RecordEmbeddingVector, VectorQueryError, VectorSearchHit};
pub(crate) use validation::{
    check_embedding_readiness_connection, validate_embedding_readiness_connection,
    validate_vector_index_connection, vector_extension_unavailable_report_from_base,
};

impl SqliteIndexReader {
    pub fn validate_vector_index(&self) -> Result<ArtifactValidationReport, IndexValidationError> {
        validate_vector_index_connection(
            self.path().display().to_string(),
            self.validate()?,
            &self.validation_connection()?,
        )
    }

    pub fn vector_validation_report(&self) -> ArtifactValidationReport {
        match self.validate_vector_index() {
            Ok(report) => report,
            Err(error) => crate::validation_report_from_error(self.path(), error),
        }
    }

    pub fn query_vector_index(
        &self,
        query_vector: &[f32],
        filter: Option<&SearchFilterNode>,
        limit: u32,
        include_child_units: bool,
    ) -> Result<Vec<VectorSearchHit>, VectorQueryError> {
        self.with_diesel_connection(|connection| {
            query_vector_index(connection, query_vector, filter, limit, include_child_units)
        })
    }

    pub fn load_record_embedding_vectors(
        &self,
        record_key: &RecordKey,
    ) -> Result<Vec<RecordEmbeddingVector>, VectorQueryError> {
        self.with_diesel_connection(|connection| {
            load_record_embedding_vectors(connection, record_key)
        })
    }
}
