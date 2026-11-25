//! Admission failures and completed checks have different diagnostic outcomes.
use crate::{AtlasRuntime, SetupTarget};
use atlas_index::{
    ArtifactValidationReport, IndexError, SourceArtifactStatistics, SourceCheckCounts,
    SqliteIndexReader,
};
use atlas_storage::StorageError;
use std::fmt;

#[derive(Debug)]
pub enum IndexDiagnosticError {
    MissingVectors,
    InvalidArtifact(Box<IndexError>),
    CouldNotRun(Box<IndexError>),
}

impl IndexDiagnosticError {
    fn could_not_run(error: impl Into<IndexError>) -> Self {
        Self::CouldNotRun(Box::new(error.into()))
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::MissingVectors => "vectors_unavailable",
            Self::InvalidArtifact(_) => "artifact_invalid",
            Self::CouldNotRun(error) => match error.as_ref() {
                IndexError::Storage(error) if error.is_contention() => "service_busy",
                IndexError::Storage(error) if error.is_access_failure() => "index_unavailable",
                IndexError::Storage(StorageError::Missing(_) | StorageError::Maintenance(_))
                | IndexError::Unavailable(_) => "index_unavailable",
                IndexError::Unsupported(_)
                | IndexError::Storage(StorageError::Schema { .. } | StorageError::Unsupported(_)) => {
                    "artifact_incompatible"
                }
                _ => "artifact_invalid",
            },
        }
    }
}

impl fmt::Display for IndexDiagnosticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingVectors => f.write_str(
                "The requested semantic target lacks stored vectors for one or more profiles.",
            ),
            Self::InvalidArtifact(error) | Self::CouldNotRun(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for IndexDiagnosticError {}

impl AtlasRuntime {
    fn diagnostic_index(
        &self,
        target: SetupTarget,
    ) -> Result<SqliteIndexReader, IndexDiagnosticError> {
        let session = crate::workspace::reference_read_session(self.index_path())
            .map_err(IndexDiagnosticError::could_not_run)?;
        let index = SqliteIndexReader::from_session(session)
            .map_err(IndexDiagnosticError::could_not_run)?;
        if target.requires_embeddings()
            && !index
                .has_stored_vectors()
                .map_err(IndexDiagnosticError::could_not_run)?
        {
            return Err(IndexDiagnosticError::MissingVectors);
        }
        Ok(index)
    }

    pub fn check_index_counts(
        &self,
        target: SetupTarget,
    ) -> Result<SourceCheckCounts, IndexDiagnosticError> {
        self.diagnostic_index(target)?
            .check_counts()
            .map_err(IndexDiagnosticError::could_not_run)
    }

    pub fn check_index_report(
        &self,
        target: SetupTarget,
    ) -> Result<SourceArtifactStatistics, IndexDiagnosticError> {
        self.diagnostic_index(target)?
            .statistics()
            .map_err(IndexDiagnosticError::could_not_run)
    }

    pub fn validate_index_report(
        &self,
        target: SetupTarget,
    ) -> Result<ArtifactValidationReport, IndexDiagnosticError> {
        let index = self.diagnostic_index(target)?;
        atlas_index::validate_reference_with_token_count_digests(
            index.workspace_session().connection(),
        )
        .map_err(|error| match error {
            IndexError::Invalid(_) | IndexError::ReferenceRepairRequired(_) => {
                IndexDiagnosticError::InvalidArtifact(Box::new(error))
            }
            other => IndexDiagnosticError::could_not_run(other),
        })
    }
}
