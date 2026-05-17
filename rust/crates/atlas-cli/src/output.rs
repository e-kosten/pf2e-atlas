use std::process::ExitCode;

use atlas_index::{ArtifactValidationReport, ValidationCode, ValidationStatus};
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Debug, Serialize)]
struct SuccessEnvelope<T>
where
    T: Serialize,
{
    status: &'static str,
    data: T,
}

#[derive(Debug, Serialize)]
struct ErrorEnvelope {
    status: &'static str,
    error: CliError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CliError {
    pub code: &'static str,
    pub message: String,
}

pub(crate) fn write_json_data<T>(data: T) -> Result<(), String>
where
    T: Serialize,
{
    let body = serde_json::to_string_pretty(&SuccessEnvelope { status: "ok", data })
        .map_err(|error| error.to_string())?;
    println!("{body}");
    Ok(())
}

pub(crate) fn write_json_error(code: &'static str, message: String) -> Result<(), String> {
    let body = serde_json::to_string_pretty(&ErrorEnvelope {
        status: "error",
        error: CliError { code, message },
    })
    .map_err(|error| error.to_string())?;
    println!("{body}");
    Ok(())
}

pub(crate) fn write_validation_report(
    report: ArtifactValidationReport,
    json: bool,
) -> Result<ExitCode, String> {
    let exit_code = match report.status {
        ValidationStatus::Ok => ExitCode::SUCCESS,
        ValidationStatus::Error => ExitCode::from(3),
    };

    if json {
        write_json_data(validation_report_data(&report))?;
    } else {
        println!(
            "{}: {}",
            validation_code_label(&report.code),
            report.message
        );
    }

    Ok(exit_code)
}

fn validation_report_data(report: &ArtifactValidationReport) -> Value {
    let mut data = json!({
        "valid": report.status == ValidationStatus::Ok,
        "code": validation_code_json(&report.code),
        "index": report.index,
        "message": report.message,
    });

    let object = data
        .as_object_mut()
        .expect("validation report data should be an object");
    insert_optional(
        object,
        "artifact_contract_version",
        &report.artifact_contract_version,
    );
    insert_optional(object, "schema_version", &report.schema_version);
    insert_optional(object, "source_kind", &report.source_kind);
    insert_optional(object, "source_signature", &report.source_signature);
    insert_optional(object, "source_record_count", &report.source_record_count);
    insert_optional(
        object,
        "artifact_record_count",
        &report.artifact_record_count,
    );
    insert_optional(
        object,
        "generated_record_count",
        &report.generated_record_count,
    );
    insert_optional(
        object,
        "content_hash_algorithm",
        &report.content_hash_algorithm,
    );
    insert_optional(
        object,
        "embedding_provider_family",
        &report.embedding_provider_family,
    );
    insert_optional(object, "embedding_model_id", &report.embedding_model_id);
    insert_optional(
        object,
        "embedding_model_revision",
        &report.embedding_model_revision,
    );
    insert_optional(
        object,
        "embedding_tokenizer_id",
        &report.embedding_tokenizer_id,
    );
    insert_optional(object, "embedding_pooling", &report.embedding_pooling);
    insert_optional(
        object,
        "embedding_normalization",
        &report.embedding_normalization,
    );
    insert_optional(object, "embedding_dimensions", &report.embedding_dimensions);
    insert_optional(object, "embedding_dtype", &report.embedding_dtype);
    insert_optional(
        object,
        "embedding_distance_metric",
        &report.embedding_distance_metric,
    );
    insert_optional(
        object,
        "embedding_document_prefix",
        &report.embedding_document_prefix,
    );
    insert_optional(
        object,
        "embedding_query_prefix",
        &report.embedding_query_prefix,
    );
    insert_optional(object, "fts_tokenizer", &report.fts_tokenizer);
    insert_optional(
        object,
        "adjacent_manifest_path",
        &report.adjacent_manifest_path,
    );
    insert_optional(
        object,
        "legacy_schema_version",
        &report.legacy_schema_version,
    );
    if !report.missing_keys.is_empty() {
        object.insert("missing_keys".to_string(), json!(report.missing_keys));
    }
    if !report.diagnostics.is_empty() {
        object.insert(
            "diagnostics".to_string(),
            Value::Array(
                report
                    .diagnostics
                    .iter()
                    .map(|diagnostic| {
                        let mut value = json!({
                            "code": validation_code_json(&diagnostic.code),
                            "family": diagnostic.family,
                            "message": diagnostic.message,
                        });
                        let object = value
                            .as_object_mut()
                            .expect("validation diagnostic data should be an object");
                        insert_optional(object, "key", &diagnostic.key);
                        insert_optional(object, "expected", &diagnostic.expected);
                        insert_optional(object, "actual", &diagnostic.actual);
                        value
                    })
                    .collect(),
            ),
        );
    }

    data
}

fn insert_optional(
    object: &mut serde_json::Map<String, Value>,
    key: &'static str,
    value: &Option<String>,
) {
    if let Some(value) = value {
        object.insert(key.to_string(), Value::String(value.clone()));
    }
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

fn validation_code_json(code: &ValidationCode) -> &'static str {
    match code {
        ValidationCode::Ok => "ok",
        ValidationCode::IndexUnavailable => "index_unavailable",
        ValidationCode::MissingArtifactMetadata => "missing_artifact_metadata",
        ValidationCode::MissingRequiredMetadata => "missing_required_metadata",
        ValidationCode::UnsupportedContractVersion => "unsupported_contract_version",
        ValidationCode::UnsupportedSchemaVersion => "unsupported_schema_version",
        ValidationCode::ArtifactContractViolation => "artifact_contract_violation",
        ValidationCode::InvalidSourceMetadata => "invalid_source_metadata",
        ValidationCode::StaleSourceSignature => "stale_source_signature",
        ValidationCode::EmbeddingMismatch => "embedding_mismatch",
        ValidationCode::FtsMismatch => "fts_mismatch",
        ValidationCode::ManifestMismatch => "manifest_mismatch",
        ValidationCode::VectorExtensionUnavailable => "vector_extension_unavailable",
        ValidationCode::QueryFailed => "query_failed",
    }
}
