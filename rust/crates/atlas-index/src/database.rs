use std::path::{Path, PathBuf};

use atlas_domain::SearchFilterNode;
use atlas_record::{PersistedRecord, PersistedRecordSet};
use rusqlite::{Connection, OpenFlags};

use crate::vector::register_sqlite_vec_extension;
use crate::{
    ArtifactValidationReport, IndexInspectionReport, IndexValidationError, RecordLoadError,
    VectorQueryError, VectorSearchHit, inspect, records, validate_index_connection, vector,
};

pub struct AtlasIndex {
    path: PathBuf,
    connection: Connection,
}

impl AtlasIndex {
    pub fn open_read_only(path: impl AsRef<Path>) -> Result<Self, IndexValidationError> {
        let path = path.as_ref().to_path_buf();
        let connection = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
        Ok(Self { path, connection })
    }

    pub fn open_read_only_with_vectors(
        path: impl AsRef<Path>,
    ) -> Result<Self, IndexValidationError> {
        register_sqlite_vec_extension().map_err(IndexValidationError::Unavailable)?;
        Self::open_read_only(path)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn validate(&self) -> Result<ArtifactValidationReport, IndexValidationError> {
        validate_index_connection(self.path.display().to_string(), &self.connection)
    }

    pub fn validate_report(&self) -> ArtifactValidationReport {
        match self.validate() {
            Ok(report) => report,
            Err(error) => crate::validation_report_from_error(&self.path, error),
        }
    }

    pub fn inspect(&self) -> Result<IndexInspectionReport, IndexValidationError> {
        let validation = self.validate()?;
        inspect::inspect_index_connection(
            self.path.display().to_string(),
            validation,
            &self.connection,
        )
    }

    pub fn load_records(&self) -> Result<Vec<PersistedRecord>, RecordLoadError> {
        records::load_persisted_records_from_connection(&self.connection)
    }

    pub fn load_record_set(&self) -> Result<PersistedRecordSet, RecordLoadError> {
        records::load_persisted_record_set_from_connection(&self.connection)
    }

    pub fn validate_vector_index(&self) -> Result<ArtifactValidationReport, IndexValidationError> {
        vector::validate_vector_index_connection(
            self.path.display().to_string(),
            self.validate()?,
            &self.connection,
        )
    }

    pub fn vector_validation_report(&self) -> ArtifactValidationReport {
        match self.validate_vector_index() {
            Ok(report) => report,
            Err(error) => crate::validation_report_from_error(&self.path, error),
        }
    }

    pub fn query_vector_index(
        &self,
        query_vector: &[f32],
        filter: Option<&SearchFilterNode>,
        limit: u32,
        include_child_units: bool,
    ) -> Result<Vec<VectorSearchHit>, VectorQueryError> {
        vector::query_vector_index(
            &self.connection,
            query_vector,
            filter,
            limit,
            include_child_units,
        )
    }
}
