use crate::read::search::vector;
use crate::sqlite::SqliteIndexReader;
use crate::{
    ArtifactValidationReport, IndexInspectionReport, IndexValidationError, ValidationTarget,
    check_index_connection, inspect, validate_index_connection,
};

impl SqliteIndexReader {
    pub fn validate(&self) -> Result<ArtifactValidationReport, IndexValidationError> {
        validate_index_connection(
            self.path().display().to_string(),
            &self.validation_connection()?,
        )
    }

    pub fn validate_report(&self) -> ArtifactValidationReport {
        match self.validate() {
            Ok(report) => report,
            Err(error) => crate::validation_report_from_error(self.path(), error),
        }
    }

    pub fn check(&self) -> Result<ArtifactValidationReport, IndexValidationError> {
        check_index_connection(
            self.path().display().to_string(),
            &self.validation_connection()?,
        )
    }

    pub fn check_report(&self) -> ArtifactValidationReport {
        match self.check() {
            Ok(report) => report,
            Err(error) => crate::validation_report_from_error(self.path(), error),
        }
    }

    pub fn check_embedding_readiness_report(&self) -> ArtifactValidationReport {
        match self.check() {
            Ok(report) => {
                let connection = match self.validation_connection() {
                    Ok(connection) => connection,
                    Err(error) => return crate::validation_report_from_error(self.path(), error),
                };
                match vector::check_embedding_readiness_connection(
                    self.path().display().to_string(),
                    report,
                    &connection,
                ) {
                    Ok(report) => report,
                    Err(error) => crate::validation_report_from_error(self.path(), error),
                }
            }
            Err(error) => crate::validation_report_from_error(self.path(), error),
        }
    }

    pub fn validate_target(
        &self,
        target: ValidationTarget,
    ) -> Result<ArtifactValidationReport, IndexValidationError> {
        match target {
            ValidationTarget::BaseOnly => self.validate(),
            ValidationTarget::Full => self.validate_vector_index(),
            ValidationTarget::EmbeddingsOnly => self.validate_embedding_readiness(),
        }
    }

    pub fn validate_embedding_readiness(
        &self,
    ) -> Result<ArtifactValidationReport, IndexValidationError> {
        vector::validate_embedding_readiness_connection(
            self.path().display().to_string(),
            &self.validation_connection()?,
        )
    }

    pub fn vector_extension_unavailable_report(
        &self,
        target: ValidationTarget,
        message: String,
    ) -> ArtifactValidationReport {
        let base_report = match target {
            ValidationTarget::EmbeddingsOnly => match self.validation_connection() {
                Ok(connection) => {
                    match crate::validate_index_metadata_connection(
                        self.path().display().to_string(),
                        &connection,
                    ) {
                        Ok(report) => report,
                        Err(error) => crate::validation_report_from_error(self.path(), error),
                    }
                }
                Err(error) => crate::validation_report_from_error(self.path(), error),
            },
            ValidationTarget::BaseOnly | ValidationTarget::Full => self.validate_report(),
        };
        vector::vector_extension_unavailable_report_from_base(
            self.path().display().to_string(),
            base_report,
            message,
        )
    }

    pub fn validate_target_report(&self, target: ValidationTarget) -> ArtifactValidationReport {
        match self.validate_target(target) {
            Ok(report) => report,
            Err(error) => crate::validation_report_from_error(self.path(), error),
        }
    }

    pub fn inspect(&self) -> Result<IndexInspectionReport, IndexValidationError> {
        let validation = self.validate()?;
        inspect::inspect_index_connection(
            self.path().display().to_string(),
            validation,
            &self.validation_connection()?,
        )
    }
}
