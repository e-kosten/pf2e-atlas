use serde::Serialize;

#[derive(Debug, Serialize)]
struct DataEnvelope<T>
where
    T: Serialize,
{
    status: JsonStatus,
    data: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CliError {
    pub code: String,
    pub message: String,
}

/// Overall result of a command or batch, independent of the process exit class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JsonStatus {
    Ok,
    Partial,
    Error,
}

pub fn write_json_response<T: Serialize>(status: JsonStatus, data: T) -> Result<(), String> {
    let body = serde_json::to_string_pretty(&DataEnvelope { status, data })
        .map_err(|error| error.to_string())?;
    println!("{body}");
    Ok(())
}

/// Preserve the caller's typed error, including structured details and recovery hints.
pub fn write_json_failure<T: Serialize>(error: T) -> Result<(), String> {
    #[derive(Serialize)]
    struct Failure<T> {
        status: JsonStatus,
        error: T,
    }
    let body = serde_json::to_string_pretty(&Failure {
        status: JsonStatus::Error,
        error,
    })
    .map_err(|error| error.to_string())?;
    println!("{body}");
    Ok(())
}

pub fn write_json_data<T>(data: T) -> Result<(), String>
where
    T: Serialize,
{
    write_json_response(JsonStatus::Ok, data)
}

pub fn write_json_error(code: impl Into<String>, message: String) -> Result<(), String> {
    write_json_failure(CliError {
        code: code.into(),
        message,
    })
}
