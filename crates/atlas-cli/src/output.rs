use std::process::ExitCode;
use std::time::Duration;

use atlas_index::{ArtifactValidationReport, ValidationCode, ValidationStatus};
use serde::Serialize;
use serde_json::{Map, Value, json};

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

#[derive(Debug, Serialize)]
struct ErrorEnvelopeWithData<T>
where
    T: Serialize,
{
    status: &'static str,
    error: CliErrorWithData<T>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CliError {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Serialize)]
struct CliErrorWithData<T>
where
    T: Serialize,
{
    code: &'static str,
    message: String,
    data: T,
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

pub(crate) fn format_duration_ms(duration_ms: u128) -> String {
    let millis = u64::try_from(duration_ms).unwrap_or(u64::MAX);
    format_duration(Duration::from_millis(millis))
}

fn format_duration(duration: Duration) -> String {
    let total_millis = duration.as_millis();
    if total_millis < 1_000 {
        return format!("{total_millis}ms");
    }

    let total_seconds = duration.as_secs();
    let millis = duration.subsec_millis();
    if total_seconds < 60 {
        return format!("{}.{:01}s", total_seconds, millis / 100);
    }

    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    if minutes < 60 {
        return format!("{minutes}m {seconds:02}.{:01}s", millis / 100);
    }

    let hours = minutes / 60;
    let minutes = minutes % 60;
    format!("{hours}h {minutes:02}m {seconds:02}s")
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

pub(crate) fn write_json_error_data<T>(
    code: &'static str,
    message: String,
    data: T,
) -> Result<(), String>
where
    T: Serialize,
{
    let body = serde_json::to_string_pretty(&ErrorEnvelopeWithData {
        status: "error",
        error: CliErrorWithData {
            code,
            message,
            data,
        },
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
    let mut object = Map::from_iter([
        (
            "valid".to_string(),
            Value::Bool(report.status == ValidationStatus::Ok),
        ),
        (
            "code".to_string(),
            Value::String(validation_code_json(&report.code).to_string()),
        ),
        ("index".to_string(), Value::String(report.index.clone())),
        ("message".to_string(), Value::String(report.message.clone())),
    ]);

    insert_optional(
        &mut object,
        "artifact_contract_version",
        &report.artifact_contract_version,
    );
    insert_optional(&mut object, "schema_version", &report.schema_version);
    insert_optional(&mut object, "source_kind", &report.source_kind);
    insert_optional(&mut object, "source_signature", &report.source_signature);
    insert_optional(
        &mut object,
        "source_record_count",
        &report.source_record_count,
    );
    insert_optional(
        &mut object,
        "artifact_record_count",
        &report.artifact_record_count,
    );
    insert_optional(
        &mut object,
        "generated_record_count",
        &report.generated_record_count,
    );
    insert_optional(
        &mut object,
        "content_hash_algorithm",
        &report.content_hash_algorithm,
    );
    insert_optional(
        &mut object,
        "embedding_provider_family",
        &report.embedding_provider_family,
    );
    insert_optional(
        &mut object,
        "embedding_model_id",
        &report.embedding_model_id,
    );
    insert_optional(
        &mut object,
        "embedding_model_revision",
        &report.embedding_model_revision,
    );
    insert_optional(
        &mut object,
        "embedding_tokenizer_id",
        &report.embedding_tokenizer_id,
    );
    insert_optional(&mut object, "embedding_pooling", &report.embedding_pooling);
    insert_optional(
        &mut object,
        "embedding_normalization",
        &report.embedding_normalization,
    );
    insert_optional(
        &mut object,
        "embedding_dimensions",
        &report.embedding_dimensions,
    );
    insert_optional(&mut object, "embedding_dtype", &report.embedding_dtype);
    insert_optional(
        &mut object,
        "embedding_distance_metric",
        &report.embedding_distance_metric,
    );
    insert_optional(
        &mut object,
        "embedding_document_prefix",
        &report.embedding_document_prefix,
    );
    insert_optional(
        &mut object,
        "embedding_query_prefix",
        &report.embedding_query_prefix,
    );
    insert_optional(
        &mut object,
        "embedding_unit_policy_version",
        &report.embedding_unit_policy_version,
    );
    insert_optional(&mut object, "fts_tokenizer", &report.fts_tokenizer);
    insert_optional(
        &mut object,
        "adjacent_manifest_path",
        &report.adjacent_manifest_path,
    );
    insert_optional(
        &mut object,
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
                        let mut object = Map::from_iter([
                            (
                                "code".to_string(),
                                Value::String(validation_code_json(&diagnostic.code).to_string()),
                            ),
                            ("family".to_string(), json!(diagnostic.family)),
                            (
                                "message".to_string(),
                                Value::String(diagnostic.message.clone()),
                            ),
                        ]);
                        insert_optional(&mut object, "key", &diagnostic.key);
                        insert_optional(&mut object, "expected", &diagnostic.expected);
                        insert_optional(&mut object, "actual", &diagnostic.actual);
                        Value::Object(object)
                    })
                    .collect(),
            ),
        );
    }

    Value::Object(object)
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use atlas_index::{
        ArtifactContractFamily, ArtifactValidationDiagnostic, ArtifactValidationReport,
    };

    use super::*;

    #[test]
    fn validation_report_json_includes_optional_metadata_and_diagnostics() {
        let report = ArtifactValidationReport {
            status: ValidationStatus::Error,
            code: ValidationCode::MissingRequiredMetadata,
            index: "artifact.sqlite".to_string(),
            message: "missing metadata".to_string(),
            artifact_contract_version: Some("1".to_string()),
            schema_version: Some("1".to_string()),
            source_kind: Some("foundry-pf2e".to_string()),
            source_signature: Some("signature".to_string()),
            source_record_count: Some("10".to_string()),
            artifact_record_count: Some("12".to_string()),
            generated_record_count: Some("2".to_string()),
            content_hash_algorithm: Some("sha256".to_string()),
            embedding_provider_family: Some("fastembed".to_string()),
            embedding_model_id: Some("model".to_string()),
            embedding_model_revision: Some("revision".to_string()),
            embedding_tokenizer_id: Some("tokenizer".to_string()),
            embedding_pooling: Some("mean".to_string()),
            embedding_normalization: Some("l2".to_string()),
            embedding_dimensions: Some("384".to_string()),
            embedding_dtype: Some("f32".to_string()),
            embedding_distance_metric: Some("cosine".to_string()),
            embedding_document_prefix: Some("document:".to_string()),
            embedding_query_prefix: Some("query:".to_string()),
            embedding_unit_policy_version: Some("policy".to_string()),
            fts_tokenizer: Some("unicode61".to_string()),
            adjacent_manifest_path: Some("manifest.json".to_string()),
            missing_keys: vec!["schema_version".to_string()],
            diagnostics: vec![ArtifactValidationDiagnostic {
                code: ValidationCode::MissingRequiredMetadata,
                family: ArtifactContractFamily::Contract,
                message: "missing schema_version".to_string(),
                key: Some("schema_version".to_string()),
                expected: Some("present".to_string()),
                actual: Some("missing".to_string()),
            }],
            legacy_schema_version: Some("0".to_string()),
        };

        let data = validation_report_data(&report);

        assert_eq!(data["valid"], false);
        assert_eq!(data["code"], "missing_required_metadata");
        assert_eq!(data["artifact_contract_version"], "1");
        assert_eq!(data["embedding_model_id"], "model");
        assert_eq!(data["missing_keys"][0], "schema_version");
        assert_eq!(data["diagnostics"][0]["key"], "schema_version");
        assert_eq!(data["diagnostics"][0]["expected"], "present");
        assert_eq!(data["diagnostics"][0]["actual"], "missing");
        assert_eq!(data["legacy_schema_version"], "0");
    }

    #[test]
    fn formats_elapsed_durations_for_human_output() {
        assert_eq!(format_duration(Duration::from_millis(438)), "438ms");
        assert_eq!(format_duration_ms(438), "438ms");
        assert_eq!(format_duration(Duration::from_millis(1_617)), "1.6s");
        assert_eq!(format_duration(Duration::from_millis(152_738)), "2m 32.7s");
        assert_eq!(
            format_duration(Duration::from_millis(3_723_400)),
            "1h 02m 03s"
        );
    }
}
