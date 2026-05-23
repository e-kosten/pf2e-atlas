use std::path::Path;
use std::process::Command;
use std::time::Instant;

use serde_json::Value as JsonValue;

#[derive(Debug)]
pub(crate) struct AtlasJsonRun {
    pub(crate) elapsed_ms: u128,
    pub(crate) json: Option<JsonValue>,
    pub(crate) error: Option<String>,
}

pub(crate) fn run_atlas_json(atlas_bin: &Path, args: &[String]) -> AtlasJsonRun {
    let mut command = Command::new(atlas_bin);
    command.args(args);

    let started = Instant::now();
    let output = match command.output() {
        Ok(output) => output,
        Err(error) => {
            return AtlasJsonRun {
                elapsed_ms: started.elapsed().as_millis(),
                json: None,
                error: Some(format!("failed to run atlas: {error}")),
            };
        }
    };
    let elapsed_ms = started.elapsed().as_millis();
    if !output.status.success() {
        return AtlasJsonRun {
            elapsed_ms,
            json: None,
            error: Some(summarize_atlas_command_error(&output)),
        };
    }
    let json = match serde_json::from_slice::<JsonValue>(&output.stdout) {
        Ok(json) => json,
        Err(error) => {
            return AtlasJsonRun {
                elapsed_ms,
                json: None,
                error: Some(format!("failed to parse atlas JSON: {error}")),
            };
        }
    };
    AtlasJsonRun {
        elapsed_ms,
        json: Some(json),
        error: None,
    }
}

pub(crate) fn summarize_json(value: &JsonValue) -> String {
    let value = value.to_string();
    if value.len() > 180 {
        format!("{}...", &value[..180])
    } else {
        value
    }
}

fn summarize_atlas_command_error(output: &std::process::Output) -> String {
    let stdout_json = serde_json::from_slice::<JsonValue>(&output.stdout).ok();
    let json_error = stdout_json
        .as_ref()
        .and_then(|json| json.pointer("/error"))
        .map(|error| {
            let code = error
                .pointer("/code")
                .and_then(JsonValue::as_str)
                .unwrap_or("unknown");
            let message = error
                .pointer("/message")
                .and_then(JsonValue::as_str)
                .unwrap_or("no message");
            format!("{code}: {}", truncate_message(message, 420))
        });
    let stderr = String::from_utf8_lossy(&output.stderr);
    match json_error {
        Some(error) if stderr.trim().is_empty() => format!("status {} {error}", output.status),
        Some(error) => format!(
            "status {} {error}; stderr {}",
            output.status,
            truncate_message(stderr.trim(), 240)
        ),
        None => format!(
            "status {} stderr {} stdout {}",
            output.status,
            truncate_message(stderr.trim(), 240),
            truncate_message(&String::from_utf8_lossy(&output.stdout), 420)
        ),
    }
}

fn truncate_message(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    let mut truncated = value
        .chars()
        .take(max_chars.saturating_sub(3))
        .collect::<String>();
    truncated.push_str("...");
    truncated
}
