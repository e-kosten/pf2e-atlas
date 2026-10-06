use serde::Serialize;

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
pub struct CliError {
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

pub fn write_json_data<T>(data: T) -> Result<(), String>
where
    T: Serialize,
{
    let body = serde_json::to_string_pretty(&SuccessEnvelope { status: "ok", data })
        .map_err(|error| error.to_string())?;
    println!("{body}");
    Ok(())
}

pub fn write_json_error(code: &'static str, message: String) -> Result<(), String> {
    let body = serde_json::to_string_pretty(&ErrorEnvelope {
        status: "error",
        error: CliError { code, message },
    })
    .map_err(|error| error.to_string())?;
    println!("{body}");
    Ok(())
}

pub fn write_json_error_data<T>(code: &'static str, message: String, data: T) -> Result<(), String>
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
