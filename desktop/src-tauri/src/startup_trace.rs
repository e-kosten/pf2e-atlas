//! Opt-in diagnostics for the bundled startup document and native handoff.

use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};
use tauri::{AppHandle, Manager, WebviewWindow, ipc::CapabilityBuilder};

use crate::bundled_origin;

pub fn enabled() -> bool {
    std::env::var_os("ATLAS_PILOT_STARTUP_TRACE").is_some_and(|value| value == "1")
}

pub fn enable(app: &AppHandle) -> tauri::Result<()> {
    if enabled() {
        app.add_capability(
            CapabilityBuilder::new("atlas-debug-startup-trace")
                .webview("atlas-primary")
                .webview("bootstrap")
                .permission("allow-desktop-startup-trace"),
        )?;
    }
    Ok(())
}

pub fn record(event: &str, fields: Value) {
    if enabled() {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        eprintln!(
            "atlas-startup {}",
            json!({ "at_ms": timestamp, "pid": std::process::id(), "event": event, "fields": fields })
        );
    }
}

pub fn received(window: &WebviewWindow, command: &str, guard: &Result<(), String>) {
    if !enabled() {
        return;
    }
    let document = match window.url() {
        Ok(url) if bundled_origin(&url) => "bundled",
        Ok(_) => "outside-bootstrap",
        Err(_) => "unavailable",
    };
    record(
        "native-command-received",
        json!({ "command": command, "window": window.label(), "document": document, "guard_error": guard.as_ref().err() }),
    );
}

#[tauri::command]
pub fn desktop_startup_trace(window: WebviewWindow, entry: Value) {
    // The dynamically added local capability checks the initiating document.
    // The live URL can already be the product when its outgoing page logs.
    record(
        "bootstrap",
        json!({ "window": window.label(), "entry": entry }),
    );
}
