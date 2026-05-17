use std::process::ExitCode;

use atlas_index::{ArtifactValidationReport, ValidationCode, ValidationStatus};

pub(crate) fn write_validation_report(
    report: ArtifactValidationReport,
    json: bool,
) -> Result<ExitCode, String> {
    let exit_code = match report.status {
        ValidationStatus::Ok => ExitCode::SUCCESS,
        ValidationStatus::Error => ExitCode::from(1),
    };

    if json {
        let body = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?;
        println!("{body}");
    } else {
        println!(
            "{}: {}",
            validation_code_label(&report.code),
            report.message
        );
    }

    Ok(exit_code)
}

fn validation_code_label(code: &ValidationCode) -> &'static str {
    match code {
        ValidationCode::Ok => "ok",
        ValidationCode::IndexUnavailable => "index-unavailable",
        ValidationCode::MissingArtifactMetadata => "missing-artifact-metadata",
        ValidationCode::MissingRequiredMetadata => "missing-required-metadata",
        ValidationCode::UnsupportedContractVersion => "unsupported-contract-version",
        ValidationCode::UnsupportedSchemaVersion => "unsupported-schema-version",
        ValidationCode::ArtifactContractViolation => "artifact-contract-violation",
        ValidationCode::InvalidSourceMetadata => "invalid-source-metadata",
        ValidationCode::StaleSourceSignature => "stale-source-signature",
        ValidationCode::EmbeddingMismatch => "embedding-mismatch",
        ValidationCode::FtsMismatch => "fts-mismatch",
        ValidationCode::ManifestMismatch => "manifest-mismatch",
        ValidationCode::VectorExtensionUnavailable => "vector-extension-unavailable",
        ValidationCode::QueryFailed => "query-failed",
    }
}
