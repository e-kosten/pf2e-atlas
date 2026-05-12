#![deny(unsafe_code)]

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use atlas_domain::{ArtifactValidationReport, ValidationCode, ValidationStatus};
use atlas_index::validate_index;

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
        "validate-index" => run_validate_index(args),
        _ => Err(format!("unknown command `{command}`")),
    }
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
            source_signature: None,
            embedding_model_id: None,
            embedding_dimensions: None,
            missing_keys: Vec::new(),
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
        "atlas validate-index --index <path> [--json]\n\
         \n\
         Commands:\n\
           validate-index   Open an index read-only and validate Rust artifact metadata"
    );
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
        ValidationCode::QueryFailed => "query-failed",
    }
}
