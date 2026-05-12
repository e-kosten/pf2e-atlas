#![deny(unsafe_code)]

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use atlas_domain::{ArtifactValidationReport, ValidationCode, ValidationStatus};
use atlas_index::validate_index;
use atlas_ingest::{BuildArtifactOptions, build_minimal_artifact};

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() || args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return Ok(ExitCode::SUCCESS);
    }

    let command = args.remove(0);
    match command.as_str() {
        "build-index" => run_build_index(args),
        "validate-index" => run_validate_index(args),
        _ => Err(format!("unknown command `{command}`")),
    }
}

fn run_build_index(args: Vec<String>) -> Result<ExitCode, String> {
    let options = parse_build_index(args)?;
    let report = build_minimal_artifact(BuildArtifactOptions {
        source_root: options.source,
        output_path: options.output,
        manifest_path: options.manifest,
    })
    .map_err(|error| error.to_string())?;

    if options.json {
        let body = serde_json::json!({
            "status": "ok",
            "output": report.output_path.display().to_string(),
            "pack_count": report.pack_count,
            "record_count": report.record_count,
            "warnings": report.warnings,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&body).map_err(|error| error.to_string())?
        );
    } else {
        println!(
            "ok: wrote {} records from {} packs to {}",
            report.record_count,
            report.pack_count,
            report.output_path.display()
        );
    }

    Ok(ExitCode::SUCCESS)
}

fn run_validate_index(args: Vec<String>) -> Result<ExitCode, String> {
    let options = parse_validate_index(args)?;
    let report = match validate_index(&options.index) {
        Ok(report) => report,
        Err(error) => ArtifactValidationReport {
            status: ValidationStatus::Error,
            code: match error {
                atlas_index::IndexValidationError::Unavailable(_) => {
                    ValidationCode::IndexUnavailable
                }
                atlas_index::IndexValidationError::QueryFailed(_) => ValidationCode::QueryFailed,
            },
            index: options.index.display().to_string(),
            message: error.to_string(),
            artifact_contract_version: None,
            schema_version: None,
            source_kind: None,
            source_signature: None,
            source_record_count: None,
            content_hash_algorithm: None,
            embedding_provider_family: None,
            embedding_model_id: None,
            embedding_model_revision: None,
            embedding_tokenizer_id: None,
            embedding_pooling: None,
            embedding_normalization: None,
            embedding_dimensions: None,
            embedding_dtype: None,
            embedding_distance_metric: None,
            embedding_document_prefix: None,
            embedding_query_prefix: None,
            fts_tokenizer: None,
            adjacent_manifest_path: None,
            missing_keys: Vec::new(),
            diagnostics: Vec::new(),
            legacy_schema_version: None,
        },
    };

    let exit_code = match report.status {
        ValidationStatus::Ok => ExitCode::SUCCESS,
        ValidationStatus::Error => ExitCode::from(1),
    };

    if options.json {
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

fn print_help() {
    println!(
        "atlas build-index --source <path> --output <path> [--manifest <path>] [--json]\n\
         atlas validate-index --index <path> [--json]\n\
         \n\
         Commands:\n\
           build-index      Build a minimal Rust artifact from Foundry source files\n\
           validate-index   Open an index read-only and validate Rust artifact metadata"
    );
}

#[derive(Debug)]
struct BuildIndexOptions {
    source: PathBuf,
    output: PathBuf,
    manifest: Option<PathBuf>,
    json: bool,
}

fn parse_build_index(args: Vec<String>) -> Result<BuildIndexOptions, String> {
    let mut source = None;
    let mut output = None;
    let mut manifest = None;
    let mut json = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--source" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--source requires a path".to_string())?;
                source = Some(PathBuf::from(value));
                i += 2;
            }
            "--output" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--output requires a path".to_string())?;
                output = Some(PathBuf::from(value));
                i += 2;
            }
            "--manifest" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--manifest requires a path".to_string())?;
                manifest = Some(PathBuf::from(value));
                i += 2;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            flag if flag.starts_with("--") => {
                return Err(format!("unknown build-index option `{flag}`"));
            }
            value => return Err(format!("unexpected build-index argument `{value}`")),
        }
    }

    Ok(BuildIndexOptions {
        source: source.ok_or_else(|| "build-index requires --source <path>".to_string())?,
        output: output.ok_or_else(|| "build-index requires --output <path>".to_string())?,
        manifest,
        json,
    })
}

#[derive(Debug)]
struct ValidateIndexOptions {
    index: PathBuf,
    json: bool,
}

fn parse_validate_index(args: Vec<String>) -> Result<ValidateIndexOptions, String> {
    let mut index = None;
    let mut json = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--index" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--index requires a path".to_string())?;
                index = Some(PathBuf::from(value));
                i += 2;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            flag if flag.starts_with("--") => {
                return Err(format!("unknown validate-index option `{flag}`"));
            }
            value => return Err(format!("unexpected validate-index argument `{value}`")),
        }
    }

    let index = index.ok_or_else(|| "validate-index requires --index <path>".to_string())?;
    Ok(ValidateIndexOptions { index, json })
}

fn validation_code_label(code: &ValidationCode) -> &'static str {
    match code {
        ValidationCode::Ok => "ok",
        ValidationCode::IndexUnavailable => "index-unavailable",
        ValidationCode::MissingArtifactMetadata => "missing-artifact-metadata",
        ValidationCode::MissingRequiredMetadata => "missing-required-metadata",
        ValidationCode::UnsupportedContractVersion => "unsupported-contract-version",
        ValidationCode::UnsupportedSchemaVersion => "unsupported-schema-version",
        ValidationCode::InvalidSourceMetadata => "invalid-source-metadata",
        ValidationCode::StaleSourceSignature => "stale-source-signature",
        ValidationCode::EmbeddingMismatch => "embedding-mismatch",
        ValidationCode::FtsMismatch => "fts-mismatch",
        ValidationCode::ManifestMismatch => "manifest-mismatch",
        ValidationCode::QueryFailed => "query-failed",
    }
}
