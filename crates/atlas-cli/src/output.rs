use atlas_cli_support::{CliError, write_json_data, write_json_failure};
use atlas_runtime::IndexDiagnosticError;
use serde::Serialize;
use std::process::ExitCode;

pub(crate) fn index_diagnostic<T: Serialize>(
    result: Result<T, IndexDiagnosticError>,
    json: bool,
    human: impl FnOnce(T),
) -> Result<ExitCode, String> {
    #[derive(Serialize)]
    struct Valid<T> {
        valid: bool,
        #[serde(flatten)]
        report: T,
    }
    match result {
        Ok(report) => {
            tracing::info!(target:"atlas_progress",complete=true,"Index diagnostic finished");
            if json {
                write_json_data(Valid {
                    valid: true,
                    report,
                })?;
            } else {
                human(report);
            }
            Ok(ExitCode::SUCCESS)
        }
        Err(error) => {
            tracing::info!(target:"atlas_progress",complete=true,"Index diagnostic failed");
            if json {
                let reason = CliError {
                    code: error.code().into(),
                    message: error.to_string(),
                };
                match error {
                    IndexDiagnosticError::CouldNotRun(_) => write_json_failure(reason)?,
                    IndexDiagnosticError::InvalidArtifact(_)
                    | IndexDiagnosticError::MissingVectors => {
                        write_json_data(serde_json::json!({"valid": false, "reason": reason}))?;
                    }
                }
            } else {
                eprintln!("{error}");
            }
            Ok(ExitCode::from(3))
        }
    }
}
